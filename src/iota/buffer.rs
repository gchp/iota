//TODO: Write tests
//TODO: UTF8 support

use log::{Log, Change, LogEntry};

use gapbuffer::GapBuffer;

use std::cmp;
use std::collections::HashMap;
use std::io::{File, Reader, BufferedReader};

#[deriving(PartialEq, Eq, Hash)]
pub enum Mark {
    Point,                  //The active index being edited (there is always exactly 1).
    Cursor(uint),           //For keeping track of a cursor, which the point could be set to.
    DisplayMark(uint),      //For using in determining some display of characters.
    UserDefined(uint),      //For user defined marks.
}

#[deriving(Copy, Show, PartialEq, Eq)]
pub enum Direction {
    Up(uint), Down(uint), Left(uint), Right(uint),
    LineStart, LineEnd,
}

pub struct Buffer {
    text: GapBuffer<u8>,                    //Actual text data being edited.
    marks: HashMap<Mark, (uint, uint)>,     //Table of marked indices in the text.
                                            // KEY: mark id => VALUE : (absolute index, line index)
    log: Log,                            //History of undoable transactions.
    pub file_path: Option<Path>,            //TODO: replace with a general metadata table
}

impl Buffer {

    //----- CONSTRUCTORS ---------------------------------------------------------------------------

    /// Constructor for empty buffer.
    pub fn new() -> Buffer {
        let text = GapBuffer::new();
        let mut marks = HashMap::new();
        marks.insert(Mark::Point, (0,0));
        Buffer {
            file_path: None,
            text: text,
            marks: marks,
            log: Log::new(),
        }
    }

    /// Constructor for buffer from reader.
    pub fn new_from_reader<R: Reader>(reader: R) -> Buffer {
        let mut buff = Buffer::new();
        if let Ok(contents) = BufferedReader::new(reader).read_to_string() {
            buff.text.extend(contents.bytes());
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

    ///Length of the text stored in this buffer.
    pub fn len(&self) -> uint {
        self.text.len()
    }

    ///The x,y coordinates of a mark within the file. None if not a valid mark.
    pub fn get_mark_coords(&self, mark: Mark) -> Option<(uint, uint)> {
        if let Some(idx) = self.get_mark_idx(mark) {
            if let Some(line) = self.get_line(idx) {
                let newlines: Vec<&u8> = self.text[..line].iter()
                                                          .filter(|ch| -> bool { *ch == &b'\n' })
                                                          .collect();
                Some((idx - line, newlines.len()))
            } else { None }
        } else { None }
    }

    ///The absolute index of a mark within the file. None if not a valid mark.
    pub fn get_mark_idx(&self, mark: Mark) -> Option<uint> {
        if let Some(&(idx, _)) = self.marks.get(&mark) {
            if idx < self.len() {
                Some(idx)
            } else { None }
        } else { None }
    }

    ///Creates an iterator on the text by lines.
    pub fn lines(&self) -> Lines {
        Lines {
            buffer: self.text[],
            tail: 0,
            head: self.len()
        }
    }

    ///Creates an iterator on the text by lines that begins at the specified mark.
    pub fn lines_from(&self, mark: Mark) -> Option<Lines> {
        if let Some(&(idx, _)) = self.marks.get(&mark) {
            if idx < self.len() {
                Some(Lines {
                    buffer: self.text[idx..],
                    tail: 0,
                    head: self.len() - idx,
                })
            } else { None }
        } else { None }
    }

    ///Returns the status text for this buffer.
    pub fn status_text(&self) -> String {
        match self.file_path {
            Some(ref path)  =>  format!("{}", path.display()),
            None            =>  format!("untitled"),
        }
    }

    //----- MUTATORS -------------------------------------------------------------------------------

    ///Sets the mark to a given absolute index. Adds a new mark or overwrites an existing mark.
    pub fn set_mark(&mut self, mark: Mark, idx: uint) {
        if let Some(line) = self.get_line(idx) {
            if let Some(tuple) = self.marks.get_mut(&mark) {
                *tuple = (idx, idx - line);
                return;
            }
            self.marks.insert(mark, (idx, idx - line));
        }
    }

    ///Sets the mark to a given x,y coordinates. Adds a new mark or overwrites an existing mark.
    pub fn set_mark_by_coords(&mut self, mark: Mark, x: uint, y: uint) {
        let mut y_idx = 0;
        for _ in range(0, y) {
            y_idx = self.get_line_end(y_idx).unwrap() + 1;
        }
        if y_idx + x < self.len() {
            if let Some(tuple) = self.marks.get_mut(&mark) {
                *tuple = (y_idx + x, x);
                return;
            }
            self.marks.insert(mark, (y_idx + x, x));
        }        
    }

    //Shift a mark relative to its position according to the direction given.
    pub fn shift_mark(&mut self, mark: Mark, direction: Direction) {
        if let Some(tuple) = match direction {
            Direction::Left(_)   | Direction::Right(_)  =>  self.offset_mark(mark, direction),
            Direction::Up(_)     | Direction::Down(_)   =>  self.offset_mark_line(mark, direction),
            Direction::LineStart | Direction::LineEnd   =>  {
                if let Some(&(idx, _)) = self.marks.get(&mark) {
                    if direction == Direction::LineStart {
                        let start = self.get_line(idx).unwrap();
                        Some((start, 0))
                    } else {
                        let end = self.get_line_end(idx).unwrap();
                        Some((end, end - self.get_line(idx).unwrap()))
                    }
                }
                else { None }
            }
        } {
            if let Some(old_mark) = self.marks.get_mut(&mark) { *old_mark = tuple; }
        }
    }

    ///Remove the char at the point.
    pub fn remove_char(&mut self) -> Option<u8> {
        if let Some(&(idx, _)) = self.marks.get(&Mark::Point) {
            if let Some(ch) = self.text.remove(idx) {
                let mut transaction = self.log.start(idx);
                transaction.log(Change::Remove(idx, ch), idx);
                Some(ch)
            } else { None }
        } else { None }
    }

    ///Insert a char at the point.
    pub fn insert_char(&mut self, ch: u8) {
        if let Some(&(idx, _)) = self.marks.get(&Mark::Point) {
            self.text.insert(idx, ch);
            let mut transaction = self.log.start(idx);
            transaction.log(Change::Insert(idx, ch), idx);
        }
    }

    ///Updates the point to be equivalent to a given mark.
    pub fn update_point(&mut self, mark: Mark) {
        if let Some(&tuple) = self.marks.get(&mark) {
            if tuple.val0() < self.len() {
                *self.marks.get_mut(&Mark::Point).unwrap() = tuple;
            }
        }
    }

    pub fn redo(&mut self) {
        if let Some(transaction) = self.log.redo() {
            self.reverse(transaction);
        }
    }

    pub fn undo(&mut self) {
        if let Some(transaction) = self.log.undo() {
            self.reverse(transaction);
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
                    Some(&b'\n') | None => { return Some(idx); }
                    _ => { idx -= 1; }
                }
            }
        } else { None }
    }

    //Returns the index of the newline character at the end of the line mark is in.
    //Newline after mark (INCLUSIVE).
    //None iff mark is outside the len of text.
    fn get_line_end(&self, mark: uint) -> Option<uint> {
    //FIXME unicode support
        if mark < self.len() {
            let mut idx = mark;
            loop {
                match self.text[mark..self.len()].iter().next() {
                    Some(&b'\n') | None => { return Some(idx); }
                    _ => { idx += 1; }
                }
            }
        } else { None }
    }

    //Returns the mark offset by some number of chars.
    //None iff mark is not in the hashmap.
    fn offset_mark(&self, mark: Mark, direction: Direction) -> Option<(uint, uint)> {
    //FIXME unicode support
        let offset = match direction {
            Direction::Left(n)  =>  -(n as int),
            Direction::Right(n) =>    n as int ,
            _ => 0,
        };
        if let Some(tuple) = self.marks.get(&mark) {
            let idx = (*tuple).val0() as int + offset;
            match (idx >= 0, idx < self.len() as int ) {
                (true, true)    => Some((idx as uint,
                                         idx as uint - self.get_line(idx as uint).unwrap())),
                (false, true)   => Some((0, 0)),
                (_, false)      => Some((self.len() -1,
                                        self.len() - 1 - self.get_line(self.len() -1).unwrap())),
            }
        } else { None }
    }

    //Returns the mark offset by some number of line breaks.
    // None iff mark is not in the hashmap.
    fn offset_mark_line(&self, mark: Mark, direction: Direction) -> Option<(uint, uint)> {
    //FIXME unicode support
        let offset = match direction {
            Direction::Up(n)    =>  -(n as int),
            Direction::Down(n)  =>    n as int ,
            _ => 0,
        };
        if let Some(tuple) = self.marks.get(&mark) {
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

    fn reverse(&mut self, transaction: &LogEntry) {
        for change in transaction.changes.iter() {
            match change {
                &Change::Insert(idx, ch) => { 
                    self.set_mark(Mark::Point, idx);
                    self.insert_char(ch);
                }
                &Change::Remove(idx, _) => {
                    self.set_mark(Mark::Point, idx);
                    self.remove_char();
                }
            }
        }
    }

}

//----- ITERATE BY LINES ---------------------------------------------------------------------------

pub struct Lines<'a> {
    buffer: &'a [u8],
    tail: uint,
    head: uint,
}

impl<'a> Iterator<&'a [u8]> for Lines<'a> {
//FIXME unicode support?

    fn next(&mut self) -> Option<&'a [u8]> {
        if self.tail != self.head {

            let old_tail = self.tail;

            //update tail to either the first char after the next \n or to self.head
            self.tail = range(old_tail, self.head).filter(|idx| -> bool {
                self.buffer[*idx] == b'\n' || *idx + 1 == self.head
            }).min().unwrap() + 1;

            Some(self.buffer[old_tail..self.tail])

        } else { None }
    }

    fn size_hint(&self) -> (uint, Option<uint>) {
        //TODO: this is technically correct but a better estimate could be implemented
        (1, Some(self.head))
    }

}

//----- TESTS --------------------------------------------------------------------------------------

#[cfg(test)]
mod test {

    //TODO Tests

}
