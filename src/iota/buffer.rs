#![feature(slicing_syntax)]

//TODO: UTF8 support
//TODO: Save cursor x offset as it traverses up/down
//TODO: Write tests

extern crate gapbuffer;

use std::io::{File, Reader, BufferedReader};
use gapbuffer::GapBuffer;

#[deriving(Copy, Show)]
pub enum Direction {
    Up, Down, Left, Right,
}

pub struct Buffer {
    pub file_path: Option<Path>,
    pub text: GapBuffer<char>,
    pub cursor: uint,
}

//Private methods
impl Buffer {

    //Returns the number of newlines in the buffer before the mark.
    fn get_line(&self, mark: uint) -> Option<uint> {
        let mut linenum = 0;
        if mark < self.text.len() {
            for c in self.text[0..mark].iter() {
                if c == &'\n' { linenum += 1; }
            }
            Some(linenum)
        } else { None }
    }

    //Returns the index of the nth newline in the buffer.
    fn get_line_idx(&self, line: uint) -> Option<uint> {
        let mut linenum = 0;
        for idx in range(0, self.text.len()) {
            if self.text[idx] == '\n' { linenum += 1; if linenum == line { return Some(idx) } }
        }
        None
    }

    //Returns the index of the point the mark would be at if shifted 'offset' lines.
    fn move_line(&self, mark: uint, offset: int) -> Option<uint> {
        let mut x_offset = 1;
        loop {
            match self.text[0..mark].iter().next_back() {
                Some(&'\n') | None => { break; }
                _                  => { x_offset += 1; }
            }
        }
        if let Some(line) = self.get_line(mark) {
            let newline = (line as int - offset) as uint;
            Some(self.get_line_idx(newline).unwrap() + x_offset)
        } else { None } 
    }


}

//Public methods
impl Buffer {

    /// Constructor for empty buffer.
    pub fn new() -> Buffer {
        let text = GapBuffer::new();
        Buffer {
            file_path: None,
            text: text,
            cursor: 0u,
        }
    }

    /// Constructor for buffer from reader.
    pub fn new_from_reader<R: Reader>(reader: R) -> Buffer {
        let mut buff = Buffer::new();
        if let Ok(contents) = BufferedReader::new(reader).read_to_string() {
            buff.text.extend(contents.chars());
        }
        buff
    }

    /// Constructor for buffer from file.
    pub fn new_from_file(path: Path) -> Buffer {
        if let Ok(file) = File::open(&path) {
            let mut buff = Buffer::new_from_reader(file);
            buff.file_path = Some(path);
            buff
        } else {
            Buffer::new()
        }
    }

    //Move the cursor by some amount (can be negative).
    pub fn move_cursor(&mut self, offset: int) {
        let idx = self.cursor as int + offset;
        if 0 >= idx && idx > self.text.len() as int {
            self.set_cursor(idx as uint);
        }
    }

    //Set the cursor to some index.
    pub fn set_cursor(&mut self, location: uint) {
        self.cursor = location;
    }

    //Shift the cursor by one in any of the four directions.
    pub fn shift_cursor(&mut self, direction: Direction) {
        match direction {
            Direction::Up if self.get_line(self.cursor).unwrap() > 0             => {
                self.move_line(self.cursor, -1);
            }
            Direction::Down
            if self.get_line(self.cursor) < self.get_line(self.text.len() - 1)  => {
                self.move_line(self.cursor, 1);
            }
            Direction::Left if self.cursor > 0                                  => {
                self.cursor -= 1;
            }
            Direction::Right if self.cursor < self.text.len()                   => {
                self.cursor += 1;
            }
            _ => { }
        }
    }

    //Remove the char the cursor is at the index of.
    pub fn remove_char(&mut self) -> Option<char> {
        self.text.remove(self.cursor)
    }

    //Insert the char the cursor is at the index of.
    pub fn insert_char(&mut self, c: char) {
        self.text.insert(self.cursor, c);
    }

}

#[cfg(test)]
mod test {
    
}
