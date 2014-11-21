use std::cell::RefCell;

use buffer::Line;
use utils;

pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[deriving(Clone)]
enum CursorPos {
    Place(uint, uint),
}

impl CursorPos {
    fn expand(&self) -> (uint, uint) {
        match self {
            &CursorPos::Place(x, y) => return (x, y)
        }
    }
}

#[deriving(Clone)]
pub struct Cursor<'c> {
    buffer_pos: CursorPos,
    line: Option<&'c RefCell<Line>>,
}

impl<'c> Cursor<'c> {
    /// Create a new cursor instance
    pub fn new() -> Cursor<'c> {
        Cursor {
            buffer_pos: CursorPos::Place(0, 0),
            line: None,
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

    pub fn set_line(&mut self, line: Option<&'c RefCell<Line>>) {
        self.line = line
    }

    pub fn get_line(&self) -> &'c RefCell<Line> {
        self.line.unwrap()
    }

    pub fn delete_char(&mut self) {
        let (mut offset, line_num) = self.get_position();
        offset -= 1;
        let line = self.get_line();
        line.borrow_mut().data.remove(offset);
        self.set_position(offset, line_num);
    }

    pub fn insert_char(&mut self, ch: char) {
        let (mut offset, line_num) = self.get_position();
        let line = self.get_line();
        line.borrow_mut().data.insert(offset, ch);
        offset += 1;
        self.set_position(offset, line_num);
    }
}
