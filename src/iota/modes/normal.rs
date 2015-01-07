use super::Mode;
use super::KeyMap;
use super::Key;
use super::Command;
use super::KeyMapState;
use super::Direction;
use super::WordEdgeMatch;
use super::{Overlay, OverlayType, OverlayEvent};


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
        keymap.bind_key(Key::Char('W'), Command::MoveCursor(Direction::RightWord(1, WordEdgeMatch::Whitespace)));
        keymap.bind_key(Key::Char('B'), Command::MoveCursor(Direction::LeftWord(1, WordEdgeMatch::Whitespace)));
        keymap.bind_key(Key::Char('w'), Command::MoveCursor(Direction::RightWord(1, WordEdgeMatch::Alphabet)));
        keymap.bind_key(Key::Char('b'), Command::MoveCursor(Direction::LeftWord(1, WordEdgeMatch::Alphabet)));
        keymap.bind_key(Key::Char('0'), Command::LineStart);
        keymap.bind_key(Key::Char('$'), Command::LineEnd);

        // editing
        keymap.bind_keys(&[Key::Char('d'), Key::Char('W')], Command::Delete(Direction::RightWord(1, WordEdgeMatch::Whitespace)));
        keymap.bind_keys(&[Key::Char('d'), Key::Char('B')], Command::Delete(Direction::LeftWord(1, WordEdgeMatch::Whitespace)));
        keymap.bind_keys(&[Key::Char('d'), Key::Char('w')], Command::Delete(Direction::RightWord(1, WordEdgeMatch::Alphabet)));
        keymap.bind_keys(&[Key::Char('d'), Key::Char('b')], Command::Delete(Direction::LeftWord(1, WordEdgeMatch::Alphabet)));
        keymap.bind_key(Key::Char('x'), Command::Delete(Direction::Right(1)));
        keymap.bind_key(Key::Char('X'), Command::Delete(Direction::Left(1)));
        keymap.bind_key(Key::Char('u'), Command::Undo);
        keymap.bind_key(Key::Ctrl('r'), Command::Redo);

        // open a prompt to the user
        keymap.bind_key(Key::Char(':'), Command::SetOverlay(OverlayType::Prompt));

        keymap
    }

}

impl Mode for NormalMode {
    fn handle_key_event(&mut self, key: Option<Key>) -> Command {
        let key = match key {
            Some(k) => k,
            None => return Command::Unknown
        };

        if let KeyMapState::Match(command) = self.keymap.check_key(key) {
            return command
        }

        Command::Unknown
    }
}
