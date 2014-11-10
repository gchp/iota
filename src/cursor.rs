use utils;
use buffer::Link;

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
    pub fn new() -> Cursor {
        Cursor {
            x: 0,
            y: 0,
            line: None,
        }
    }

    pub fn draw(&self) {
        utils::draw_cursor(self.x, self.y);
    }

    pub fn adjust(&mut self, direction: Direction) {
        let mut line = self.line.clone().unwrap();
        match direction {
            Up => {
                let prev_line = line.prev.resolve().map(|prev| prev);
                if prev_line.is_some() {
                    self.line = Some(box prev_line.unwrap().clone());
                    self.y -= 1
                }
            },
            Down => {
                if line.next.is_some() {
                    self.y += 1;
                    self.line = line.next;
                }
            },
            Left => {
                if self.x > 0 {
                    self.x -= 1
                }
            },
            Right => {
                if self.x < line.len() {
                    self.x += 1
                }
            },
        }
    }
}
