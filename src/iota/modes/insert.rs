use keyboard::Key;
use keymap::{KeyMap, KeyMapState, CommandInfo};
use command::{BuilderEvent, BuilderArgs };
use textobject::{ Offset, Kind, Anchor };
use buffer::Mark;

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
        // Cursor movement
        keymap.bind_key(
            Key::Up,
            CommandInfo {
                command_name: String::from("buffer::move_cursor"),
                args: Some(BuilderArgs::new().with_kind(Kind::Line(Anchor::Same))
                                             .with_offset(Offset::Backward(1, Mark::Cursor(0))))
            }
        );
        keymap.bind_key(
            Key::Down,
            CommandInfo {
                command_name: String::from("buffer::move_cursor"),
                args: Some(BuilderArgs::new().with_kind(Kind::Line(Anchor::Same))
                                             .with_offset(Offset::Forward(1, Mark::Cursor(0))))
            }
        );
        keymap.bind_key(
            Key::Left,
            CommandInfo {
                command_name: String::from("buffer::move_cursor"),
                args: Some(BuilderArgs::new().with_kind(Kind::Char)
                                             .with_offset(Offset::Backward(1, Mark::Cursor(0))))
            }
        );
        keymap.bind_key(
            Key::Right,
            CommandInfo {
                command_name: String::from("buffer::move_cursor"),
                args: Some(BuilderArgs::new().with_kind(Kind::Char)
                                             .with_offset(Offset::Forward(1, Mark::Cursor(0))))
            }
        );
        keymap.bind_key(
            Key::Tab,
            CommandInfo {
                command_name: String::from("buffer::insert_tab"),
                args: None,
            }
        );
        keymap.bind_key(
            Key::Enter,
            CommandInfo {
                command_name: String::from("buffer::insert_char"),
                args: Some(BuilderArgs::new().with_char_arg('\n')),
            }
        );
        keymap.bind_key(
            Key::Backspace,
            CommandInfo {
                command_name: String::from("buffer::delete_char"),
                args: Some(BuilderArgs::new().with_kind(Kind::Char)
                                             .with_offset(Offset::Backward(1, Mark::Cursor(0))))
            }
        );
        keymap.bind_key(
            Key::Backspace,
            CommandInfo {
                command_name: String::from("buffer::delete_char"),
                args: Some(BuilderArgs::new().with_kind(Kind::Char)
                                             .with_offset(Offset::Forward(1, Mark::Cursor(0))))
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
