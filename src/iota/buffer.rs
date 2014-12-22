#![feature(slicing_syntax)]

//TODO: UTF8 support
//TODO: Write tests

extern crate gapbuffer;

use std::cmp;
use std::collections::HashMap;
use std::io::{File, Reader, BufferedReader};

use gapbuffer::GapBuffer;

#[deriving(Copy, Show)]
pub enum Direction {
    Up, Down, Left, Right,
}

pub struct Buffer {
    pub cursor: &'static str,                   //Key for the current editing cursor.
    text: GapBuffer<char>,                      //Actual text data being edited.
    marks: HashMap<&'static str, (uint, uint)>, //Table of marked indices in the text.
                                                // KEY: name => VALUE : (absolute index, line index)
    file_path: Option<Path>,                    //TODO: replace with a general metadata table
}

impl Buffer {

    //----- CONSTRUCTORS ---------------------------------------------------------------------------

    /// Constructor for empty buffer.
    pub fn new() -> Buffer {
        let text = GapBuffer::new();
        let mut marks = HashMap::new();
        marks.insert("cursor", (0,0));
        Buffer {
            file_path: None,
            text: text,
            marks: marks,
            cursor: "cursor",
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
        } else { Buffer::new() }
    }

    //----- ACCESSORS ------------------------------------------------------------------------------

    pub fn len(&self) -> uint {
        self.text.len()
    }

    //----- MUTATORS -------------------------------------------------------------------------------

    //Shift the cursor by one in any of the four directions.
    pub fn shift_mark(&mut self, mark: &str, direction: Direction) {
        if let Some(tuple) = match direction {
            Direction::Left     =>  self.offset_mark(mark, -1),
            Direction::Right    =>  self.offset_mark(mark,  1),
            Direction::Up       =>  self.offset_mark_line(mark, -1),
            Direction::Down     =>  self.offset_mark_line(mark,  1),
        } { *self.marks.get_mut(mark).unwrap() = tuple; }
    }

    //Remove the char the cursor is at the index of.
    pub fn remove_char(&mut self) -> Option<char> {
        if let Some(tuple) = self.marks.get(self.cursor) {
            self.text.remove((*tuple).val0())
        } else { None }
    }

    //Insert the char the cursor is at the index of.
    pub fn insert_char(&mut self, ch: char) {
        if let Some(tuple) = self.marks.get(self.cursor) {
            self.text.insert((*tuple).val0(), ch);
        }
    }

    //----- PRIVATE METHODS ------------------------------------------------------------------------

    //Returns the index of the first character of the line the mark is in.
    //Newline prior to mark (EXCLUSIVE) + 1.
    //None iff mark is outside of the len of text.
    fn get_line(&self, mark: uint) -> Option<uint> {
    //FIXME unicode support
        if mark < self.len() {
            let mut idx = mark;
            loop {
                match self.text[0..mark].iter().next_back() {
                    Some(&'\n') | None => { return Some(idx); }
                    _ => { idx -= 1; }
                }
            }
        } else { None }
    }

    //Returns the index of the newline character at the end of the line mark is in.
    //Newline after mark (INCLUSIVE).
    //None iff mark is outside the len of text.
    fn get_line_end(&self, mark:uint) -> Option<uint> {
    //FIXME unicode support
        if mark < self.len() {
            let mut idx = mark;
            loop {
                match self.text[mark..self.len()].iter().next() {
                    Some(&'\n') | None => { return Some(idx); }
                    _ => { idx += 1; }
                }
            }
        } else { None }
    }

    //Returns the mark offset by some number of chars.
    fn offset_mark(&self, mark: &str, offset: int) -> Option<(uint, uint)> {
    //FIXME unicode support
        if let Some(tuple) = self.marks.get(mark) {
            let idx = (*tuple).val0() as int + offset;
            if idx > 0 && idx < self.len() as int {
                Some((idx as uint, idx as uint - self.get_line(idx as uint).unwrap()))
            } else { None }
        } else { None }
    }

    //Returns the mark offset by some number of line breaks.
    fn offset_mark_line(&self, mark: &str, offset: int) -> Option<(uint, uint)> {
    //FIXME unicode support
        if let Some(tuple) = self.marks.get(mark) {
            let mut line = self.get_line((*tuple).val0());
            let line_idx = (*tuple).val1();
            if offset >= 0 {
                for _ in range(0, offset as uint) {
                    line = Some(self.get_line_end(line.unwrap()).unwrap() + 1);
                    if line == None { line = self.get_line(self.len() - 1); break; }
                }
            } else {
                for _ in range(0, -offset as uint) {
                    line = self.get_line(line.unwrap() - 1);
                    if line == None { line = Some(0); break; }
                }
            }
            Some((cmp::min(line.unwrap() + line_idx, self.get_line_end(line.unwrap()).unwrap()),
                 line_idx))
        } else { None }
    }

}

//----- TESTS---------------------------------------------------------------------------------------

#[cfg(test)]
mod test {

    fn setup_buffer() -> Buffer {
        let mut buffer = Buffer::new();
        buffer.file_path = Some(Path::new("/some/file.txt"));
        for c in "test\n\ntext file\ncontent".chars() {
            buffer.insert_char(c);
            buffer.shift_mark(buffer.cursor, Direction::Right);
        }
    }

    //TODO Tests

}
