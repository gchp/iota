use keyboard::Key;
use keymap::{KeyMap, KeyMapState};
use command::{BuilderEvent, Command};
use textobject::{ Offset, Kind, Anchor };
use buffer::Mark;
use overlay::OverlayType;

use super::{Mode, ModeType};


/// `NormalMode` mimics Vi's Normal mode.
pub struct NormalMode {
    keymap: KeyMap,
    number: Option<i32>,
    reading_number: bool,
}

impl NormalMode {

    /// Create a new instance of `NormalMode`
    pub fn new() -> NormalMode {
        NormalMode {
            keymap: NormalMode::key_defaults(),
            number: None,
            reading_number: false,
        }
    }

    /// Creates a `KeyMap` with default `NormalMode` key bindings
    fn key_defaults() -> KeyMap {
        let mut keymap = KeyMap::new();
        // movement
        keymap.bind_key(Key::Char('h'), Command::movement(Offset::Backward(1, Mark::Cursor(0)), Kind::Char));
        keymap.bind_key(Key::Char('j'), Command::movement(Offset::Forward(1, Mark::Cursor(0)), Kind::Line(Anchor::Same)));
        keymap.bind_key(Key::Char('k'), Command::movement(Offset::Backward(1, Mark::Cursor(0)), Kind::Line(Anchor::Same)));
        keymap.bind_key(Key::Char('l'), Command::movement(Offset::Forward(1, Mark::Cursor(0)), Kind::Char));
        keymap.bind_key(Key::Left, Command::movement(Offset::Backward(1, Mark::Cursor(0)), Kind::Char));
        keymap.bind_key(Key::Down, Command::movement(Offset::Forward(1, Mark::Cursor(0)), Kind::Line(Anchor::Same)));
        keymap.bind_key(Key::Up, Command::movement(Offset::Backward(1, Mark::Cursor(0)), Kind::Line(Anchor::Same)));
        keymap.bind_key(Key::Right, Command::movement(Offset::Forward(1, Mark::Cursor(0)), Kind::Char));
        keymap.bind_key(Key::Char('w'), Command::movement(Offset::Forward(1, Mark::Cursor(0)), Kind::Word(Anchor::Start)));
        keymap.bind_key(Key::Char('b'), Command::movement(Offset::Backward(1, Mark::Cursor(0)), Kind::Word(Anchor::Start)));
        keymap.bind_key(Key::Char('$'), Command::movement(Offset::Forward(0, Mark::Cursor(0)), Kind::Line(Anchor::End)));
        keymap.bind_key(Key::Char('0'), Command::movement(Offset::Backward(0, Mark::Cursor(0)), Kind::Line(Anchor::Start)));

        // actions
        keymap.bind_key(Key::Char('u'), Command::undo());
        keymap.bind_key(Key::Ctrl('r'), Command::redo());
        keymap.bind_key(Key::Char('i'), Command::set_mode(ModeType::Insert));
        keymap.bind_key(Key::Char(':'), Command::set_overlay(OverlayType::CommandPrompt));

        keymap
    }

}

impl Mode for NormalMode {
    fn handle_key_event(&mut self, key: Key) -> BuilderEvent {
        if let Key::Char(c) = key {
            // '0' might be bound (start of line), and cannot be the start of a number sequence
            if c.is_digit(10) && (self.reading_number || c != '0') {
                let n = c.to_digit(10).unwrap() as i32;
                self.reading_number = true;
                if let Some(current) = self.number {
                    self.number = Some((current*10) + n);
                } else {
                    self.number = Some(n);
                }
                return BuilderEvent::Incomplete;
            } else if self.reading_number {
                self.reading_number = false;
            }
        }
        match self.keymap.check_key(key) {
            KeyMapState::Match(mut c) => {
                if let Some(num) = self.number {
                    c.number = num;
                }
                self.number = None;
                BuilderEvent::Complete(c)
            }
            _ => {
                BuilderEvent::Incomplete
            }
        }
    }
}

impl Default for NormalMode {
    fn default() -> Self {
        Self::new()
    }
}
