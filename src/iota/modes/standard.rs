use keyboard::Key;
use keymap::{KeyMap, KeyMapState};
use buffer::Mark;
use command::{BuilderEvent, Operation, Command, Action};
use textobject::{Anchor, Kind, TextObject, Offset};

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
    keymap: KeyMap<Command>,
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
    fn key_defaults() -> KeyMap<Command> {
        let mut keymap = KeyMap::new();

        // Editor Commands
        keymap.bind_key(Key::Ctrl('q'), Command::exit_editor());
        keymap.bind_key(Key::Ctrl('s'), Command::save_buffer());

        // Cursor movement
        keymap.bind_key(Key::Up, Command::movement(Offset::Backward(1, Mark::Cursor(0)), Kind::Line(Anchor::Same)));
        keymap.bind_key(Key::Down, Command::movement(Offset::Forward(1, Mark::Cursor(0)), Kind::Line(Anchor::Same)));
        keymap.bind_key(Key::Left, Command::movement(Offset::Backward(1, Mark::Cursor(0)), Kind::Char));
        keymap.bind_key(Key::Right, Command::movement(Offset::Forward(1, Mark::Cursor(0)), Kind::Char));

        keymap.bind_key(Key::CtrlRight, Command::movement(Offset::Forward(1, Mark::Cursor(0)), Kind::Word(Anchor::Start)));
        keymap.bind_key(Key::CtrlLeft, Command::movement(Offset::Backward(1, Mark::Cursor(0)), Kind::Word(Anchor::Start)));

        keymap.bind_key(Key::End, Command::movement(Offset::Forward(0, Mark::Cursor(0)), Kind::Line(Anchor::End)));
        keymap.bind_key(Key::Home, Command::movement(Offset::Backward(0, Mark::Cursor(0)), Kind::Line(Anchor::Start)));

        // Editing
        keymap.bind_key(Key::Tab, Command::insert_tab());
        keymap.bind_key(Key::Enter, Command::insert_char('\n'));
        keymap.bind_key(Key::Backspace, Command {
            number: 1,
            action: Action::Operation(Operation::DeleteFromMark(Mark::Cursor(0))),
            object: Some(TextObject {
                kind: Kind::Char,
                offset: Offset::Backward(1, Mark::Cursor(0))
            })
        });
        keymap.bind_key(Key::Delete, Command {
            number: 1,
            action: Action::Operation(Operation::DeleteFromMark(Mark::Cursor(0))),
            object: Some(TextObject {
                kind: Kind::Char,
                offset: Offset::Forward(1, Mark::Cursor(0))
            })
        });
        keymap.bind_key(Key::Ctrl('h'), Command {
            number: 1,
            action: Action::Operation(Operation::DeleteFromMark(Mark::Cursor(0))),
            object: Some(TextObject {
                kind: Kind::Char,
                offset: Offset::Backward(1, Mark::Cursor(0))
            })
        });

        keymap.bind_key(Key::Ctrl('d'), Command::duplicate_selection());
        keymap.bind_key(Key::Ctrl('x'), Command::cut_selection());
        keymap.bind_key(Key::Ctrl('c'), Command::copy_selection());
        keymap.bind_key(Key::Ctrl('v'), Command::paste());
        keymap.bind_key(Key::CtrlUp, Command::move_selection(false));
        keymap.bind_key(Key::CtrlDown, Command::move_selection(true));

        // History
        keymap.bind_key(Key::Ctrl('z'), Command::undo());
        keymap.bind_key(Key::Ctrl('y'), Command::redo());

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
            },
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
            return self.check_key(key)
        }

        if let Key::Char(c) = key {
            BuilderEvent::Complete(Command::insert_char(c))
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
