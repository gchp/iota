use super::Mode;
use super::KeyMap;
use super::Key;
use super::KeyMapState;
use super::Direction;

use command::{BuilderEvent, Operation, Command, Action};
use textobject::{Kind, TextObject, Offset};


/// Standard mode is Iota's default mode.
///
/// Standard mode uses non-vi-like keybindings.
/// Unlike Normal, Command and Visual modes which are all used together, Standard
/// mode is used on its own.
///
/// Standard mode allows Iota to be used in a non-modal way, similar to mainstream
/// editors like emacs or sublime.
pub struct StandardMode {
    keymap: KeyMap<Command>,
}

impl StandardMode {

    /// Create a new instance of StandardMode
    pub fn new() -> StandardMode {
        StandardMode {
            keymap: StandardMode::key_defaults(),
        }
    }

    /// Creates a KeyMap with default StandardMode key bindings
    fn key_defaults() -> KeyMap<Command> {
        let mut keymap = KeyMap::new();

        // Editor Commands
        keymap.bind_key(Key::Ctrl('q'), Command::exit_editor());
        keymap.bind_key(Key::Ctrl('s'), Command::save_buffer());
        keymap.bind_keys(vec![Key::Ctrl('x'), Key::Ctrl('c')].as_slice(), Command::exit_editor());
        keymap.bind_keys(vec![Key::Ctrl('x'), Key::Ctrl('s')].as_slice(), Command::save_buffer());

        // // Navigation
        // keymap.bind_key(Key::Up, Command::MoveCursor(Direction::Up, 1));
        // keymap.bind_key(Key::Down, Command::MoveCursor(Direction::Down, 1));
        // keymap.bind_key(Key::Left, Command::MoveCursor(Direction::Left, 1));
        // keymap.bind_key(Key::Right, Command::MoveCursor(Direction::Right, 1));

        // keymap.bind_key(Key::Ctrl('p'), Command::MoveCursor(Direction::Up, 1));
        // keymap.bind_key(Key::Ctrl('n'), Command::MoveCursor(Direction::Down, 1));
        // keymap.bind_key(Key::Ctrl('b'), Command::MoveCursor(Direction::Left, 1));
        // keymap.bind_key(Key::Ctrl('f'), Command::MoveCursor(Direction::Right, 1));
        // keymap.bind_key(Key::Ctrl('e'), Command::LineEnd);
        // keymap.bind_key(Key::Ctrl('a'), Command::LineStart);

        // // Editing
        // keymap.bind_key(Key::Tab, Command::InsertTab);
        // keymap.bind_key(Key::Enter, Command::InsertChar('\n'));
        // keymap.bind_key(Key::Backspace, Command::Delete(Direction::Left, 1));
        // keymap.bind_key(Key::Ctrl('h'), Command::Delete(Direction::Left, 1));
        // keymap.bind_key(Key::Delete, Command::Delete(Direction::Right, 1));
        // keymap.bind_key(Key::Ctrl('d'), Command::Delete(Direction::Right, 1));

        // // History
        // keymap.bind_key(Key::Ctrl('y'), Command::Redo);
        // keymap.bind_key(Key::Ctrl('z'), Command::Undo);

        keymap
    }

}

impl Mode for StandardMode {
    /// Given a key, pass it through the StandardMode KeyMap and return the associated Command, if any.
    /// If no match is found, treat it as an InsertChar command.
    fn handle_key_event(&mut self, key: Key) -> BuilderEvent {

        if let Key::Char(c) = key {
            BuilderEvent::Complete(Command {
                number: 1,
                action: Action::Operation(Operation::Insert(c)),
                object: TextObject {
                    kind: Kind::Char,
                    offset: Offset::Absolute(0)
                }
            })
        } else {
            if let KeyMapState::Match(c) = self.keymap.check_key(key) {
                BuilderEvent::Complete(c)
            } else {
                BuilderEvent::Incomplete
            }
        }
    }
}
