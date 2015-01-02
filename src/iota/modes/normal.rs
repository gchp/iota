use super::Mode;
use super::KeyMap;
use super::Key;
use super::Command;
use super::View;
use super::KeyMapState;
use super::EventStatus;
use super::Direction;
use super::Response;
use super::utils;


pub struct NormalMode {
    keymap: KeyMap,
}

impl NormalMode {

    pub fn new() -> NormalMode {
        NormalMode {
            keymap: NormalMode::key_defaults(),
        }
    }

    fn key_defaults() -> KeyMap {
        let mut keymap = KeyMap::new();

        // movement
        keymap.bind_key(Key::Up, Command::MoveCursor(Direction::Up(1)));
        keymap.bind_key(Key::Down, Command::MoveCursor(Direction::Down(1)));
        keymap.bind_key(Key::Left, Command::MoveCursor(Direction::Left(1)));
        keymap.bind_key(Key::Right, Command::MoveCursor(Direction::Right(1)));
        keymap.bind_key(Key::Char('h'), Command::MoveCursor(Direction::Left(1)));
        keymap.bind_key(Key::Char('j'), Command::MoveCursor(Direction::Down(1)));
        keymap.bind_key(Key::Char('k'), Command::MoveCursor(Direction::Up(1)));
        keymap.bind_key(Key::Char('l'), Command::MoveCursor(Direction::Right(1)));
        keymap.bind_key(Key::Char('^'), Command::LineStart);
        keymap.bind_key(Key::Char('$'), Command::LineEnd);

        // editing
        keymap.bind_key(Key::Char('x'), Command::Delete(Direction::Right(1)));
        keymap.bind_key(Key::Char('u'), Command::Undo);
        keymap.bind_key(Key::Ctrl('r'), Command::Redo);

        // TODO: remove this - temporary workaround until overlays are done
        keymap.bind_key(Key::Ctrl('q'), Command::ExitEditor);

        keymap
    }

    fn handle_command(&mut self, c: Command, view: &mut View) -> Response {
        match c {
            // Editor Commands
            Command::ExitEditor      => return Response::Quit,
            Command::SaveBuffer      => utils::save_buffer(&view.buffer),

            // Navigation
            Command::MoveCursor(dir) => view.move_cursor(dir),
            Command::LineEnd         => view.move_cursor_to_line_end(),
            Command::LineStart       => view.move_cursor_to_line_start(),

            // Editing
            Command::Delete(dir)     => view.delete_char(dir),
            Command::Redo            => view.redo(),
            Command::Undo            => view.undo(),

            _                        => {}
        }
        Response::Continue
    }
}

impl Mode for NormalMode {
    fn handle_key_event(&mut self, key: Option<Key>, view: &mut View) -> EventStatus {
        let key = match key {
            Some(k) => k,
            None => return EventStatus::NotHandled
        };

        // send key to the keymap
        match self.keymap.check_key(key) {
            KeyMapState::Match(command) => {
                EventStatus::Handled(self.handle_command(command, view))
            },
            KeyMapState::Continue => {
                // keep going and wait for more keypresses
                EventStatus::Handled(Response::Continue)
            },
            KeyMapState::None => { EventStatus::NotHandled }
        }

    }

}
