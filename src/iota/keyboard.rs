#[deriving(Copy)]
pub enum Key {
    Tab,
    Enter,
    CtrlQ,
    CtrlR,
    CtrlS,
    Esc,
    Backspace,
    Right,
    Left,
    Down,
    Up,
    Delete,

    Char(char),
}

impl Key {
    pub fn from_special_code(code: u16) -> Option<Key> {
        match code {
            9     => Some(Key::Tab),
            13    => Some(Key::Enter),
            17    => Some(Key::CtrlQ),
            18    => Some(Key::CtrlR),
            19    => Some(Key::CtrlS),
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
