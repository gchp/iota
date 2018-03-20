use keyboard::Key;
use keymap::{KeyMap, KeyMapState};
use command::{BuilderEvent, BuilderArgs };

use super::Mode;


/// `InsertMode` mimics Vi's Insert mode.
pub struct InsertMode {
    keymap: KeyMap,
}

impl InsertMode {

    /// Create a new instance of `InsertMode`
    pub fn new() -> InsertMode {
        InsertMode {
            keymap: InsertMode::key_defaults(),
        }
    }

    /// Creates a `KeyMap` with default `InsertMode` key bindings
    fn key_defaults() -> KeyMap {
        let mut keymap = KeyMap::new();

        keymap.bind_key(Key::Esc, "editor::set_mode_normal".into());

        keymap
    }

}

impl Mode for InsertMode {
    fn handle_key_event(&mut self, key: Key) -> BuilderEvent {
        if let Key::Char(c) = key {
            let builder_args = BuilderArgs::new().with_char_arg(c);
            BuilderEvent::Complete("buffer::insert_char".into(), Some(builder_args))
        } else if let KeyMapState::Match(c) = self.keymap.check_key(key) {
            BuilderEvent::Complete(c, None)
        } else {
            BuilderEvent::Incomplete
        }
    }
}
