use std::path::PathBuf;
use std::sync::{Mutex, Arc};
use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc::channel;

use input::Input;
use keyboard::Key;
use view::View;
use frontends::{Frontend, EditorEvent};
use modes::{Mode, ModeType, InsertMode, NormalMode};
use overlay::{Overlay, OverlayEvent};
use buffer::Buffer;
use command::Command;
use command::{Action, BuilderEvent, Operation, Instruction};
use chained_command::ChainedCmdBuilder;

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

        let mut remove_overlay = false;
        let mut command = BuilderEvent::Incomplete;

        match self.view.overlay {
            Overlay::None => { command = self.mode.handle_key_event(key) },
            _             => {
                let event = self.view.overlay.handle_key_event(key);
                match event {
                    OverlayEvent::Finished(response) => {
                        remove_overlay = true;
                        let commands = self.handle_overlay_response(response);
                        for cmd in commands {
                            if let BuilderEvent::Complete(c) = cmd {
                                self.handle_command(c);
                            }
                        }
                    }

                    _ => { command = BuilderEvent::Incomplete }
                }
            }
        }

        if remove_overlay {
            self.view.overlay = Overlay::None;
            self.view.clear(&mut self.frontend);
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
    fn handle_overlay_response(&mut self, response: Option<String>) -> Vec<BuilderEvent> {
        let mut commands = vec![];

        match response {
            Some(data) => {
                match self.view.overlay {

                    Overlay::Prompt { ref data, .. } => {
                        let mut builder = ChainedCmdBuilder::new();
                        let cmds: Vec<Command> = builder.parse(&**data);

                        if cmds.len() < 1 {
                            commands.push(BuilderEvent::Incomplete);
                        } else {
                            for cmd in cmds {
                                commands.push(BuilderEvent::Complete(cmd));
                            }
                        }
                    }

                    Overlay::SavePrompt { .. } => {
                        if data.is_empty() {
                            commands.push(BuilderEvent::Invalid);
                        } else {
                            let path = PathBuf::from(&*data);
                            self.view.buffer.lock().unwrap().file_path = Some(path);
                            commands.push(BuilderEvent::Complete(Command::save_buffer()));
                        }
                    }

                    Overlay::SelectFile { .. } => {
                        let path = PathBuf::from(data);
                        let buffer = Arc::new(Mutex::new(Buffer::from(path)));
                        self.buffers.push(buffer.clone());
                        self.view.set_buffer(buffer.clone());
                        self.view.clear(&mut self.frontend);
                        commands.push(BuilderEvent::Complete(Command::noop()));
                    }

                    _ => commands.push(BuilderEvent::Incomplete),
                }
            }
            None => commands.push(BuilderEvent::Incomplete)
        }

        commands
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

            while let Ok(message) = self.command_queue.try_recv() {
                self.handle_command(message)
            }

            let event = self.frontend.poll_event();

            match event {
                Some(EditorEvent::KeyEvent(key))         => self.handle_key_event(key),
                Some(EditorEvent::Resize(width, height)) => self.handle_resize_event(width, height),

                _ => {}
            }
        }
    }
}
