use keyboard::Key;
use keymap::{KeyMap, KeyMapState};
use command::{BuilderEvent, Command};

use super::{Mode, ModeType};


/// `InsertMode` mimics Vi's Insert mode.
pub struct InsertMode {
    keymap: KeyMap<Command>,
}

impl InsertMode {

    /// Create a new instance of `InsertMode`
    pub fn new() -> InsertMode {
        InsertMode {
            keymap: InsertMode::key_defaults(),
        }
    }

    /// Creates a `KeyMap` with default `InsertMode` key bindings
    fn key_defaults() -> KeyMap<Command> {
        let mut keymap = KeyMap::new();

        keymap.bind_key(Key::Esc, Command::set_mode(ModeType::Normal));

        keymap
    }

}

impl Mode for InsertMode {
    fn handle_key_event(&mut self, key: Key) -> BuilderEvent {
        if let Key::Char(c) = key {
            BuilderEvent::Complete(Command::insert_char(c))
        } else {
            if let KeyMapState::Match(c) = self.keymap.check_key(key) {
                BuilderEvent::Complete(c)
            } else {
                BuilderEvent::Incomplete
            }
        }
    }
}
