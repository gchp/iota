use utils;
use buffer::{Line, Link};

pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}


#[deriving(Clone)]
pub struct Cursor {
    pub x: uint,
    pub y: uint,

    pub line: Link,
}

impl Cursor {
    /// Create a new cursor instance
    pub fn new() -> Cursor {
        Cursor {
            x: 0,
            y: 0,
            line: None,
        }
    }

    /// Draw the cursor based on the `x` and `y` values
    pub fn draw(&self) {
        utils::draw_cursor(self.x, self.y);
    }

    /// Adjust the cursor position
    pub fn adjust(&mut self, direction: Direction) {
        match direction {
            Up => { self.move_up() },
            Down => { self.move_down() },
            Left => { self.move_left() },
            Right => { self.move_right() },
        }
    }

    /// Get the current line
    ///
    /// Returns a clone of the line from within the Link structure
    fn get_line(&mut self) -> Box<Line> {
        self.line.clone().unwrap()
    }

    /// Move the cursor up one line
    fn move_up(&mut self) {
        let mut line = self.get_line();

        let prev_line = line.prev.resolve().map(|prev| prev);
        if prev_line.is_some() {
            self.line = Some(box prev_line.unwrap().clone());
            self.y -= 1;
            self.maybe_goto_end_line();
        }
    }

    /// Move the cursor down one line
    fn move_down(&mut self) {
        let line = self.get_line();
        if line.next.is_some() {
            self.y += 1;
            self.line = line.next;
            self.maybe_goto_end_line();
        }

    }

    /// Move the cursor left by one character
    fn move_left(&mut self) {
        if self.x > 0 {
            self.x -= 1
        }
    }

    /// Move the cursor right by one character
    fn move_right(&mut self) {
        let line = self.get_line();
        if self.x < line.len() {
            self.x += 1
        }
    }

    /// Check if the cursors `x` position is greater than the length of the
    /// current line. If so, move it back to the end of the line.
    fn maybe_goto_end_line(&mut self) {
        let line = self.get_line();
        if line.len() < self.x {
            self.goto_end_line();
        }
    }

    fn goto_end_line(&mut self) {
        self.x = self.get_line().len()
    }

}
