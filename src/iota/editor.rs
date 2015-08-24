use std::path::PathBuf;
use std::sync::{Mutex, Arc};
use std::collections::HashMap;

use input::Input;
use keyboard::Key;
use view::View;
use frontends::{Frontend, EditorEvent};
use modes::{Mode, ModeType, InsertMode, NormalMode};
use overlay::{Overlay, OverlayEvent};
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
    events: HashMap<&'static str, Box<Fn(&mut View) -> ()>>,
    events_queue: Vec<&'static str>,
}

impl<'e, T: Frontend> Editor<'e, T> {
    /// Create a new Editor instance from the given source
    pub fn new(source: Input, mode: Box<Mode + 'e>, frontend: T) -> Editor<'e, T> {
        let height = frontend.get_window_height();
        let width = frontend.get_window_width();

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
            events: HashMap::new(),
            events_queue: Vec::new(),
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

        // TODO: send key to plugins for resolution
        // let mut event = self.plugin_resolve(key);
        let mut event = None;

        // if None, use core resolution
        if event.is_none() {
            event = match key {
                Key::Down => Some("iota.move_down"),
                Key::Ctrl('q') => Some("iota.quit"),
                _ => None,
            }
        }

        // if we have an event, fire it
        // otherwise just move on
        if let Some(e) = event {
            self.fire_event(e);
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
                        let path = PathBuf::from(&*data);
                        self.view.buffer.lock().unwrap().file_path = Some(path);
                        BuilderEvent::Complete(Command::save_buffer())
                    }

                    Overlay::SelectFile { .. } => {
                        let path = PathBuf::from(data);
                        let buffer = Arc::new(Mutex::new(Buffer::from(path)));
                        self.buffers.push(buffer.clone());
                        self.view.set_buffer(buffer.clone());
                        self.view.clear(&mut self.frontend);
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
            Instruction::SwitchToLastBuffer => {
                self.view.switch_last_buffer();
                self.view.clear(&mut self.frontend);
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

    fn register_event_listeners(&mut self) {
        // TODO: register the event & handler

        fn some_handler(e: &mut View) {
            panic!("from the handler")
        }

        self.register_handler("iota.move_down", Box::new(some_handler));
    }

    fn register_handler(&mut self, event: &'static str, handler: Box<Fn(&mut View) -> ()>) {
        self.events.insert(event, handler);
    }

    fn fire_event(&mut self, event: &'static str) {
        self.events_queue.push(event);
    }

    fn process_event(&mut self, event: &'static str) {
        // TODO: look up the events register to see if there is a callback
        //       registered for this event
        if self.events.contains_key(event) {
            let callback = self.events.get(event).unwrap();
            callback(&mut self.view);
        }
    }

    /// Start Iota!
    pub fn start(&mut self) {
        self.register_event_listeners();

        while self.running {
            self.draw();
            self.frontend.present();
            let event = self.frontend.poll_event();

            match event {
                EditorEvent::KeyEvent(key)         => self.handle_key_event(key),
                EditorEvent::Resize(width, height) => self.handle_resize_event(width, height),

                _ => {}
            }

            // FIXME: is there a way to not use clone here?
            let events = self.events_queue.clone();
            for event in events.iter() {
                self.process_event(event)
            }
        }
    }
}
