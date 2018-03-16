use std::char;
use std::time::Duration;

use rustbox::{RustBox, Event};

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum Key {
    Tab,
    Enter,
    Esc,
    Backspace,
    Right,
    Left,
    Down,
    Up,
    Delete,
    Home,
    End,
    CtrlLeft,
    CtrlRight,
    CtrlUp,
    CtrlDown,

    Char(char),
    Ctrl(char),
}

impl Key {
    pub fn from_special_code(code: u16) -> Option<Key> {
        match code {
            1     => Some(Key::Ctrl('a')),
            2     => Some(Key::Ctrl('b')),
            3     => Some(Key::Ctrl('c')),
            4     => Some(Key::Ctrl('d')),
            5     => Some(Key::Ctrl('e')),
            6     => Some(Key::Ctrl('f')),
            7     => Some(Key::Ctrl('g')),
            8     => Some(Key::Ctrl('h')),
            9     => Some(Key::Tab),
            13    => Some(Key::Enter),
            14    => Some(Key::Ctrl('n')),
            16    => Some(Key::Ctrl('p')),
            17    => Some(Key::Ctrl('q')),
            18    => Some(Key::Ctrl('r')),
            19    => Some(Key::Ctrl('s')),
            20    => Some(Key::Ctrl('t')),
            21    => Some(Key::Ctrl('u')),
            22    => Some(Key::Ctrl('v')),
            23    => Some(Key::Ctrl('w')),
            24    => Some(Key::Ctrl('x')),
            25    => Some(Key::Ctrl('y')),
            26    => Some(Key::Ctrl('z')),
            27    => Some(Key::Esc),
            32    => Some(Key::Char(' ')),
            127   => Some(Key::Backspace),
            65514 => Some(Key::Right),
            65515 => Some(Key::Left),
            65516 => Some(Key::Down),
            65517 => Some(Key::Up),
            65520 => Some(Key::End),
            65521 => Some(Key::Home),
            65522 => Some(Key::Delete),
            _     => None,
        }
    }

    pub fn from_chord(rb: &mut RustBox, start: u16) -> Option<Key> {
        let chord = Key::get_chord(rb, start);

        match chord.as_str() {
            "\x1b[1;5A" => Some(Key::CtrlUp),
            "\x1b[1;5B" => Some(Key::CtrlDown),
            "\x1b[1;5C" => Some(Key::CtrlRight),
            "\x1b[1;5D" => Some(Key::CtrlLeft),
            _ => Key::from_special_code(start)
        }
    }

    pub fn get_chord(rb: &mut RustBox, start: u16) -> String {
            // Copy any data waiting to a string
            // There may be a cleaner way to do this?
            let mut chord = char::from_u32(u32::from(start)).unwrap().to_string();
            while let Ok(Event::KeyEventRaw(_, _, ch)) = rb.peek_event(Duration::from_secs(0), true) {
                chord.push(char::from_u32(ch).unwrap())
            }

            chord
    }
    
    pub fn from_event(rb: &mut RustBox, event: Event) -> Option<Key> {
        match event {
            Event::KeyEventRaw(_, k, ch) => {
                match k {
                    0 => char::from_u32(ch).map(Key::Char),
                    0x1b => Key::from_chord(rb, 0x1b),
                    a => Key::from_special_code(a)
                }
            },
            _ => None
        }
    }
}
