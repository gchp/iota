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
        match self {
            &CursorPos::Place(x, y) => return (x, y)
        }
    }

    pub fn get_offset(&self) -> uint {
        match self {
            &CursorPos::Place(x, _) => return x
        }
    }

    pub fn get_linenum(&self) -> uint {
        match self {
            &CursorPos::Place(_, y) => return y
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
            buffer_pos: CursorPos::Place(0, 0)
        }
    }

    /// Draw the cursor based on the `x` and `y` values
    pub fn draw(&self) {
        match self.buffer_pos {
            CursorPos::Place(x, y) => utils::draw_cursor(x, y)
        }
    }

    pub fn set_position(&mut self, x: uint, y: uint) {
        self.buffer_pos = CursorPos::Place(x, y);
    }

    pub fn get_position(&self) -> (uint, uint) {
        self.buffer_pos.expand()
    }

    pub fn get_offset(&self) -> uint {
        self.buffer_pos.get_offset()
    }

    pub fn get_linenum(&self) -> uint {
        self.buffer_pos.get_linenum()
    }
}
