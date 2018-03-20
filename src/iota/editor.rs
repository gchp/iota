use std::path::PathBuf;
use std::sync::{Mutex, Arc};
use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc::channel;
use std::collections::HashMap;

use rustbox::{RustBox, Event};

use input::Input;
use keyboard::Key;
use view::View;
use modes::{Mode, ModeType, InsertMode, NormalMode};
use buffer::Buffer;
use command::Command;
use command::{Action, BuilderEvent, BuilderArgs, Operation, Instruction};
use textobject::{ Offset, Kind, Anchor };
use buffer::Mark;

type EditorCommand = fn(Option<BuilderArgs>) -> Command;
lazy_static! {
    pub static ref ALL_COMMANDS: HashMap<&'static str, EditorCommand> = {
        let mut map: HashMap<&'static str, EditorCommand> = HashMap::new();

        map.insert("editor::quit", Command::exit_editor);
        map.insert("editor::save_buffer", Command::save_buffer);

        map.insert("buffer::move_cursor_forward_char", Command::move_cursor_forward_char);
        map.insert("buffer::move_cursor_backward_char", Command::move_cursor_backward_char);
        map.insert("buffer::move_cursor_forward_line", Command::move_cursor_forward_line);
        map.insert("buffer::move_cursor_backward_line", Command::move_cursor_backward_line);

        map.insert("editor::set_overlay_command_prompt", Command::set_overlay_command_prompt);

        map
    };
}

/// The main Editor structure
///
/// This is the top-most structure in Iota.
pub struct Editor<'e> {
    buffers: Vec<Arc<Mutex<Buffer>>>,
    view: View<'e>,
    running: bool,
    rb: RustBox,
    mode: Box<Mode + 'e>,

    command_queue: Receiver<Command>,
    command_sender: Sender<Command>,
}

impl<'e> Editor<'e> {

    /// Create a new Editor instance from the given source
    pub fn new(source: Input, mode: Box<Mode + 'e>, rb: RustBox) -> Editor<'e> {
        let height = rb.height();
        let width = rb.width();

        let (snd, recv) = channel();

        let mut buffers = Vec::new();

        let buffer = match source {
            Input::Filename(path) => {
                match path {
                    Some(path) => Buffer::from(PathBuf::from(path)),
                    None       => Buffer::new(),
                }
            },
            Input::Stdin(reader) => {
                Buffer::from(reader)
            }
        };
        buffers.push(Arc::new(Mutex::new(buffer)));

        let view = View::new(buffers[0].clone(), width, height);

        Editor {
            buffers: buffers,
            view: view,
            running: true,
            rb: rb,
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
    fn handle_key_event(&mut self, event: Event) {
        let key = Key::from_event(&mut self.rb, event);
 
        let key = match key {
            Some(k) => k,
            None => return
        };

        let command = match self.view.overlay {
            None                  => self.mode.handle_key_event(key),
            Some(ref mut overlay) => overlay.handle_key_event(key),
        };

        if let BuilderEvent::Complete(c, args) = command {
            self.view.overlay = None;
            self.view.clear(&mut self.rb);

            match ALL_COMMANDS.get(&*c) {
                Some(cmd) => {
                    let cmd = cmd(args);
                    let _ = self.command_sender.send(cmd);
                }
                None => {
                    panic!("Unknown command: {}", c);
                }
            }

            // let _ = self.command_sender.send(c);
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
        self.view.draw(&mut self.rb);
    }

    /// Handle the given command, performing the associated action
    fn handle_command(&mut self, command: Command) {
        let repeat = if command.number > 0 {
            command.number
        } else { 1 };
        for _ in 0..repeat {
            match command.action {
                Action::Instruction(_) => self.handle_instruction(command.clone()),
                Action::Operation(_) => self.handle_operation(command.clone()),
            }
        }
    }


    fn handle_instruction(&mut self, command: Command) {
        match command.action {
            Action::Instruction(Instruction::SaveBuffer) => { self.view.try_save_buffer() }
            Action::Instruction(Instruction::ExitEditor) => {
                if self.view.buffer_is_dirty() {
                    let args = BuilderArgs::new().with_str("Unsaved changes".into());
                    let _ = self.command_sender.send(Command::show_message(Some(args)));
                } else {
                    self.running = false;
                }

            }
            Action::Instruction(Instruction::SetMark(mark)) => {
                if let Some(object) = command.object {
                    self.view.move_mark(mark, object)
                }
            }
            Action::Instruction(Instruction::SetOverlay(overlay_type)) => {
                self.view.set_overlay(overlay_type)
            }
            Action::Instruction(Instruction::SetMode(mode)) => {
                match mode {
                    ModeType::Insert => { self.mode = Box::new(InsertMode::new()) }
                    ModeType::Normal => { self.mode = Box::new(NormalMode::new()) }
                }
            }
            Action::Instruction(Instruction::SwitchToLastBuffer) => {
                self.view.switch_last_buffer();
                self.view.clear(&mut self.rb);
            }
            Action::Instruction(Instruction::ShowMessage(msg)) => {
                self.view.show_message(msg)
            }

            _ => {}
        }
    }

    fn handle_operation(&mut self, command: Command) {
        match command.action {
            Action::Operation(Operation::Insert(c)) => {
                for _ in 0..command.number {
                    self.view.insert_char(c)
                }
            }
            Action::Operation(Operation::DeleteObject) => {
                if let Some(obj) = command.object {
                    self.view.delete_object(obj);
                }
            }
            Action::Operation(Operation::DeleteFromMark(m)) => {
                if command.object.is_some() {
                    self.view.delete_from_mark_to_object(m, command.object.unwrap())
                }
            }
            Action::Operation(Operation::Undo) => { self.view.undo() }
            Action::Operation(Operation::Redo) => { self.view.redo() }

            Action::Instruction(_) => {}
        }
    }

    /// Start Iota!
    pub fn start(&mut self) {
        while self.running {
            self.draw();
            self.rb.present();
            self.view.maybe_clear_message();

            match self.rb.poll_event(true) {
                Ok(Event::ResizeEvent(width, height)) => self.handle_resize_event(width as usize, height as usize),
                Ok(key_event) => self.handle_key_event(key_event),
                _ => {}
            }

            while let Ok(message) = self.command_queue.try_recv() {
                self.handle_command(message)
            }
        }
    }
}
