//FIXME: Check unicode support

use log::{Log, Change, LogEntry};

use gapbuffer::GapBuffer;

use std::cmp;
use std::collections::HashMap;
use std::io::{File, Reader, BufferedReader};

#[deriving(Copy, PartialEq, Eq, Hash, Show)]
pub enum Mark {
    Cursor(uint),           //For keeping track of cursors.
    DisplayMark(uint),      //For using in determining some display of characters.
}

#[deriving(Copy, PartialEq, Eq, Show)]
pub enum Direction {
    Up(uint), Down(uint), Left(uint), Right(uint),
    LineStart, LineEnd,
}

pub struct Buffer {
    text: GapBuffer<u8>,                    //Actual text data being edited.
    marks: HashMap<Mark, (uint, uint)>,     //Table of marked indices in the text.
                                            // KEY: mark id => VALUE : (absolute index, line index)
    log: Log,                               //History of undoable transactions.
    pub file_path: Option<Path>,            //TODO: replace with a general metadata table
}

impl Buffer {

    //----- CONSTRUCTORS ---------------------------------------------------------------------------

    /// Constructor for empty buffer.
    pub fn new() -> Buffer {
        Buffer {
            file_path: None,
            text: GapBuffer::new(),
            marks: HashMap::new(),
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
        self.text.len() + 1
    }

    ///The x,y coordinates of a mark within the file. None if not a valid mark.
    pub fn get_mark_coords(&self, mark: Mark) -> Option<(uint, uint)> {
        if let Some(idx) = self.get_mark_idx(mark) {
            if let Some(line) = self.get_line(idx) {
                Some((idx - line, self.text[..line].iter()
                                                   .filter(|ch| -> bool { *ch == &b'\n' })
                                                   .collect::<Vec<&u8>>()
                                                   .len()))
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
                        if let Some(start) = self.get_line(idx) {
                            Some((start, 0))
                        } else { None }
                    } else {
                        if let Some(end) = self.get_line_end(idx) {
                            Some((end, end - self.get_line(idx).unwrap()))
                        } else { None }
                    }
                } else { None }
            }
        } {
            if let Some(old_mark) = self.marks.get_mut(&mark) { *old_mark = tuple; }
        }
    }

    ///Remove the char at the mark.
    pub fn remove_char(&mut self, mark: Mark) -> Option<u8> {
        if let Some(&(idx, _)) = self.marks.get(&mark) {
            if let Some(ch) = self.text.remove(idx) {
                let mut transaction = self.log.start(idx);
                transaction.log(Change::Remove(idx, ch), idx);
                Some(ch)
            } else { None }
        } else { None }
    }

    ///Insert a char at the mark.
    pub fn insert_char(&mut self, mark: Mark, ch: u8) {
        if let Some(&(idx, _)) = self.marks.get(&mark) {
            self.text.insert(idx, ch);
            let mut transaction = self.log.start(idx);
            transaction.log(Change::Insert(idx, ch), idx);
        }
    }

    pub fn redo(&mut self) {
        if let Some(transaction) = self.log.redo() {
            commit(transaction, &mut self.text);
        }
    }

    pub fn undo(&mut self) {
        if let Some(transaction) = self.log.undo() {
            commit(transaction, &mut self.text);
        }
    }

    //----- PRIVATE METHODS ------------------------------------------------------------------------

    fn get_line(&self, mark: uint) -> Option<uint> { get_line(mark, &self.text) }
    fn get_line_end(&self, mark: uint) -> Option<uint> { get_line_end(mark, &self.text) }

    //Returns the mark offset by some number of chars.
    //None iff mark is not in the hashmap.
    fn offset_mark(&self, mark: Mark, direction: Direction) -> Option<(uint, uint)> {
        let offset = match direction {
            Direction::Left(n)  =>  -(n as int),
            Direction::Right(n) =>    n as int ,
            _ => 0,
        };
        if let Some(tuple) = self.marks.get(&mark) {
            let idx = (*tuple).0 as int + offset;
            match (idx >= 0, idx < self.len() as int ) {
                (true, true)    => Some((idx as uint,
                                         idx as uint - self.get_line(idx as uint).unwrap())),
                (false, true)   => Some((0, 0)),
                (_, false)      => Some((self.len() - 1,
                                         self.len() - 1 - self.get_line(self.len()-1).unwrap())),
            }
        } else { None }
    }

    //Returns the mark offset by some number of line breaks.
    // None iff mark is not in the hashmap.
    fn offset_mark_line(&self, mark: Mark, direction: Direction) -> Option<(uint, uint)> {
        let offset = match direction {
            Direction::Up(n)    =>  -(n as int),
            Direction::Down(n)  =>    n as int ,
            _ => 0,
        };
        if let Some(&(idx, line_idx)) = self.marks.get(&mark) {
            let line = if offset > 0 {
                let nlines = range(idx, self.len()).filter(|i| *i == 0 || self.text[*i-1] == b'\n')
                                                   .collect::<Vec<uint>>();
                if nlines.len() > 0 { nlines[cmp::min((offset - 1) as uint, nlines.len() - 1)] }
                else { self.get_line(idx).unwrap() }
            } else if offset < 0 {
                let nlines = range(0, idx+1).filter(|i| *i == 0 || self.text[*i - 1] == b'\n')
                                            .collect::<Vec<uint>>();
                if nlines.len() > 0 { nlines[cmp::max(nlines.len() as int + offset-1, 0) as uint] }
                else { self.get_line(idx).unwrap() }
            } else { self.get_line(idx).unwrap() };

            Some((cmp::min(line + line_idx, self.get_line_end(line).unwrap()), line_idx))
        } else { None }
    }

}

//Returns the index of the first character of the line the mark is in.
//Newline prior to mark (EXCLUSIVE) + 1.
//None iff mark is outside of the len of text.
fn get_line(mark: uint, text: &GapBuffer<u8>) -> Option<uint> {
    if mark <= text.len() {
        range(0, mark + 1).filter(|idx| *idx == 0 || text[*idx - 1] == b'\n').max()
    } else { None }
}

//Returns the index of the newline character at the end of the line mark is in.
//Newline after mark (INCLUSIVE).
//None iff mark is outside the len of text.
fn get_line_end(mark: uint, text: &GapBuffer<u8>) -> Option<uint> {
    if mark <= text.len() {
        range(mark, text.len()).filter(|idx| *idx == text.len() - 1 ||text[*idx] == b'\n').min()
    } else { None }
}

//Performs a transaction on the passed in buffer.
fn commit(transaction: &LogEntry, text: &mut GapBuffer<u8>) {
    for change in transaction.changes.iter() {
        match change {
            &Change::Insert(idx, ch) => { 
                text.insert(idx, ch);
            }
            &Change::Remove(idx, _) => {
                text.remove(idx);
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

    fn next(&mut self) -> Option<&'a [u8]> {
        if self.tail != self.head {

            let old_tail = self.tail;
            //update tail to either the first char after the next \n or to self.head
            self.tail = range(old_tail, self.head).filter(|idx| -> bool {
                *idx + 1 == self.head || self.buffer[*idx] == b'\n'
            }).min().unwrap() + 1;
            if self.tail == self.head { Some(self.buffer[old_tail..self.tail-1]) }
            else { Some(self.buffer[old_tail..self.tail]) }

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

    use buffer::{Buffer, Direction, Mark};

    fn setup_buffer(testcase: &'static str) -> Buffer {
        let mut buffer = Buffer::new();
        buffer.text.extend(testcase.bytes());
        buffer.set_mark(Mark::Cursor(0), 0);
        buffer
    }

    #[test]
    fn test_insert() {
        let mut buffer = setup_buffer("");
        buffer.insert_char(Mark::Cursor(0), b'A');
        assert_eq!(buffer.len(), 2);
        assert_eq!(buffer.lines().next().unwrap(), [b'A']);
    }

    #[test]
    fn test_remove() {
        let mut buffer = setup_buffer("ABCD");
        buffer.remove_char(Mark::Cursor(0));

        assert_eq!(buffer.len(), 4);
        assert_eq!(buffer.lines().next().unwrap(), [b'B', b'C', b'D']);
    }

    #[test]
    fn test_set_mark() {
        let mut buffer = setup_buffer("Test");
        buffer.set_mark(Mark::Cursor(0), 2);

        assert_eq!(buffer.get_mark_idx(Mark::Cursor(0)).unwrap(), 2);
    }

    #[test]
    fn test_shift_down() {
        let mut buffer = setup_buffer("Test\nA\nTest");
        buffer.set_mark(Mark::Cursor(0), 2);
        buffer.shift_mark(Mark::Cursor(0), Direction::Down(2));

        assert_eq!(buffer.get_mark_coords(Mark::Cursor(0)).unwrap(), (2, 2));
    }

    #[test]
    fn test_shift_right() {
        let mut buffer = setup_buffer("Test");
        buffer.shift_mark(Mark::Cursor(0), Direction::Right(1));

        assert_eq!(buffer.get_mark_idx(Mark::Cursor(0)).unwrap(), 1);
    }

    #[test]
    fn test_shift_up() {
        let mut buffer = setup_buffer("Test\nA\nTest");
        buffer.set_mark_by_coords(Mark::Cursor(0), 2, 2);
        buffer.shift_mark(Mark::Cursor(0), Direction::Up(1));

        assert_eq!(buffer.get_mark_coords(Mark::Cursor(0)).unwrap(), (1, 1));
    }

    #[test]
    fn test_shift_left() {
        let mut buffer = setup_buffer("Test");
        buffer.set_mark(Mark::Cursor(0), 2);
        buffer.shift_mark(Mark::Cursor(0), Direction::Left(1));

        assert_eq!(buffer.get_mark_idx(Mark::Cursor(0)).unwrap(), 1);
    }

    #[test]
    fn test_shift_linestart() {
        let mut buffer = setup_buffer("Test");
        buffer.set_mark(Mark::Cursor(0), 2);
        buffer.shift_mark(Mark::Cursor(0), Direction::LineStart);

        assert_eq!(buffer.get_mark_idx(Mark::Cursor(0)).unwrap(), 0);
    }

    #[test]
    fn test_shift_lineend() {
        let mut buffer = setup_buffer("Test");
        buffer.shift_mark(Mark::Cursor(0), Direction::LineEnd);

        assert_eq!(buffer.get_mark_idx(Mark::Cursor(0)).unwrap(), 3);
    }

    #[test]
    fn test_to_lines() {
        let mut buffer = setup_buffer("Test\nA\nTest");
        let mut lines = buffer.lines();

        assert_eq!(lines.next().unwrap(), [b'T',b'e',b's',b't',b'\n']);
        assert_eq!(lines.next().unwrap(), [b'A',b'\n']);
        assert_eq!(lines.next().unwrap(), [b'T',b'e',b's',b't']);
    }

    #[test]
    fn test_to_lines_from() {
        let mut buffer = setup_buffer("Test\nA\nTest");
        buffer.set_mark(Mark::Cursor(0), 6);
        let mut lines = buffer.lines_from(Mark::Cursor(0)).unwrap();

        assert_eq!(lines.next().unwrap(), [b'\n']);
        assert_eq!(lines.next().unwrap(), [b'T',b'e',b's',b't']);
    }

}
