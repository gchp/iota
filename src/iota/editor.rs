use std::path::PathBuf;
use std::sync::{Mutex, Arc};
use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc::channel;
use std::env;
use std::rc::Rc;

#[cfg(feature="syntax-highlighting")] use syntect::highlighting::ThemeSet;
#[cfg(feature="syntax-highlighting")] use syntect::parsing::SyntaxSet;

use input::Input;
use keyboard::Key;
use view::View;
use frontends::{Frontend, EditorEvent};
use modes::{Mode, ModeType, InsertMode, NormalMode};
use buffer::Buffer;
use command::Command;
use command::{Action, BuilderEvent, Operation, Instruction};


/// The main Editor structure
///
/// This is the top-most structure in Iota.
pub struct Editor<'e, T: Frontend> {
    buffers: Vec<Arc<Mutex<Buffer>>>,
    view: View,
    running: bool,
    frontend: T,
    mode: Box<Mode + 'e>,

    command_queue: Receiver<Command>,
    command_sender: Sender<Command>,
}

impl<'e, T: Frontend> Editor<'e, T> {

    /// Create a new Editor instance from the given source
    #[cfg(feature="syntax-highlighting")]
    pub fn new(source: Input, mode: Box<Mode + 'e>, frontend: T) -> Editor<'e, T> {
        let height = frontend.get_window_height();
        let width = frontend.get_window_width();

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
            frontend: frontend,
            mode: mode,

            command_queue: recv,
            command_sender: snd,
        }
    }

    /// Create a new Editor instance from the given source
    #[cfg(not(feature="syntax-highlighting"))]
    pub fn new(source: Input, mode: Box<Mode + 'e>, frontend: T) -> Editor<'e, T> {
        let height = frontend.get_window_height();
        let width = frontend.get_window_width();

        let (snd, recv) = channel();

        let mut buffers = Vec::new();

        let buffer = Buffer::from(source);
        buffers.push(Arc::new(Mutex::new(buffer)));

        let view = View::new(buffers[0].clone(), width, height);
        Editor {
            buffers: buffers,
            view: view,
            running: true,
            frontend: frontend,
            mode: mode,

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

        let command = match self.view.overlay {
            Some(ref mut overlay) => overlay.handle_key_event(key),
            None => self.mode.handle_key_event(key),
        };

        if let BuilderEvent::Complete(c) = command {
            let _ = self.command_sender.send(c);
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
        self.view.draw(&mut self.frontend);
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
                self.view.clear(&mut self.frontend);
            }
            Instruction::ShowMessage(msg) => {
                self.view.show_message(msg)
            }
            Instruction::ClearOverlay => {
                self.view.overlay = None;
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
            self.frontend.present();
            self.view.maybe_clear_message();

            match self.frontend.poll_event() {
                Some(EditorEvent::KeyEvent(key))         => self.handle_key_event(key),
                Some(EditorEvent::Resize(width, height)) => self.handle_resize_event(width, height),

                _ => {}
            }

            while let Ok(message) = self.command_queue.try_recv() {
                self.handle_command(message)
            }
        }
    }
}
