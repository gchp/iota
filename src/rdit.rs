use std::collections::dlist::DList;
use std::io::{File, BufferedReader};

pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}


pub enum Response {
    Continue,
    Quit,
}

pub struct Buffer {
    pub lines: DList<Line>,
    pub active: bool,
    pub num_lines: int,
    pub cursor_x: int,
    pub cursor_y: int,
}

impl Buffer {
    pub fn new() -> Buffer {
        Buffer {
            lines: DList::new(),
            active: false,
            num_lines: 0,
            cursor_x: 0,
            cursor_y: 0,
        }
    }

    pub fn new_from_file(filename: String) -> Buffer {
        let path = Path::new(filename.to_string());

        let mut new_buffer = Buffer::new();
        let mut file = BufferedReader::new(File::open(&path));
        let lines: Vec<String> = file.lines().map(|x| x.unwrap()).collect();

        for line in lines.iter() {
            new_buffer.lines.push(Line{data: line.clone()})
        }

        new_buffer
    }

    pub fn move_cursor(&mut self, direction: Direction) {
        match direction {
            Up =>  self.cursor_y -= 1,
            Down => self.cursor_y += 1,
            Left => self.cursor_x -= 1,
            Right => self.cursor_x += 1,
        }
    }
}

pub struct Line {
    pub data: String,
}
