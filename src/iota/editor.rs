use std::path::PathBuf;
use std::sync::{Mutex, Arc};
use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc::channel;
use std::env;
use std::rc::Rc;
use std::char;

use rustbox::{RustBox, Event};
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;

use input::Input;
use keyboard::Key;
use view::View;
use modes::{Mode, ModeType, InsertMode, NormalMode};
use overlay::{Overlay, OverlayEvent};
use buffer::Buffer;
use command::Command;
use command::{Action, BuilderEvent, Operation, Instruction};


pub struct Options {
    pub syntax_enabled: bool,
}

impl Default for Options {
    fn default() -> Options {
        Options {
            syntax_enabled: false,
        }
    }
}


/// The main Editor structure
///
/// This is the top-most structure in Iota.
pub struct Editor<'e> {
    buffers: Vec<Arc<Mutex<Buffer>>>,
    view: View,
    running: bool,
    rb: RustBox,
    mode: Box<Mode + 'e>,
    options: Options,

    command_queue: Receiver<Command>,
    command_sender: Sender<Command>,
}

impl<'e> Editor<'e> {

    /// Create a new Editor instance from the given source
    pub fn new(source: Input, mode: Box<Mode + 'e>, rb: RustBox, opts: Options) -> Editor<'e> {
        let height = rb.height();
        let width = rb.width();

        let (snd, recv) = channel();

        let mut buffers = Vec::new();

        // TODO: load custom syntax files rather than using defaults
        //       see below
        let mut ps = SyntaxSet::load_defaults_nonewlines();
        ps.link_syntaxes();

        let buffer = match source {
            Input::Filename(path) => {
                match path {
                    Some(path) => Buffer::new_with_syntax(PathBuf::from(path), &ps),
                    None       => Buffer::new(),
                }
            },
            Input::Stdin(reader) => {
                Buffer::from(reader)
            }
        };
        buffers.push(Arc::new(Mutex::new(buffer)));

        // NOTE: this will only work on linux
        // TODO: make this more cross-platform friendly
        let mut subl_config = env::home_dir().unwrap();
        subl_config.push(".config/sublime-text-3/Packages/Base16/");

        let (theme_name, ts) = if subl_config.exists() {
            (String::from("base16-default-dark"),
            Rc::new(ThemeSet::load_from_folder(subl_config).unwrap()))
        } else {
            (String::from("base16-eighties.dark"),
            Rc::new(ThemeSet::load_defaults()))
        };

        let view = View::new(buffers[0].clone(), ts.clone(), theme_name,  width, height);

        Editor {
            buffers: buffers,
            view: view,
            running: true,
            rb: rb,
            mode: mode,
            options: opts,

            command_queue: recv,
            command_sender: snd,
        }
    }

    /// Handle key events
    ///
    /// Key events can be handled in an Overlay, OR in the current Mode.
    ///
    /// If there is an active Overlay, the key event is sent there, which gives
    /// back an OverlayEvent. We then parse this OverlayEvent and determine if
    /// the Overlay is finished and can be cleared. The response from the
    /// Overlay is then converted to a Command and sent off to be handled.
    ///
    /// If there is no active Overlay, the key event is sent to the current
    /// Mode, which returns a Command which we dispatch to handle_command.
    fn handle_key_event(&mut self, key: Option<Key>) {
        let key = match key {
            Some(k) => k,
            None => return
        };

        let mut remove_overlay = false;
        let command = match self.view.overlay {
            Overlay::None => self.mode.handle_key_event(key),
            _             => {
                let event = self.view.overlay.handle_key_event(key);
                match event {
                    OverlayEvent::Finished(response) => {
                        remove_overlay = true;
                        self.handle_overlay_response(response)
                    }

                    _ => { BuilderEvent::Incomplete }
                }
            }
        };

        if remove_overlay {
            self.view.overlay = Overlay::None;
            self.view.clear(&mut self.rb);
        }

        if let BuilderEvent::Complete(c) = command {
            let _ = self.command_sender.send(c);
        }
    }

    /// Translate the response from an Overlay to a Command wrapped in a BuilderEvent
    ///
    /// In most cases, we will just want to convert the response directly to
    /// a Command, however in some cases we will want to perform other actions
    /// first, such as in the case of Overlay::SavePrompt.
    fn handle_overlay_response(&mut self, response: Option<String>) -> BuilderEvent {
        // FIXME: This entire method neext to be updated
        match response {
            Some(data) => {
                match self.view.overlay {

                    // FIXME: this is just a temporary fix
                    Overlay::Prompt { ref data, .. } => {
                        match &**data {
                            // FIXME: need to find a better system for these commands
                            //        They should be chainable
                            //          ie: wq - save & quit
                            //        They should also take arguments
                            //          ie w file.txt - write buffer to file.txt
                            "q" | "quit" => BuilderEvent::Complete(Command::exit_editor()),
                            "w" | "write" => BuilderEvent::Complete(Command::save_buffer()),

                            _ => BuilderEvent::Incomplete
                        }
                    }

                    Overlay::SavePrompt { .. } => {
                        if data.is_empty() {
                            BuilderEvent::Invalid
                        } else {
                            let path = PathBuf::from(&*data);
                            self.view.buffer.lock().unwrap().file_path = Some(path);
                            BuilderEvent::Complete(Command::save_buffer())
                        }
                    }

                    Overlay::SelectFile { .. } => {
                        let path = PathBuf::from(data);
                        let buffer = Arc::new(Mutex::new(Buffer::from(path)));
                        self.buffers.push(buffer.clone());
                        self.view.set_buffer(buffer.clone());
                        self.view.clear(&mut self.rb);
                        BuilderEvent::Complete(Command::noop())
                    }

                    _ => BuilderEvent::Incomplete,
                }
            }
            None => BuilderEvent::Incomplete
        }
    }

    /// Handle resize events
    ///
    /// width and height represent the new height of the window.
    fn handle_resize_event(&mut self, width: usize, height: usize) {
        self.view.resize(width, height);
    }

    /// Draw the current view to the frontend
    fn draw(&mut self) {
        self.view.draw(&mut self.rb, self.options.syntax_enabled);
    }

    /// Handle the given command, performing the associated action
    fn handle_command(&mut self, command: Command) {
        let repeat = if command.number > 0 {
            command.number
        } else { 1 };
        for _ in 0..repeat {
            match command.action {
                Action::Instruction(i) => self.handle_instruction(i, command),
                Action::Operation(o) => self.handle_operation(o, command),
            }
        }
    }


    fn handle_instruction(&mut self, instruction: Instruction, command: Command) {
        match instruction {
            Instruction::SaveBuffer => { self.view.try_save_buffer() }
            Instruction::ExitEditor => {
                if self.view.buffer_is_dirty() {
                    let _ = self.command_sender.send(Command::show_message("Unsaved changes"));
                } else {
                    self.running = false;
                }

            }
            Instruction::SetMark(mark) => {
                if let Some(object) = command.object {
                    self.view.move_mark(mark, object)
                }
            }
            Instruction::SetOverlay(overlay_type) => {
                self.view.set_overlay(overlay_type)
            }
            Instruction::SetMode(mode) => {
                match mode {
                    ModeType::Insert => { self.mode = Box::new(InsertMode::new()) }
                    ModeType::Normal => { self.mode = Box::new(NormalMode::new()) }
                }
            }
            Instruction::SwitchToLastBuffer => {
                self.view.switch_last_buffer();
                self.view.clear(&mut self.rb);
            }
            Instruction::ShowMessage(msg) => {
                self.view.show_message(msg)
            }

            _ => {}
        }
    }

    fn handle_operation(&mut self, operation: Operation, command: Command) {
        match operation {
            Operation::Insert(c) => {
                for _ in 0..command.number {
                    self.view.insert_char(c)
                }
            }
            Operation::DeleteObject => {
                if let Some(obj) = command.object {
                    self.view.delete_object(obj);
                }
            }
            Operation::DeleteFromMark(m) => {
                if command.object.is_some() {
                    self.view.delete_from_mark_to_object(m, command.object.unwrap())
                }
            }
            Operation::Undo => { self.view.undo() }
            Operation::Redo => { self.view.redo() }
        }
    }

    /// Start Iota!
    pub fn start(&mut self) {
        while self.running {
            self.draw();
            self.rb.present();
            self.view.maybe_clear_message();

            match self.rb.poll_event(true) {
                Ok(Event::KeyEventRaw(_, key, ch)) => {
                    let k = match key {
                        0 => char::from_u32(ch).map(Key::Char),
                        a => Key::from_special_code(a),
                    };
                    self.handle_key_event(k)
                },
                Ok(Event::ResizeEvent(width, height)) => self.handle_resize_event(width as usize, height as usize),

                _ => {}
            }

            while let Ok(message) = self.command_queue.try_recv() {
                self.handle_command(message)
            }
        }
    }
}
