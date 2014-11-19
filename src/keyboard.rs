use std::num::FromPrimitive;

pub enum Key {
    Enter = 13,
    Down  = 1073741905,
}

impl FromPrimitive for Key {
    fn from_u64(n: u64) -> Option<Key> {
        match n {
            13         => Some(Key::Enter),
            1073741905 => Some(Key::Down),
            _          => None
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
}
