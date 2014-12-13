#[deriving(Copy)]
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

    Char(char),
    Ctrl(char),
}

impl Key {
    pub fn from_special_code(code: u16) -> Option<Key> {
        match code {
            9     => Some(Key::Tab),
            13    => Some(Key::Enter),
            14    => Some(Key::Ctrl('n')),
            16    => Some(Key::Ctrl('p')),
            17    => Some(Key::Ctrl('q')),
            18    => Some(Key::Ctrl('r')),
            19    => Some(Key::Ctrl('s')),
            24    => Some(Key::Ctrl('x')),
            27    => Some(Key::Esc),
            32    => Some(Key::Char(' ')),
            127   => Some(Key::Backspace),
            65514 => Some(Key::Right),
            65515 => Some(Key::Left),
            65516 => Some(Key::Down),
            65517 => Some(Key::Up),
            65522 => Some(Key::Delete),
            _     => None,
        }
    }
}
