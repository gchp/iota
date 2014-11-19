use utils;

pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

enum CursorPos {
    Place(uint, uint),
}

impl CursorPos {
    fn expand(&self) -> (uint, uint) {
        match self {
            &CursorPos::Place(x, y) => return (x, y)
        }
    }

    fn get_offset(&self) -> uint {
        match self {
            &CursorPos::Place(x, _) => return x
        }
    }

    fn get_linenum(&self) -> uint {
        match self {
            &CursorPos::Place(_, y) => return y
        }
    }
}

pub struct Cursor {
    buffer_pos: CursorPos,
}

impl Cursor {
    /// Create a new cursor instance
    pub fn new() -> Cursor {
        Cursor {
            buffer_pos: CursorPos::Place(0, 0)
        }
    }

    /// Draw the cursor
    pub fn draw(&self) {
        let (offset, line_num) = self.get_position();
        utils::draw_cursor(offset, line_num)
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
