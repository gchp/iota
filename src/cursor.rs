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

    fn get_linenum(&self) -> uint {
        let (_, line_num) = self.expand();
        line_num
    }

    fn get_offset(&self) -> uint {
        let (offset, _) = self.expand();
        offset
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

    pub fn get_linenum(&self) -> uint {
        self.buffer_pos.get_linenum()
    }

    pub fn set_linenum(&mut self, line_num: uint) {
        let offset = self.get_offset();
        self.buffer_pos = CursorPos::Place(offset, line_num);
    }

    pub fn get_offset(&self) -> uint {
        self.buffer_pos.get_offset()
    }

    pub fn set_offset(&mut self, offset: uint) {
        let line_num = self.get_linenum();
        self.buffer_pos = CursorPos::Place(offset, line_num);
    }

    pub fn set_line(&mut self, line: Option<&'c RefCell<Line>>) {
        self.line = line;

        // check that the current offset is longer than the length of the line
        let offset = self.get_offset();
        let line_length = self.get_line().borrow().len();
        if offset > line_length {
            self.set_offset(line_length);
        }
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

    pub fn move_right(&mut self) {
        let line_len = self.get_line().borrow().len();
        let current_offset = self.get_offset();
        if line_len > current_offset {
            self.set_offset(current_offset + 1);
        }
    }

    pub fn move_left(&mut self) {
        let current_offset = self.get_offset();
        if current_offset > 0 {
            self.set_offset(current_offset - 1);
        }
    }

    pub fn get_status_text(&self) -> String {
        let (offset, line_num) = self.get_position();
        format!("({}, {})", offset, line_num)
    }
}
