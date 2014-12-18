extern crate gapbuffer;

use std::io::{File, Reader, BufferedReader};
use gapbuffer::GapBuffer;

#[deriving(Copy, Show)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

pub struct Buffer {
    pub file_path: Option<Path>,
    pub text: GapBuffer<u8>,
    pub cursor: uint,
}

impl Buffer {

    /// Constructor for empty buffer.
    pub fn new() -> Buffer {
        let text: GapBuffer<u8> = GapBuffer::new();
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
            Direction::Up /*if not the first line.*/            => {
                //up code
            }
            Direction::Down /*if not the last line.*/           => {
                //down code
            }
            Direction::Left if self.cursor > 0                  => {
                //needs to be changed for proper utf8 support?
                self.cursor -= 1;
            }
            Direction::Right if self.cursor < self.text.len()   => {
                //needs to be changed for proper utf8 support?
                self.cursor += 1;
            }
            _ => { break; }
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
