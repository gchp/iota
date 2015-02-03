use input::Input;
use buffer::Direction;
use keyboard::Key;
use view::View;
use frontends::{Frontend, EditorEvent};
use modes::Mode;
use overlay::{Overlay, OverlayType, OverlayEvent};


#[derive(Copy, Debug, PartialEq, Eq)]
pub enum Command {
    SaveBuffer,
    ExitEditor,

    MoveCursor(Direction, usize),
    LineEnd,
    LineStart,

    Delete(Direction, usize),
    InsertTab,
    InsertChar(char),

    SetOverlay(OverlayType),

    Undo,
    Redo,

    Unknown,
    None,
}

impl Command {
    #[inline]
    pub fn from_str(string: &str) -> Command {
        match string {
            "q" | "quit"  => Command::ExitEditor,
            "w" | "write" => Command::SaveBuffer,

            _             => Command::Unknown,
        }
    }
}

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

                    _ => { Command:: None }
                }
            }
        };

        if remove_overlay {
            self.view.overlay = Overlay::None;
            self.view.clear(&mut self.frontend);
        }

        self.handle_command(command);
    }

    /// Translate the response from an Overlay to a Command
    ///
    /// In most cases, we will just want to convert the response directly to
    /// a Command, however in some cases we will want to perform other actions
    /// first, such as in the case of Overlay::SavePrompt.
    fn handle_overlay_response(&mut self, response: Option<String>) -> Command {
        match response {
            Some(data) => {
                match self.view.overlay {
                    Overlay::Prompt { .. } => Command::from_str(&*data),
                    Overlay::SavePrompt { .. } => {
                        let path = Path::new(&*data);
                        self.view.buffer.file_path = Some(path);
                        Command::SaveBuffer
                    }
                    _ => Command::None,
                }
            }
            None => Command::None
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
        // check for the ExitEditor command first
        if let Command::ExitEditor = command {
            self.running = false;
            return;
        }

        match command {
            // Editor Commands
            Command::SaveBuffer         => self.view.try_save_buffer(),
            Command::SetOverlay(o)      => self.view.set_overlay(o),

            // Navigation
            Command::MoveCursor(dir, n) => self.view.move_cursor(dir, n),
            Command::LineEnd            => self.view.move_cursor_to_line_end(),
            Command::LineStart          => self.view.move_cursor_to_line_start(),

            // Editing
            Command::Delete(dir, n)     => self.view.delete_chars(dir, n),
            Command::InsertTab          => self.view.insert_tab(),
            Command::InsertChar(c)      => self.view.insert_char(c),
            Command::Redo               => self.view.redo(),
            Command::Undo               => self.view.undo(),

            _ => {},
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
