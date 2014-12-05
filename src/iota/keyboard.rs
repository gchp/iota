use std::num::FromPrimitive;
use std::char;

pub enum Key {
    Unknown      = 0,
    Tab          = 9,
    Enter        = 13,
    CtrlQ        = 17,
    CtrlR        = 18,
    CtrlS        = 19,
    Esc          = 27,
    Space        = 32,
    Exclaim      = 33,
    Hash         = 35,
    Dollar       = 36,
    Percent      = 37,
    Ampersand    = 38,
    Quote        = 39,
    LeftParen    = 40,
    RightParen   = 41,
    Asterisk     = 42,
    Plus         = 43,
    Comma        = 44,
    Minus        = 45,
    Period       = 46,
    Slash        = 47,
    D0           = 48,
    D1           = 49,
    D2           = 50,
    D3           = 51,
    D4           = 52,
    D5           = 53,
    D6           = 54,
    D7           = 55,
    D8           = 56,
    D9           = 57,
    Colon        = 58,
    Semicolon    = 59,
    Less         = 60,
    Equals       = 61,
    Greater      = 62,
    Question     = 63,
    At           = 64,
    LeftBracket  = 91,
    Backslash    = 92,
    RightBracket = 93,
    Caret        = 94,
    Underscore   = 95,
    Backquote    = 96,
    A            = 97,
    B            = 98,
    C            = 99,
    D            = 100,
    E            = 101,
    F            = 102,
    G            = 103,
    H            = 104,
    I            = 105,
    J            = 106,
    K            = 107,
    L            = 108,
    M            = 109,
    N            = 110,
    O            = 111,
    P            = 112,
    Q            = 113,
    R            = 114,
    S            = 115,
    T            = 116,
    U            = 117,
    V            = 118,
    W            = 119,
    X            = 120,
    Y            = 121,
    Z            = 122,
    LeftBrace    = 123,
    Pipe         = 124,
    RightBrace   = 125,
    Tilde        = 126,
    Backspace    = 127,
    Right        = 65514,
    Left         = 65515,
    Down         = 65516,
    Up           = 65517,
    Delete       = 65522,
}

impl Key {
    #[inline(always)]
    pub fn code(&self) -> u32 {
        *self as u32
    }

    pub fn get_char(&self) -> Option<char> {
        char::from_u32(self.code())
    }
}

impl FromPrimitive for Key {
    fn from_u64(n: u64) -> Option<Key> {
        match n {
            0     => Some(Key::Unknown),
            9     => Some(Key::Tab),
            13    => Some(Key::Enter),
            17    => Some(Key::CtrlQ),
            18    => Some(Key::CtrlR),
            19    => Some(Key::CtrlS),
            27    => Some(Key::Esc),
            32    => Some(Key::Space),
            33    => Some(Key::Exclaim),
            35    => Some(Key::Hash),
            36    => Some(Key::Dollar),
            37    => Some(Key::Percent),
            38    => Some(Key::Ampersand),
            39    => Some(Key::Quote),
            40    => Some(Key::LeftParen),
            41    => Some(Key::RightParen),
            42    => Some(Key::Asterisk),
            43    => Some(Key::Plus),
            44    => Some(Key::Comma),
            45    => Some(Key::Minus),
            46    => Some(Key::Period),
            47    => Some(Key::Slash),
            48    => Some(Key::D0),
            49    => Some(Key::D1),
            50    => Some(Key::D2),
            51    => Some(Key::D3),
            52    => Some(Key::D4),
            53    => Some(Key::D5),
            54    => Some(Key::D6),
            55    => Some(Key::D7),
            56    => Some(Key::D8),
            57    => Some(Key::D9),
            58    => Some(Key::Colon),
            59    => Some(Key::Semicolon),
            60    => Some(Key::Less),
            61    => Some(Key::Equals),
            62    => Some(Key::Greater),
            63    => Some(Key::Question),
            64    => Some(Key::At),
            91    => Some(Key::LeftBracket),
            92    => Some(Key::Backslash),
            93    => Some(Key::RightBracket),
            94    => Some(Key::Caret),
            95    => Some(Key::Underscore),
            96    => Some(Key::Backquote),
            97    => Some(Key::A),
            98    => Some(Key::B),
            99    => Some(Key::C),
            100   => Some(Key::D),
            101   => Some(Key::E),
            102   => Some(Key::F),
            103   => Some(Key::G),
            104   => Some(Key::H),
            105   => Some(Key::I),
            106   => Some(Key::J),
            107   => Some(Key::K),
            108   => Some(Key::L),
            109   => Some(Key::M),
            110   => Some(Key::N),
            111   => Some(Key::O),
            112   => Some(Key::P),
            113   => Some(Key::Q),
            114   => Some(Key::R),
            115   => Some(Key::S),
            116   => Some(Key::T),
            117   => Some(Key::U),
            118   => Some(Key::V),
            119   => Some(Key::W),
            120   => Some(Key::X),
            121   => Some(Key::Y),
            122   => Some(Key::Z),
            123   => Some(Key::LeftBrace),
            124   => Some(Key::Pipe),
            125   => Some(Key::RightBrace),
            126   => Some(Key::Tilde),
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

    #[inline(always)]
    fn from_u32(n: u32) -> Option<Key> {
        FromPrimitive::from_u64(n as u64)
    }
}
