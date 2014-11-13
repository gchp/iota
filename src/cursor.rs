use utils;

pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[deriving(Clone)]
pub enum CursorPos {
    Place(uint, uint),
}

impl CursorPos {
    pub fn expand(&self) -> (uint, uint) {
        match  self {
            &Place(x, y) => return (x, y)
        }
    }
}

#[deriving(Clone)]
pub struct Cursor {
    pub buffer_pos: CursorPos,
}

impl Cursor {
    /// Create a new cursor instance
    pub fn new() -> Cursor {
        Cursor {
            buffer_pos: Place(0, 0)
        }
    }

    /// Draw the cursor based on the `x` and `y` values
    pub fn draw(&self) {
        match self.buffer_pos {
            Place(x, y) => utils::draw_cursor(x, y)
        }
    }

    pub fn adjust_buffer_pos(&mut self, x: uint, y: uint) {
        self.buffer_pos = Place(x, y);
    }
}
