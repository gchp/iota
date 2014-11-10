use utils;

pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}


pub struct Cursor {
    pub x: int,
    pub y: int,
}

impl Cursor {
    pub fn new() -> Cursor {
        Cursor {
            x: 0,
            y: 0,
        }
    }

    pub fn draw(&self) {
        utils::draw_cursor(self.x, self.y);
    }

    pub fn adjust(&mut self, direction: Direction) {
        match direction {
            Up => self.y -= 1,
            Down => self.y += 1,
            Left => self.x -= 1,
            Right => self.x += 1,
        }
    }
}
