use std::path::Path;

use input::Input;
use keyboard::Key;
use view::View;
use frontends::{Frontend, EditorEvent};
use modes::{Mode, ModeType, InsertMode, NormalMode};
use overlay::{Overlay, OverlayEvent};

use command::Command;
use command::{Action, BuilderEvent, Operation, Instruction};


/// The main Editor structure
///
/// This is the top-most structure in Iota.
pub struct Editor<'e, T: Frontend> {
    view: View<'e>,
    running: bool,
    frontend: T,
    mode: Box<Mode + 'e>,
}

impl<'e, T: Frontend> Editor<'e, T> {
    /// Create a new Editor instance
    pub fn new(source: Input, mode: Box<Mode + 'e>, frontend: T) -> Editor<'e, T> {
        let height = frontend.get_window_height();
        let width = frontend.get_window_width();
        let view = View::new(source, width, height);

        Editor {
            view: view,
            running: true,
            frontend: frontend,
            mode: mode,
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
            self.view.clear(&mut self.frontend);
        }

        if let BuilderEvent::Complete(c) = command {
            self.handle_command(c);
        }
    }

    /// Translate the response from an Overlay to a Command wrapped in a BuilderEvent
    ///
    /// In most cases, we will just want to convert the response directly to
    /// a Command, however in some cases we will want to perform other actions
    /// first, such as in the case of Overlay::SavePrompt.
    fn handle_overlay_response(&mut self, response: Option<String>) -> BuilderEvent {
        if let Some(response_data) = response {
            panic!(response_data);
        }
        BuilderEvent::Incomplete
        // FIXME: This entire method neext to be updated
        // match response {
        //     Some(data) => {
        //         match self.view.overlay {

        //             // FIXME: this is just a temporary fix
        //             Overlay::Prompt { ref data, .. } => {
        //                 match &**data {
        //                     // FIXME: need to find a better system for these commands
        //                     //        They should be chainable
        //                     //          ie: wq - save & quit
        //                     //        They should also take arguments
        //                     //          ie w file.txt - write buffer to file.txt
        //                     "q" | "quit" => BuilderEvent::Complete(Command::exit_editor()),
        //                     "w" | "write" => BuilderEvent::Complete(Command::save_buffer()),

        //                     _ => BuilderEvent::Incomplete
        //                 }
        //             }

        //             Overlay::SavePrompt { .. } => {
        //                 let path = Path::new(&*data);
        //                 self.view.buffer.file_path = Some(path);
        //                 BuilderEvent::Complete(Command::save_buffer())
        //             }
        //             _ => BuilderEvent::Incomplete,
        //         }
        //     }
        //     None => BuilderEvent::Incomplete
        // }
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
            Instruction::ExitEditor => { self.running = false; }
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
            let event = self.frontend.poll_event();

            match event {
                EditorEvent::KeyEvent(key)         => self.handle_key_event(key),
                EditorEvent::Resize(width, height) => self.handle_resize_event(width, height),

                _ => {}
            }
        }
    }
}
