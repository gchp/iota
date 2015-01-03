//FIXME: Check unicode support

use log::{Log, Change, LogEntry};

use gapbuffer::GapBuffer;

use std::{ cmp, char };
use std::char::UnicodeChar;
use std::collections::HashMap;
use std::io::{File, Reader, BufferedReader};

#[derive(Copy, PartialEq, Eq, Hash, Show)]
pub enum Mark {
    Cursor(uint),           //For keeping track of cursors.
    DisplayMark(uint),      //For using in determining some display of characters.
}

#[derive(Copy, PartialEq, Eq, Show)]
pub enum WordEdgeMatch {
    Alphabet,
    Whitespace,
}

#[derive(Copy, PartialEq, Eq, Show)]
pub enum Direction {
    Up(uint), Down(uint), Left(uint), Right(uint),
    LeftWord(uint, WordEdgeMatch), RightWord(uint, WordEdgeMatch),
    LineStart, LineEnd,
}

pub struct Buffer {
    text: GapBuffer<u8>,                    //Actual text data being edited.
    marks: HashMap<Mark, (uint, uint)>,     //Table of marked indices in the text.
                                            // KEY: mark id => VALUE : (absolute index, line index)
    pub log: Log,                           //History of undoable transactions.
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
            if let Some(line) = get_line(idx, &self.text) {
                Some((idx - line, range(0, idx).filter(|i| -> bool { self.text[*i] == b'\n' })
                                               .count()))
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
            Some(ref path)  =>  format!("{} ", path.display()),
            None            =>  format!("untitled "),
        }
    }

    //----- MUTATORS -------------------------------------------------------------------------------

    ///Sets the mark to a given absolute index. Adds a new mark or overwrites an existing mark.
    pub fn set_mark(&mut self, mark: Mark, idx: uint) {
        if let Some(line) = get_line(idx, &self.text) {
            if let Some(tuple) = self.marks.get_mut(&mark) {
                *tuple = (idx, idx - line);
                return;
            }
            self.marks.insert(mark, (idx, idx - line));
        }
    }

    //Shift a mark relative to its position according to the direction given.
    pub fn shift_mark(&mut self, mark: Mark, direction: Direction) {
        let last = self.len() - 1;
        let text = &self.text;
        if let Some(tuple) = self.marks.get_mut(&mark) {
            let (idx, line_idx) = *tuple;
            if let (Some(line), Some(line_end)) = (get_line(idx, text), get_line_end(idx, text)) {
                *tuple = match direction {
                    //For every relative motion of a mark, should return this tuple:
                    //  value 0:    the absolute index of the mark in the file
                    //  value 1:    the index of the mark in its line (unchanged by direct verticle
                    //              traversals)
                    Direction::Left(n)      =>  {
                        if idx >= n { (idx - n, idx - n - get_line(idx - n, text).unwrap()) }
                        else { (0, 0) }
                    }
                    Direction::Right(n)     =>  {
                        if idx + n < last { (idx + n, idx + n - get_line(idx + n, text).unwrap()) }
                        else { (last, last - get_line(last, text).unwrap()) }
                    }
                    Direction::Up(n)        =>  {
                        let nlines = range(0, idx).rev().filter(|i| text[*i] == b'\n')
                                                        .take(n + 1)
                                                        .collect::<Vec<uint>>();
                        if n == nlines.len() { (cmp::min(line_idx, nlines[0]), line_idx) }
                        else if n > nlines.len() { (0, 0) }
                        else { (cmp::min(line_idx + nlines[n] + 1, nlines[n-1]), line_idx) }
                    }
                    Direction::Down(n)      =>  {
                        let nlines = range(idx, text.len()).filter(|i| text[*i] == b'\n')
                                                           .take(n + 1)
                                                           .collect::<Vec<uint>>();
                        if n > nlines.len() { (last, last - get_line(last, text).unwrap())
                        } else if n == nlines.len() {
                            (cmp::min(line_idx + nlines[n-1] + 1, last), line_idx)
                        } else { (cmp::min(line_idx + nlines[n-1] + 1, nlines[n]), line_idx) }
                    }
                    Direction::RightWord(n_words, edger)     =>  {
                        if let Some(new_idx) = get_words(idx, n_words, edger, text) {
                            if new_idx < last { (new_idx, new_idx - get_line(new_idx, text).unwrap()) }
                            else { (last, last - get_line(last, text).unwrap()) }
                        } else { (last, last - get_line(last, text).unwrap()) }
                    }
                    Direction::LeftWord(n_words, edger)     =>  {
                        if let Some(new_idx) = get_words_rev(idx, n_words, edger, text) {
                            if new_idx > 0 { (new_idx, new_idx - get_line(new_idx, text).unwrap()) }
                            else { (0, 0) }
                        } else { (0, 0) }
                    }
                    Direction::LineStart    =>  { (line, 0) }
                    Direction::LineEnd      =>  { (line_end, line_end - line) }
                }
            }
        }
    }

    ///Remove the chars at the mark.
    pub fn remove_chars(&mut self, mark: Mark, direction: Direction) -> Option<Vec<u8>> {
        let text = &mut self.text;
        if let Some(&(idx, _)) = self.marks.get(&mark) {
            let range = match direction {
                Direction::Left(n) => range(cmp::max(0, idx - n), idx),
                Direction::Right(n) => range(idx, cmp::min(idx + n, text.len())),
                Direction::RightWord(n_words, edger) => {
                    range(idx, get_words(idx, n_words, edger, text).unwrap_or(text.len()))
                }
                Direction::LeftWord(n_words, edger) => {
                    range(get_words_rev(idx, n_words, edger, text).unwrap_or(0), idx)
                }
                _ => unimplemented!()
            };
            let mut transaction = self.log.start(idx);
            let mut vec = range
                .rev()
                .filter_map(|idx| text.remove(idx).map(|ch| (idx, ch)))
                .inspect(|&(idx, ch)| transaction.log(Change::Remove(idx, ch), idx))
                .map(|(_, ch)| ch)
                .collect::<Vec<u8>>();
            vec.reverse();
            Some(vec)
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

    ///Redo most recently undone action.
    pub fn redo(&mut self) -> Option<&LogEntry> {
        if let Some(transaction) = self.log.redo() {
            commit(transaction, &mut self.text);
            Some(transaction)
        } else { None }
    }

    ///Undo most recently performed action.
    pub fn undo(&mut self) -> Option<&LogEntry> {
        if let Some(transaction) = self.log.undo() {
            commit(transaction, &mut self.text);
            Some(transaction)
        } else { None }
    }

}

fn is_alphanumeric_or_(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

impl WordEdgeMatch {
    /// If c1 -> c2 is the start of a word.
    /// If end of word matching is wanted then pass the chars in reversed.
    fn is_word_edge(&self, c1: &u8, c2: &u8) -> bool {
        match (self, char::from_u32(*c1 as u32).unwrap(), char::from_u32(*c2 as u32).unwrap()) {
            (_, '\n', '\n') => true, // Blank lines are always counted as a word
            (&WordEdgeMatch::Whitespace, c1, c2) => c1.is_whitespace() && !c2.is_whitespace(),
            (&WordEdgeMatch::Alphabet, c1, c2) if c1.is_whitespace() => !c2.is_whitespace(),
            (&WordEdgeMatch::Alphabet, c1, c2) if is_alphanumeric_or_(c1) => !is_alphanumeric_or_(c2) && !c2.is_whitespace(),
            (&WordEdgeMatch::Alphabet, c1, c2) if !is_alphanumeric_or_(c1) => is_alphanumeric_or_(c2) && !c2.is_whitespace(),
            (&WordEdgeMatch::Alphabet, _, _) => false,
        }
    }
}

fn get_words(mark: uint, n_words: uint, edger: WordEdgeMatch, text: &GapBuffer<u8>) -> Option<uint> {
    range(mark + 1, text.len() - 1)
        .filter(|idx| edger.is_word_edge(&text[*idx - 1], &text[*idx]))
        .take(n_words)
        .next()
}

fn get_words_rev(mark: uint, n_words: uint, edger: WordEdgeMatch, text: &GapBuffer<u8>) -> Option<uint> {
    range(1, mark)
        .rev()
        .filter(|idx| edger.is_word_edge(&text[*idx - 1], &text[*idx]))
        .take(n_words)
        .next()
}

//Returns the index of the first character of the line the mark is in.
//Newline prior to mark (EXCLUSIVE) + 1.
//None iff mark is outside of the len of text.
fn get_line(mark: uint, text: &GapBuffer<u8>) -> Option<uint> {
    if mark <= text.len() {
        range(0, mark + 1).rev().filter(|idx| *idx == 0 || text[*idx - 1] == b'\n')
                                .take(1)
                                .next()
    } else { None }
}

//Returns the index of the newline character at the end of the line mark is in.
//Newline after mark (INCLUSIVE).
//None iff mark is outside the len of text.
fn get_line_end(mark: uint, text: &GapBuffer<u8>) -> Option<uint> {
    if mark <= text.len() {
        range(mark, text.len() + 1).filter(|idx| *idx == text.len() ||text[*idx] == b'\n')
                                   .take(1)
                                   .next()
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

impl<'a> Iterator for Lines<'a> {
    type Item = &'a [u8];

    fn next(&mut self) -> Option<&'a [u8]> {
        if self.tail != self.head {

            let old_tail = self.tail;
            //update tail to either the first char after the next \n or to self.head
            self.tail = range(old_tail, self.head).filter(|i| { *i + 1 == self.head 
                                                                || self.buffer[*i] == b'\n' })
                                                  .take(1)
                                                  .next()
                                                  .unwrap() + 1;
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
        buffer.remove_chars(Mark::Cursor(0), Direction::Right(1));

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
        buffer.set_mark(Mark::Cursor(0), 10);
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

        assert_eq!(buffer.get_mark_idx(Mark::Cursor(0)).unwrap(), 4);
    }

    #[test]
    fn test_to_lines() {
        let buffer = setup_buffer("Test\nA\nTest");
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

    #[test]
    fn move_from_final_position() {
        let mut buffer = setup_buffer("Test");
        buffer.set_mark(Mark::Cursor(0), 4);
        buffer.shift_mark(Mark::Cursor(0), Direction::Left(1));

        assert_eq!(buffer.get_mark_idx(Mark::Cursor(0)).unwrap(), 3);
    }

}
