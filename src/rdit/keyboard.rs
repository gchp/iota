use std::num::FromPrimitive;

pub enum Key {
    Unknown   = 0,
    Enter     = 13,
    CtrlQ    = 17,
    CtrlS     = 19,
    Esc       = 27,
    Space     = 32,
    Backspace = 127,
    Right     = 65514,
    Left      = 65515,
    Down      = 65516,
    Up        = 65517,
    Delete    = 65522,
}

impl FromPrimitive for Key {
    fn from_u64(n: u64) -> Option<Key> {
        match n {
            0     => Some(Key::Unknown),
            13    => Some(Key::Enter),
            17    => Some(Key::CtrlQ),
            19    => Some(Key::CtrlS),
            27    => Some(Key::Esc),
            32    => Some(Key::Space),
            127   => Some(Key::Backspace),
            65514 => Some(Key::Right),
            65515 => Some(Key::Left),
            65516 => Some(Key::Down),
            65517 => Some(Key::Up),
            65522 => Some(Key::Delete),
            _     => None
        }
    }

    #[inline(always)]
    fn from_i64(n: i64) -> Option<Key> {
        FromPrimitive::from_u64(n as u64)
    }

    #[inline(always)]
    fn from_int(n: int) -> Option<Key> {
        FromPrimitive::from_u64(n as u64)
    }

    #[inline(always)]
    fn from_u16(n: u16) -> Option<Key> {
        FromPrimitive::from_u64(n as u64)
    }
}
