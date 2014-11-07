extern crate rustbox;

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
    pub fn draw(&self) {
        rustbox::set_cursor(self.x, self.y);        
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
