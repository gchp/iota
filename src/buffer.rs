use std::io::{File, BufferedReader};
use std::collections::DList;
use std::cell::RefCell;

use utils;
use cursor::{Direction, Cursor};


pub struct Buffer {
    pub file_path: String,
    pub lines: DList<RefCell<Line>>,

    pub cursor: Cursor,
}

impl Buffer {
    /// Create a new buffer instance
    pub fn new() -> Buffer {
        Buffer {
            file_path: String::new(),
            lines: DList::new(),
            cursor: Cursor::new(),
        }
    }

    /// Create a new buffer instance and load the given file
    pub fn new_from_file(path: &Path) -> Buffer {
        let mut file = BufferedReader::new(File::open(path));
        let lines: Vec<String> = file.lines().map(|x| x.unwrap()).collect();
        let mut buffer = Buffer::new();

        buffer.file_path = path.as_str().unwrap().to_string();

        // for every line in the file we add a corresponding line to the buffer
        for (index, line) in lines.iter().enumerate() {
            buffer.lines.push_back(RefCell::new(Line::new(line.clone(), index)));
        }

        buffer
    }

    /// Draw the contents of the buffer
    ///
    /// Loops over each line in the buffer and draws it to the screen
    pub fn draw_contents(&self) {
        for (index, line) in self.lines.iter().enumerate() {
            utils::draw(index, line.borrow().data.clone());
        }
    }

    pub fn draw_status(&self) {
        let height = utils::get_term_height();
        utils::draw(height - 1, self.file_path.clone());
    }

    pub fn adjust_cursor(&mut self, dir: Direction) {
        let (mut x, mut y) = self.cursor.buffer_pos.expand();
        match dir {
            Direction::Up => {
                if self.get_line_at(y-1).is_some() {
                    y -= 1;
                }
            }
            Direction::Down => {
                if self.get_line_at(y+1).is_some() {
                    y += 1
                }
            }
            Direction::Right => {
                let line = &self.get_line_at(y);
                if line.is_some() && line.unwrap().borrow().len() > x {
                    x += 1
                }
            }
            Direction::Left => {
                let line = &self.get_line_at(y);
                if line.is_some() && x > 0 {
                    x -= 1
                }
            }
        }
        self.cursor.adjust_buffer_pos(x, y);
    }

    fn get_line_at(&self, line_num: uint) -> Option<&RefCell<Line>> {
        for line in self.lines.iter() {
            if line.borrow().num == line_num {
                return Some(line)
            }
        }
        None
    }

}


pub struct Line {
    data: String,
    num: uint,
}

impl Line {
    /// Create a new line instance
    pub fn new(data: String, n: uint) -> Line {
        Line{
            data: data,
            num: n
        }
    }

    /// Get the length of the current line
    pub fn len(&self) -> uint {
        self.data.len()
    }
}


