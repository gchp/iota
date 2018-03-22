use keyboard::Key;
use keymap::{KeyMap, KeyBinding, KeyMapState, CommandInfo};
use command::{BuilderEvent, BuilderArgs };

use super::{ModeType, Mode};


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

        keymap.bind_key(
            Key::Esc,
            CommandInfo {
                command_name: String::from("editor::set_mode"),
                args: Some(BuilderArgs::new().with_mode(ModeType::Normal))
            }
        );


        keymap
    }

}

impl Mode for InsertMode {
    fn handle_key_event(&mut self, key: Key) -> BuilderEvent {
        if let Key::Char(c) = key {
            let builder_args = BuilderArgs::new().with_char_arg(c);
            let command_info = CommandInfo {
                command_name: String::from("buffer::insert_char"),
                args: Some(builder_args),
            };
            BuilderEvent::Complete(command_info)
        } else if let KeyMapState::Match(c) = self.keymap.check_key(key) {
            BuilderEvent::Complete(c)
        } else {
            BuilderEvent::Incomplete
        }
    }
}
