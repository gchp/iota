use super::Mode;
use super::KeyMap;
use super::Key;
use super::Command;
use super::KeyMapState;
use super::Direction;
use super::WordEdgeMatch;
use super::OverlayType;

use command;


/// NormalMode mimics Vi's Normal mode.
pub struct NormalMode {
    keymap: KeyMap<command::Command>,
    builder: command::Builder,
}

impl NormalMode {

    /// Create a new instance of NormalMode
    pub fn new() -> NormalMode {
        NormalMode {
            keymap: NormalMode::key_defaults(),
            builder: command::Builder::new(),
        }
    }

    /// Creates a KeyMap with default NormalMode key bindings
    fn key_defaults() -> KeyMap<command::Command> {
        let mut keymap = KeyMap::new();

        // movement
        // keymap.bind_key(Key::Char('W'), Command::MoveCursor(Direction::RightWord(WordEdgeMatch::Whitespace), 1));
        // keymap.bind_key(Key::Char('B'), Command::MoveCursor(Direction::LeftWord(WordEdgeMatch::Whitespace), 1));
        // keymap.bind_key(Key::Char('w'), Command::MoveCursor(Direction::RightWord(WordEdgeMatch::Alphabet), 1));
        // keymap.bind_key(Key::Char('b'), Command::MoveCursor(Direction::LeftWord(WordEdgeMatch::Alphabet), 1));
        // keymap.bind_key(Key::Char('G'), Command::MoveCursor(Direction::LastLine, 0));
        // keymap.bind_keys(&[Key::Char('g'), Key::Char('g')], Command::MoveCursor(Direction::FirstLine, 0));

        // editing
        // keymap.bind_keys(&[Key::Char('d'), Key::Char('W')], Command::Delete(Direction::RightWord(WordEdgeMatch::Whitespace), 1));
        // keymap.bind_keys(&[Key::Char('d'), Key::Char('B')], Command::Delete(Direction::LeftWord(WordEdgeMatch::Whitespace), 1));
        // keymap.bind_keys(&[Key::Char('d'), Key::Char('w')], Command::Delete(Direction::RightWord(WordEdgeMatch::Alphabet), 1));
        // keymap.bind_keys(&[Key::Char('d'), Key::Char('b')], Command::Delete(Direction::LeftWord(WordEdgeMatch::Alphabet), 1));
        // keymap.bind_key(Key::Char('x'), Command::Delete(Direction::Right, 1));
        // keymap.bind_key(Key::Char('X'), Command::Delete(Direction::Left, 1));

        keymap.bind_key(Key::Char('u'), command::Command::undo());
        keymap.bind_key(Key::Ctrl('r'), command::Command::redo());

        keymap
    }

}

impl Mode for NormalMode {
    fn handle_key_event(&mut self, key: Key) -> command::BuilderEvent {
        match self.builder.check_key(key) {
            // builder gives us a full command, return that
            command::BuilderEvent::Complete(cmd) => command::BuilderEvent::Complete(cmd),

            // no command from the builder, check the internal keymap
            command::BuilderEvent::Incomplete => {
                if let KeyMapState::Match(c) = self.keymap.check_key(key) {
                    command::BuilderEvent::Complete(c)
                } else {
                    command::BuilderEvent::Incomplete
                }
            }

            // invalid result from builder, return invalid
            command::BuilderEvent::Invalid => { command::BuilderEvent::Invalid }
        }
    }
}
