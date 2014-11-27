use std::cell::RefCell;

use buffer::Line;

pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    pub fn is_right(&self) -> bool {
        match *self {
            Direction::Right => true,
            _                => false
        }
    }
    pub fn is_left(&self) -> bool {
        match *self {
            Direction::Left => true,
            _               => false
        }
    }
}

#[deriving(Clone)]
pub struct Cursor<'c> {
    pub offset: uint,
    line: Option<&'c RefCell<Line>>,
}

impl<'c> Cursor<'c> {
    /// Create a new cursor instance
    pub fn new() -> Cursor<'c> {
        Cursor {
            offset: 0,
            line: None,
        }
    }

    pub fn get_position(&self) -> (uint, uint) {
        (self.offset, self.get_linenum())
    }

    pub fn get_linenum(&self) -> uint {
        self.line.unwrap().borrow().linenum
    }

    pub fn get_offset(&self) -> uint {
        self.offset
    }

    pub fn set_offset(&mut self, offset: uint) {
        self.offset = offset;
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

    pub fn get_line_length(&self) -> uint {
        self.get_line().borrow().len()
    }

    pub fn delete_backward_char(&mut self) {
        let offset = self.get_offset();
        let line = self.get_line();
        line.borrow_mut().data.remove(offset-1);
        self.set_offset(offset-1);
    }

    pub fn delete_forward_char(&mut self) {
        let offset = self.get_offset();
        let line = self.get_line();
        line.borrow_mut().data.remove(offset);
        self.set_offset(offset);
    }

    pub fn insert_char(&mut self, ch: char) {
        let offset = self.get_offset();
        let line = self.get_line();
        line.borrow_mut().data.insert(offset, ch);
        self.set_offset(offset+1)
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
