use std::num::FromPrimitive;
use std::char;

pub struct Key {
    code: u64
}

pub const TAB: Key       = Key { code: 9 };
pub const ENTER: Key     = Key { code: 13 };
pub const CTRL_Q: Key     = Key { code: 17 };
pub const CTRL_R: Key     = Key { code: 18 };
pub const CTRL_S: Key     = Key { code: 19 };
#[allow(dead_code)]
pub const ESC: Key       = Key { code: 27 };
pub const BACKSPACE: Key = Key { code: 127 };
pub const RIGHT: Key     = Key { code: 65514 };
pub const LEFT: Key      = Key { code: 65515 };
pub const DOWN: Key      = Key { code: 65516 };
pub const UP: Key        = Key { code: 65517 };
pub const DELETE: Key    = Key { code: 65522 };

impl Key {
    #[inline(always)]
    pub fn code(&self) -> u32 {
        self.code as u32
    }

    pub fn get_char(&self) -> Option<char> {
        char::from_u32(self.code())
    }
}

impl FromPrimitive for Key {
    fn from_u64(n: u64) -> Option<Key> {
        Some(Key { code: n })
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

    #[inline(always)]
    fn from_u32(n: u32) -> Option<Key> {
        FromPrimitive::from_u64(n as u64)
    }
}
