use buffer::Mark;
use command::{BuilderArgs, BuilderEvent};
use keyboard::Key;
use keymap::{CommandInfo, KeyMap, KeyMapState};
use textobject::{Anchor, Kind, Offset};

use super::Mode;

/// Standard mode is Iota's default mode.
///
/// Standard mode uses non-vi-like keybindings.
/// Unlike Normal, Command and Visual modes which are all used together, Standard
/// mode is used on its own.
///
/// Standard mode allows Iota to be used in a non-modal way, similar to mainstream
/// editors like Atom or Sublime.
pub struct StandardMode {
    keymap: KeyMap,
    match_in_progress: bool,
}

impl StandardMode {
    /// Create a new instance of StandardMode
    pub fn new() -> StandardMode {
        StandardMode {
            keymap: StandardMode::key_defaults(),
            match_in_progress: false,
        }
    }

    /// Creates a KeyMap with default StandardMode key bindings
    fn key_defaults() -> KeyMap {
        let mut keymap = KeyMap::new();

        // Editor Commands
        keymap.bind_key(
            Key::Ctrl('q'),
            CommandInfo {
                command_name: String::from("editor::quit"),
                args: None,
            },
        );
        keymap.bind_key(
            Key::Ctrl('s'),
            CommandInfo {
                command_name: String::from("editor::save_buffer"),
                args: None,
            },
        );

        // Cursor movement
        keymap.bind_key(
            Key::Up,
            CommandInfo {
                command_name: String::from("buffer::move_cursor"),
                args: Some(
                    BuilderArgs::new()
                        .with_kind(Kind::Line(Anchor::Same))
                        .with_offset(Offset::Backward(1, Mark::Cursor(0))),
                ),
            },
        );
        keymap.bind_key(
            Key::Down,
            CommandInfo {
                command_name: String::from("buffer::move_cursor"),
                args: Some(
                    BuilderArgs::new()
                        .with_kind(Kind::Line(Anchor::Same))
                        .with_offset(Offset::Forward(1, Mark::Cursor(0))),
                ),
            },
        );
        keymap.bind_key(
            Key::Left,
            CommandInfo {
                command_name: String::from("buffer::move_cursor"),
                args: Some(
                    BuilderArgs::new()
                        .with_kind(Kind::Char)
                        .with_offset(Offset::Backward(1, Mark::Cursor(0))),
                ),
            },
        );
        keymap.bind_key(
            Key::Right,
            CommandInfo {
                command_name: String::from("buffer::move_cursor"),
                args: Some(
                    BuilderArgs::new()
                        .with_kind(Kind::Char)
                        .with_offset(Offset::Forward(1, Mark::Cursor(0))),
                ),
            },
        );

        keymap.bind_key(
            Key::CtrlRight,
            CommandInfo {
                command_name: String::from("buffer::move_cursor"),
                args: Some(
                    BuilderArgs::new()
                        .with_kind(Kind::Word(Anchor::Start))
                        .with_offset(Offset::Forward(1, Mark::Cursor(0))),
                ),
            },
        );
        keymap.bind_key(
            Key::CtrlLeft,
            CommandInfo {
                command_name: String::from("buffer::move_cursor"),
                args: Some(
                    BuilderArgs::new()
                        .with_kind(Kind::Word(Anchor::Start))
                        .with_offset(Offset::Backward(1, Mark::Cursor(0))),
                ),
            },
        );

        keymap.bind_key(
            Key::End,
            CommandInfo {
                command_name: String::from("buffer::move_cursor"),
                args: Some(
                    BuilderArgs::new()
                        .with_kind(Kind::Line(Anchor::End))
                        .with_offset(Offset::Forward(0, Mark::Cursor(0))),
                ),
            },
        );
        keymap.bind_key(
            Key::Home,
            CommandInfo {
                command_name: String::from("buffer::move_cursor"),
                args: Some(
                    BuilderArgs::new()
                        .with_kind(Kind::Line(Anchor::End))
                        .with_offset(Offset::Backward(0, Mark::Cursor(0))),
                ),
            },
        );

        // Editing
        keymap.bind_key(
            Key::Tab,
            CommandInfo {
                command_name: String::from("buffer::insert_tab"),
                args: None,
            },
        );
        keymap.bind_key(
            Key::Enter,
            CommandInfo {
                command_name: String::from("buffer::insert_char"),
                args: Some(BuilderArgs::new().with_char_arg('\n')),
            },
        );
        keymap.bind_key(
            Key::Backspace,
            CommandInfo {
                command_name: String::from("buffer::delete_char"),
                args: Some(
                    BuilderArgs::new()
                        .with_kind(Kind::Char)
                        .with_offset(Offset::Backward(1, Mark::Cursor(0))),
                ),
            },
        );
        keymap.bind_key(
            Key::Backspace,
            CommandInfo {
                command_name: String::from("buffer::delete_char"),
                args: Some(
                    BuilderArgs::new()
                        .with_kind(Kind::Char)
                        .with_offset(Offset::Forward(1, Mark::Cursor(0))),
                ),
            },
        );

        // History
        keymap.bind_key(
            Key::Ctrl('z'),
            CommandInfo {
                command_name: String::from("editor::undo"),
                args: None,
            },
        );
        keymap.bind_key(
            Key::Ctrl('r'),
            CommandInfo {
                command_name: String::from("editor::redo"),
                args: None,
            },
        );

        keymap
    }

    /// Checks a Key against the internal keymap
    ///
    /// - If there is a direct match, return the completed BuilderEvent
    /// - If there is a partial match, set match_in_progress to true which
    ///   indicates that the next key should check against the keymap too,
    ///   rather than possibly being inserted into the buffer. This allows
    ///   for non-prefixed keys to be used in keybindings. ie: C-x s rather
    ///   than C-x C-s.
    /// - If there is no match of any kind, return Incomplete
    fn check_key(&mut self, key: Key) -> BuilderEvent {
        match self.keymap.check_key(key) {
            KeyMapState::Match(c) => {
                self.match_in_progress = false;
                BuilderEvent::Complete(c)
            }
            KeyMapState::Continue => {
                self.match_in_progress = true;
                BuilderEvent::Incomplete
            }
            KeyMapState::None => {
                self.match_in_progress = false;
                BuilderEvent::Incomplete
            }
        }
    }
}

impl Mode for StandardMode {
    /// Given a key, pass it through the StandardMode KeyMap and return the associated Command, if any.
    /// If no match is found, treat it as an InsertChar command.
    fn handle_key_event(&mut self, key: Key) -> BuilderEvent {
        if self.match_in_progress {
            return self.check_key(key);
        }

        if let Key::Char(c) = key {
            let command_info = CommandInfo {
                command_name: String::from("buffer::insert_char"),
                args: Some(BuilderArgs::new().with_char_arg(c)),
            };
            BuilderEvent::Complete(command_info)
        } else {
            self.check_key(key)
        }
    }
}

impl Default for StandardMode {
    fn default() -> Self {
        Self::new()
    }
}
