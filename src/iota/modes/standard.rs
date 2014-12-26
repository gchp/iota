use super::Mode;
use super::KeyMap;
use super::Key;
use super::Command;
use super::View;
use super::KeyMapState;
use super::LogEntries;
use super::EventStatus;
use super::Direction;
use super::Response;


pub struct StandardMode {
    keymap: KeyMap,
}

impl StandardMode {

    fn new() -> StandardMode {
        StandardMode {
            keymap: StandardMode::key_defaults(),
        }
    }

    fn key_defaults() -> KeyMap {
        let mut keymap = KeyMap::new();

        // navigation
        keymap.bind_key(Key::Up, Command::MoveCursor(Direction::Up));
        keymap.bind_key(Key::Down, Command::MoveCursor(Direction::Down));
        keymap.bind_key(Key::Left, Command::MoveCursor(Direction::Left));
        keymap.bind_key(Key::Right, Command::MoveCursor(Direction::Right));

        keymap
    }

    fn handle_command(&mut self, c: Command, view: &mut View) -> Response {
        match c {
            Command::MoveCursor(dir) => view.move_cursor(dir),
            _                        => {},
        }
        Response::Continue
    }
}

impl Mode for StandardMode {
    fn handle_key_event(&mut self, key: Option<Key>, view: &mut View, log: &mut LogEntries) -> EventStatus {
        let key = match key {
            Some(k) => k,
            None => return EventStatus::NotHandled
        };

        // send key to the keymap
        match self.keymap.check_key(key) {
            KeyMapState::Match(command) => {
                return EventStatus::Handled(self.handle_command(command, view));
            },
            KeyMapState::Continue => {
                // keep going and wait for more keypresses
                return EventStatus::Handled(Response::Continue)
            },
            KeyMapState::None => {}  // do nothing and handle the key normally
        }

        // if the key is a character that is not part of a keybinding, insert into the buffer
        // otherwise, ignore it.
        if let Key::Char(c) = key {
            let mut transaction = log.start(view.cursor_data);
            view.insert_char(&mut transaction, c);
            EventStatus::Handled(Response::Continue)
        } else {
            EventStatus::NotHandled
        }

    }

}
