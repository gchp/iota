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
            65522 => Some(Key::Delete),
            _     => None,
        }
    }
}
