//FIXME: Check unicode support
// stdlib dependencies
use std::cmp;
use std::path::PathBuf;
use std::fs::File;
use std::io::{Stdin, Read};
use std::convert::From;
use std::fmt;
use std::str;

// external dependencies
use gapbuffer::GapBuffer;

// local dependencies
use log::{Log, Change, LogEntry};
use utils;
use utils::is_alpha_or_;
use input::Input;
use textobject::{TextObject, Kind, Offset, Anchor};

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum WordEdgeMatch {
    Alphabet,
    Whitespace,
}

pub struct Buffer {
    /// Current buffers text
    text: GapBuffer<u8>,

    /// Transaction history (used for undo/redo)
    pub log: Log,

    /// Location on disk where the current buffer should be written
    pub file_path: Option<PathBuf>,
}

impl Buffer {
    /// Constructor for empty buffer.
    pub fn new() -> Buffer {
        Buffer {
            file_path: None,
            text: GapBuffer::new(),
            log: Log::new(),
        }
    }

    /// The number of lines in the buffer. At least 1.
    pub fn num_lines(&self) -> usize {
        (0..self.text.len()).filter(|&i| self.text[i] == b'\n').count() + 1
    }

    /// Returns true if the buffer contains no characters.
    pub fn is_empty(&self) -> bool {
        self.text.len() == 0
    }

    /// Retrieves the character at a given position.
    pub fn char_at(&self, pos: Position) -> Option<char> {
        self.get_index(pos).map(|index| {
            self.char_at_index(index)
        })
    }

    /// Insert a char at the position.
    pub fn insert_char(&mut self, pos: Position, ch: char) {
        let idx = self.get_writable_index(pos).unwrap_or_else(|| {
            panic!("can't insert character into invalid position {:?}", pos);
        });
        self.insert_char_at_index(idx, ch);
        let mut transaction = self.log.start(idx);
        transaction.log(Change::Insert(idx, ch), idx);
    }

    /// Removes the characters between the two positions.
    pub fn remove(&mut self, start: Position, end: Position) {
        let a = self.get_index(start);
        let b = self.get_writable_index(end);
        match (a, b) {
            (Some(a), Some(b)) => self.remove_range(a, b),
            _ => panic!("can't remove text between invalid positions: {:?} to {:?}", start, end),
        }
    }

    /// Retrieves the character at the given byte index.
    fn char_at_index(&self, index: usize) -> char {
        let bytes = index..(index + utils::char_length(self.text[index]));
        let bytes: Vec<u8> = bytes.map(|i| self.text[i]).collect();
        str::from_utf8(&bytes).unwrap().chars().next().unwrap()
    }

    /// Returns the text in the byte-index range as a String.
    fn slice_bytes(&self, start: usize, end: usize) -> String {
        let bytes: Vec<u8> = (start..end).map(|i| self.text[i]).collect();
        String::from(str::from_utf8(&bytes).unwrap())
    }

    /// Inserts a character at the given byte index.
    fn insert_char_at_index(&mut self, index: usize, ch: char) {
        let bytes = ch.to_string().into_bytes();
        for &byte in bytes.iter().rev() {
            self.text.insert(index, byte);
        }
    }

    /// Remove the chars in the byte-index range
    fn remove_range(&mut self, start: usize, mut end: usize) {
        debug_assert!(self.has_character_at_index(start));
        let mut removed_chars = Vec::new();
        while end > start {
            let idx = self.prev_char_index(end).unwrap();
            let ch = self.remove_char_at_index(idx);
            removed_chars.push((idx, ch));
            end = idx;
        }
        let mut transaction = self.log.start(start);
        for (idx, ch) in removed_chars {
            transaction.log(Change::Remove(idx, ch), idx);
        }
    }

    /// Remove the char at the given byte index and return it
    fn remove_char_at_index(&mut self, index: usize) -> char {
        let c = self.char_at_index(index);
        let len = utils::char_length(self.text[index]);
        for _ in 0..len {
            self.text.remove(index);
        }
        c
    }

    /// The index of a Position within the file in bytes. None if the argument
    /// represents a position that doesn't have a corresponding character. If
    /// Some(x) is returned, then x is a valid index for reading into the
    /// backing GapBuffer.
    fn get_index(&self, pos: Position) -> Option<usize> {
        match self.get_writable_index(pos) {
            Some(index) if index < self.text.len() => Some(index),
            _ => None
        }
    }

    /// The index of a Position within the file in bytes. Unlike get_index,
    /// this function can return indices which are valid for writing to but
    /// not for reading from. If Some(x) is returned, then x is a valid index
    /// for inserting a character into the backing GapBuffer.
    fn get_writable_index(&self, pos: Position) -> Option<usize> {
        self.get_writable_index_of_line(pos.y).and_then(|line_start| {
            let mut index = line_start;
            for _ in 0..pos.x {
                if index >= self.text.len() {
                    return None;
                }
                match self.text[index] {
                    b'\n' => { return None },
                    b => { index += utils::char_length(b) },
                }
            }
            Some(index)
        })
    }

    /// The byte index of the first character on line `lineno`. None if the
    /// buffer doesn't have enough lines. If Some(x) is returned, then x is a
    /// valid index for reading into the backing GapBuffer.
    fn get_index_of_line(&self, lineno: usize) -> Option<usize> {
        match self.get_writable_index_of_line(lineno) {
            Some(idx) if idx < self.text.len() => Some(idx),
            _ => None,
        }
    }

    /// The byte index of the first character on line `lineno`. None if the
    /// buffer doesn't have enough lines. If Some(x) is returned, then x is a
    /// valid index for writing into the backing GapBuffer.
    fn get_writable_index_of_line(&self, lineno: usize) -> Option<usize> {
        if lineno == 0 {
            Some(0)
        } else {
            (0..self.text.len())
                .filter(|&i| self.text[i] == b'\n')
                .nth(lineno - 1)
                .map(|i| i + 1)
        }
    }

    /// Finds the position of a character at the given byte index. None if no
    /// such character exists.
    fn get_position(&self, index: usize) -> Option<Position> {
        if self.has_character_at_index(index) {
            let newlines = (0..index).filter(|&i| self.text[i] == b'\n').collect::<Vec<usize>>();
            let y = newlines.len();
            let mut cur_index = newlines.last().map(|i| i + 1).unwrap_or(0);
            let mut x = 0;
            while cur_index < index {
                cur_index += utils::char_length(self.text[cur_index]);
                x += 1;
            }
            Some(Position::new(x, y))
        } else {
            None
        }
    }

    fn get_writable_position(&self, index: usize) -> Option<Position> {
        if index == self.text.len() {
            Some(self.last_writable_position())
        } else {
            self.get_position(index)
        }
    }

    /// Finds the position of the last character in the buffer. Returns None
    /// if the buffer is empty.
    fn last_readable_position(&self) -> Option<Position> {
        // find the last u8 which is the first u8 of a character.
        (0..self.text.len()).rev()
                            .find(|&i| utils::is_start_of_char(self.text[i]))
                            .and_then(|i| self.get_position(i))

    }

    /// Finds the position after the last character in the buffer, or (0, 0)
    /// if the buffer is empty.
    fn last_writable_position(&self) -> Position {
        // find the last u8 which is the first u8 of a character.
        let index = (0..self.text.len()).rev()
                                        .find(|&i| utils::is_start_of_char(self.text[i]));
        if let Some(index) = index {
            let pos = self.get_position(index).unwrap();
            match self.text[index] {
                b'\n' => Position::new(0, pos.y + 1),
                _ => Position::new(pos.x + 1, pos.y),
            }
        } else {
            Position::origin()
        }
    }

    /// Finds the next position after the given position in the buffer.
    /// Returns None if the given position doesn't correspond to a character.
    /// The returned position will either correspond to a character or be the
    /// next position after the last character in the buffer.
    pub fn next_position(&self, pos: Position) -> Option<Position> {
        self.get_index(pos).map(|index| {
            if self.text[index] == b'\n' {
                Position::new(0, pos.y + 1)
            } else {
                Position::new(pos.x + 1, pos.y)
            }
        })
    }

    /// Finds the previous readable position before the given position in the
    /// buffer. Returns None if the position is (0, 0) or the buffer is empty.
    /// The returned position will correspond to a character.
    pub fn prev_position(&self, pos: Position) -> Option<Position> {
        if pos == Position::origin() {
            None
        } else if let Some(index) = self.get_index(pos) {
            let prev_index = self.prev_char_index(index).unwrap();
            self.get_position(prev_index)
        } else {
            self.last_readable_position()
        }
    }

    /// Returns true if the given byte index points to a character in the
    /// buffer.
    fn has_character_at_index(&self, index: usize) -> bool {
        index < self.text.len() && utils::is_start_of_char(self.text[index])
    }

    /// Finds the byte index of the character immediately before the character
    /// specified by the given byte index. None if the given index is zero.
    fn prev_char_index(&self, start: usize) -> Option<usize> {
        debug_assert!(start == self.text.len() || self.has_character_at_index(start));
        (0..start).rev().find(|&i| utils::is_start_of_char(self.text[i]))
    }

    /// Finds the "closest" position less than or equal to the given position
    /// that can be written to.
    fn closest_writable_position(&self, pos: Position) -> Position {
        if let Some(line_index) = self.get_writable_index_of_line(pos.y) {
            // step through the line until the char is found or the end of the
            // buffer or \n is reached
            let mut index = line_index;
            let mut x = 0;
            while x < pos.x && index < self.text.len() && self.text[index] != b'\n' {
                index += utils::char_length(self.text[index]);
                x += 1;
            }
            Position::new(x, pos.y)
        } else {
            // not enough lines
            self.last_writable_position()
        }
    }

    /// Returns true if the given position can be written to.
    fn position_is_writable(&self, pos: Position) -> bool {
        self.get_writable_index(pos).is_some()
    }

    /// Creates a BufferSlice representing the text between the two positions.
    pub fn slice(&self, start: Position, end: Position) -> BufferSlice {
        // make sure that the positions are valid for the buffer
        // to satify the assumptions of BufferSlice
        debug_assert!(self.position_is_writable(start));
        debug_assert!(self.position_is_writable(end));
        debug_assert!(start <= end);
        BufferSlice {
            buffer: self,
            start: start,
            end: end,
        }
    }

    /// Creates a BufferSlice representing the text up to (but not including)
    /// the specified position.
    pub fn slice_to(&self, end: Position) -> BufferSlice {
        self.slice(Position::origin(), end)
    }

    /// Creates a BufferSlice representing the text from the specified
    /// position until the end of the buffer.
    pub fn slice_from(&self, start: Position) -> BufferSlice {
        self.slice(start, self.last_writable_position())
    }

    /// Creates a BufferSlice representing the full text in the buffer.
    pub fn slice_full(&self) -> BufferSlice {
        self.slice(Position::origin(), self.last_writable_position())
    }

    /// Return the position of a TextObject relative to a given position. If
    /// Some(pos) is returned, pos will be a writable position for the buffer.
    pub fn get_object_position(&self, start: Position, obj: TextObject) -> Option<Position> {
        debug_assert!(self.get_writable_index(start).is_some());
        match obj.kind {
            Kind::Char => self.get_char_object_position(start, obj.offset),
            Kind::Line(anchor) => self.get_line_object_position(start, obj.offset, anchor),
            Kind::Word(anchor) => self.get_word_object_position(start, obj.offset, anchor),
        }
    }

    fn get_char_object_position(&self, start: Position, offset: Offset) -> Option<Position> {
        match offset {
            // the char `offset` chars after `start`
            Offset::Forward(offset) => {
                self.slice_from(start).positions().nth(offset)
                    .or_else(|| Some(self.last_writable_position()))
            }

            Offset::Backward(0) => Some(start),

            // the char `offset` chars before `start`
            Offset::Backward(offset) => {
                self.slice_to(start).positions().rev().nth(offset - 1)
                    .or_else(|| Some(Position::origin()))
            }
        }
    }

    /// Get the position of a specific line in the buffer relative to a given
    /// position based on a given Anchor.
    fn get_line_object_position(&self, start: Position, offset: Offset, anchor: Anchor) -> Option<Position> {
        let lineno = match offset {
            Offset::Forward(offset)  => { start.y + offset }
            Offset::Backward(offset) => { start.y - cmp::min(start.y, offset) }
        };
        match anchor {
            Anchor::Start => {
                let start = Position::new(0, lineno);
                match self.position_is_writable(start) {
                    true => Some(start),
                    false => None,
                }
            }

            Anchor::End => {
                let next_line = Position::new(0, lineno + 1);
                match self.position_is_writable(next_line) {
                    true => self.prev_position(next_line),
                    false => Some(self.closest_writable_position(next_line)),
                }
            }

            Anchor::Same => {
                let same = Position::new(start.x, lineno);
                match self.position_is_writable(same) {
                    true => Some(same),
                    false => Some(self.closest_writable_position(same)),
                }
            }

            _ => {
                print!("Unhandled line anchor: {:?} ", anchor);
                Some(start)
            }
        }
    }

    fn get_word_object_position(&self, start: Position, offset: Offset, anchor: Anchor) -> Option<Position> {
        match anchor {
            Anchor::Start => {}
            _ => {
                print!("Unhandled word anchor: {:?} ", anchor);
                return Some(start);
            }
        }

        match offset {
            Offset::Forward(nth_word)  => {
                self.get_index(start).map(|start_index| {
                    get_words(start_index, nth_word, WordEdgeMatch::Whitespace, &self.text).and_then(|index| {
                        self.get_writable_position(index)
                    }).unwrap_or_else(|| self.last_writable_position())
                })
            }

            Offset::Backward(nth_word) => {
                self.get_index(start).map(|start_index| {
                    get_words_rev(start_index, nth_word, WordEdgeMatch::Whitespace, &self.text).and_then(|index| {
                        self.get_writable_position(index)
                    }).unwrap_or(Position::origin())
                })
            }
        }
    }

    /// Returns the status text for this buffer.
    pub fn status_text(&self) -> String {
        match self.file_path {
            Some(ref path)  =>  format!("{} ", path.display()),
            None            =>  format!("untitled "),
        }
    }

    pub fn remove_object(&mut self, pos: Position, object: TextObject) {
        let object_start = TextObject { kind: object.kind.with_anchor(Anchor::Start), offset: object.offset };
        let object_end = TextObject { kind: object.kind.with_anchor(Anchor::End), offset: object.offset };

        let start = self.get_object_position(pos, object_start);
        let end = self.get_object_position(pos, object_end);

        if let (Some(a), Some(b)) = (start, end) {
            self.remove(a, b);
        }
    }

    // FIXME once unicode support is implemented for Log, redo() and undo()
    // can be changed to return Option<LogEntry>

    /// Redo most recently undone action.
    pub fn redo(&mut self) -> Option<Position> {
        if let Some(transaction) = self.log.redo() {
            commit(&transaction, self);
            self.get_position(transaction.end_point)
        } else { None }
    }

    /// Undo most recently performed action.
    pub fn undo(&mut self) -> Option<Position> {
        if let Some(transaction) = self.log.undo() {
            commit(&transaction, self);
            self.get_position(transaction.end_point)
        } else { None }
    }
}

impl ToString for Buffer {
    fn to_string(&self) -> String {
        let bytes: Vec<u8> = self.text.iter().map(|&b| b).collect();
        str::from_utf8(&bytes).unwrap().to_string()
    }
}

/// An (x, y) position in a buffer, where x is in units of characters, and y
/// is in units of lines.
/// 
/// It is important to note that this doesn't represent a position in screen
/// coordinates because of multi-codepoint graphemes and graphemes with a
/// width greater than 1.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Position {
    /// The vertical distance, in lines, from the first line in a buffer.
    pub y: usize,

    /// The horizontal distance, in characters, from the first character on a
    /// line.
    pub x: usize,
}

impl Position {
    /// Returns the position at (0, 0)
    pub fn origin() -> Position {
        Position { x: 0, y: 0 }
    }

    /// Returns the position at (x, y)
    pub fn new(x: usize, y: usize) -> Position {
        Position { x: x, y: y }
    }
}

impl fmt::Debug for Position {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

/// A section of a buffer representing all of the text between two positions.
/// Represents characters in the range [start, end). Both start and end must
/// either correspond to characters in the buffer or be the next position
/// after the last character in the buffer.
#[derive(Copy, Clone)]
pub struct BufferSlice<'a> {
    buffer: &'a Buffer,
    start: Position,
    end: Position,
}

impl<'a> BufferSlice<'a> {
    /// Checks to see if the slice contains any characters.
    pub fn is_empty(&self) -> bool {
        self.start >= self.end
    }

    pub fn positions(self) -> Positions<'a> {
        Positions { slice: self }
    }

    pub fn chars(self) -> Chars<'a> {
        Chars { positions: self.positions() }
    }

    pub fn lines(self) -> Lines<'a> {
        Lines { slice: self }
    }
}

impl<'a> ToString for BufferSlice<'a> {
    /// Returns the text in the slice as a String.
    fn to_string(&self) -> String {
        let a = self.buffer.get_index(self.start).unwrap();
        let b = self.buffer.get_writable_index(self.end).unwrap();
        self.buffer.slice_bytes(a, b)
    }
}

/// An iterator over the positions in a BufferSlice which have corresponding
/// characters.
pub struct Positions<'a> {
    slice: BufferSlice<'a>
}

impl<'a> Iterator for Positions<'a> {
    type Item = Position;

    fn next(&mut self) -> Option<Position> {
        if !self.slice.is_empty() {
            let result = self.slice.start;
            self.slice.start = self.slice.buffer.next_position(result).unwrap();
            Some(result)
        } else {
            None
        }
    }
}

impl<'a> DoubleEndedIterator for Positions<'a> {
    fn next_back(&mut self) -> Option<Position> {
        if !self.slice.is_empty() {
            self.slice.end = self.slice.buffer.prev_position(self.slice.end).unwrap();
            Some(self.slice.end)
        } else {
            None
        }
    }
}

/// An iterator over the characters in a BufferSlice
pub struct Chars<'a> {
    positions: Positions<'a>
}

impl<'a> Iterator for Chars<'a> {
    type Item = char;

    fn next(&mut self) -> Option<char> {
        self.positions.next().map(|pos| {
            self.positions.slice.buffer.char_at(pos).unwrap()
        })
    }
}

impl<'a> DoubleEndedIterator for Chars<'a> {
    fn next_back(&mut self) -> Option<char> {
        self.positions.next_back().map(|pos| {
            self.positions.slice.buffer.char_at(pos).unwrap()
        })
    }
}

/// An iterator over the lines in a BufferSlice
pub struct Lines<'a> {
    slice: BufferSlice<'a>
}

impl<'a> Iterator for Lines<'a> {
    type Item = String;

    fn next(&mut self) -> Option<String> {
        if !self.slice.is_empty() {
            let start = self.slice.start;
            let end = cmp::min(Position::new(0, start.y + 1), self.slice.end);
            self.slice.start = end;
            Some(self.slice.buffer.slice(start, end).to_string())
        } else {
            None
        }
    }
}

impl<'a> DoubleEndedIterator for Lines<'a> {
    fn next_back(&mut self) -> Option<String> {
        if !self.slice.is_empty() {
            let end = self.slice.end;
            let start_y = match end.x {
                0 => end.y - 1,
                _ => end.y
            };
            let start = cmp::max(Position::new(0, start_y), self.slice.start);
            self.slice.end = start;
            Some(self.slice.buffer.slice(start, end).to_string())
        } else {
            None
        }
    }
}

// This is a bit of a hack to get around an error I was getting when
// implementing From<R: Read> for Buffer with From<PathBuf> for Buffer.
// The compiler was telling me this was a conflicting implementation even
// though Read is not implemented for PathBuf. Changing R: Read to
// R: Read + BufferFrom fixes the error.
//
// TODO: investigate this further - possible compiler bug?
trait BufferFrom {}
impl BufferFrom for Stdin {}
impl BufferFrom for File {}

impl From<PathBuf> for Buffer {
    fn from(path: PathBuf) -> Buffer {
        if let Ok(file) = File::open(&path) {
            let mut buff = Buffer::from(file);
            buff.file_path = Some(path);
            buff
        } else { Buffer::new() }
    }
}

impl<R: Read + BufferFrom> From<R> for Buffer {
    fn from(mut reader: R) -> Buffer {
        let mut buff = Buffer::new();
        let mut contents = String::new();
        if let Ok(_) = reader.read_to_string(&mut contents) {
            buff.text.extend(contents.bytes());
        }
        buff
    }
}

impl From<Input> for Buffer {
    fn from(input: Input) -> Buffer {
        match input {
            Input::Filename(path) => {
                match path {
                    Some(path) => Buffer::from(PathBuf::from(path)),
                    None       => Buffer::new(),
                }
            },
            Input::Stdin(reader) => {
                Buffer::from(reader)
            }
        }
    }
}


impl WordEdgeMatch {
    /// If c1 -> c2 is the start of a word.
    /// If end of word matching is wanted then pass the chars in reversed.
    fn is_word_edge(&self, c1: &u8, c2: &u8) -> bool {
        // FIXME: unicode support - issue #69
        match (self, *c1 as char, *c2 as char) {
            (_, '\n', '\n') => true, // Blank lines are always counted as a word
            (&WordEdgeMatch::Whitespace, c1, c2) => c1.is_whitespace() && !c2.is_whitespace(),
            (&WordEdgeMatch::Alphabet, c1, c2) if c1.is_whitespace() => !c2.is_whitespace(),
            (&WordEdgeMatch::Alphabet, c1, c2) if is_alpha_or_(c1) => !is_alpha_or_(c2) && !c2.is_whitespace(),
            (&WordEdgeMatch::Alphabet, c1, c2) if !is_alpha_or_(c1) => is_alpha_or_(c2) && !c2.is_whitespace(),
            (&WordEdgeMatch::Alphabet, _, _) => false,
        }
    }
}

fn get_words(mark: usize, n_words: usize, edger: WordEdgeMatch, text: &GapBuffer<u8>) -> Option<usize> {
    (mark + 1..text.len() - 1)
        .filter(|idx| edger.is_word_edge(&text[*idx - 1], &text[*idx]))
        .take(n_words)
        .last()
}

fn get_words_rev(mark: usize, n_words: usize, edger: WordEdgeMatch, text: &GapBuffer<u8>) -> Option<usize> {
    (1..mark)
        .rev()
        .filter(|idx| edger.is_word_edge(&text[*idx - 1], &text[*idx]))
        .take(n_words)
        .last()
}

/// Performs a transaction on the passed in buffer.
fn commit(transaction: &LogEntry, buf: &mut Buffer) {
    for change in transaction.changes.iter() {
        match change {
            &Change::Insert(idx, ch) => {
                buf.insert_char_at_index(idx, ch);

            }
            &Change::Remove(idx, _) => {
                buf.remove_char_at_index(idx);
            }
        }
    }
}

#[cfg(test)]
mod test {

    use buffer::{Buffer, Position};
    use textobject::{TextObject, Offset, Kind, Anchor};

    fn setup_buffer(testcase: &'static str) -> Buffer {
        let mut buffer = Buffer::new();
        buffer.text.extend(testcase.bytes());
        buffer
    }

    #[test]
    fn move_char_right() {
        let buffer = setup_buffer("Some test content");
        let obj = TextObject {
            kind: Kind::Char,
            offset: Offset::Forward(1),
        };

        let result = buffer.get_object_position(Position::origin(), obj);
        
        assert_eq!(result.unwrap(), Position::new(1, 0));
    }

    #[test]
    fn move_char_left() {
        let buffer = setup_buffer("Some test content");
        let obj = TextObject {
            kind: Kind::Char,
            offset: Offset::Backward(1),
        };

        let result = buffer.get_object_position(Position::new(3, 0), obj);
        
        assert_eq!(result.unwrap(), Position::new(2, 0));
    }
    
    #[test]
    fn move_five_chars_right() {
        let buffer = setup_buffer("Some test content");
        let obj = TextObject {
            kind: Kind::Char,
            offset: Offset::Forward(5),
        };

        let result = buffer.get_object_position(Position::origin(), obj);
        
        assert_eq!(result.unwrap(), Position::new(5, 0));
    }

    #[test]
    fn move_line_down() {
        let buffer = setup_buffer("Some test content\nwith new\nlines!");
        let obj = TextObject {
            kind: Kind::Line(Anchor::Same),
            offset: Offset::Forward(1),
        };

        let result = buffer.get_object_position(Position::origin(), obj);
        
        assert_eq!(result.unwrap(), Position::new(0, 1));
    }

    #[test]
    fn move_line_up() {
        let buffer = setup_buffer("Some test content\nnew lines!");
        let obj = TextObject {
            kind: Kind::Line(Anchor::Same),
            offset: Offset::Backward(1),
        };

        let result = buffer.get_object_position(Position::new(1, 1), obj);
        
        assert_eq!(result.unwrap(), Position::new(1, 0));
    }

    #[test]
    fn move_two_lines_down() {
        let buffer = setup_buffer("Some test content\nwith new\nlines!");
        let obj = TextObject {
            kind: Kind::Line(Anchor::Same),
            offset: Offset::Forward(2),
        };

        let result = buffer.get_object_position(Position::origin(), obj);
        
        assert_eq!(result.unwrap(), Position::new(0, 2));
    }

    #[test]
    fn move_line_down_to_shorter_line() {
        let buffer = setup_buffer("Some test content\nwith new\nlines!");
        let obj = TextObject {
            kind: Kind::Line(Anchor::Same),
            offset: Offset::Forward(1),
        };

        let result = buffer.get_object_position(Position::new(15, 0), obj);

        assert_eq!(result.unwrap(), Position::new(8, 1));

        let obj = TextObject {
            kind: Kind::Line(Anchor::Same),
            offset: Offset::Backward(1),
        };

        let result = buffer.get_object_position(result.unwrap(), obj);

        assert_eq!(result.unwrap(), Position::new(8, 0));
    }

    #[test]
    fn move_two_words_right() {
        let buffer = setup_buffer("Some test content\nwith new\nlines!");
        let obj = TextObject {
            kind: Kind::Word(Anchor::Start),
            offset: Offset::Forward(2),
        };

        let result = buffer.get_object_position(Position::origin(), obj);

        assert_eq!(result.unwrap(), Position::new(10, 0));
    }

    #[test]
    fn move_two_words_left() {
        let buffer = setup_buffer("Some test content\nwith new\nlines!");
        let obj = TextObject {
            kind: Kind::Word(Anchor::Start),
            offset: Offset::Backward(2),
        };

        let result = buffer.get_object_position(Position::new(0, 1), obj);

        assert_eq!(result.unwrap(), Position::new(5, 0));
    }

    #[test]
    fn move_move_word_left_at_start_of_buffer() {
        let buffer = setup_buffer("Some test content\nwith new\nlines!");
        let obj = TextObject {
            kind: Kind::Word(Anchor::Start),
            offset: Offset::Backward(1),
        };

        let result = buffer.get_object_position(Position::new(5, 0), obj);

        assert_eq!(result.unwrap(), Position::origin());
    }

    #[test]
    fn move_move_word_right_past_end_of_buffer() {
        let buffer = setup_buffer("Some test content\nwith new\nlines!");
        let obj = TextObject {
            kind: Kind::Word(Anchor::Start),
            offset: Offset::Forward(8),
        };

        let result = buffer.get_object_position(Position::new(0, 2), obj);

        assert_eq!(result.unwrap(), Position::new(6, 2));
    }

    #[test]
    fn move_end_of_line() {
        let buffer = setup_buffer("Some test content\nwith new\nlines!");
        let obj = TextObject {
            kind: Kind::Line(Anchor::End),
            offset: Offset::Forward(0),
        };

        let result = buffer.get_object_position(Position::new(0, 1), obj);

        assert_eq!(result.unwrap(), Position::new(8, 1));
    }

    #[test]
    fn move_start_of_line() {
        let buffer = setup_buffer("Some test content\nwith new\nlines!");
        let obj = TextObject {
            kind: Kind::Line(Anchor::Start),
            offset: Offset::Backward(0),
        };

        let result = buffer.get_object_position(Position::new(1, 1), obj);

        assert_eq!(result.unwrap(), Position::new(0, 1));
    }

    #[test]
    fn move_past_last_line() {
        let buffer = setup_buffer("Some test content\n");
        let obj = TextObject {
            kind: Kind::Line(Anchor::Same),
            offset: Offset::Forward(6),
        };

        let result = buffer.get_object_position(Position::origin(), obj);

        assert_eq!(result.unwrap(), Position::new(0, 1));
    }

    #[test]
    fn move_line_up_middle_of_file() {
        let buffer = setup_buffer("Some\ntest\ncontent");
        let obj = TextObject {
            kind: Kind::Line(Anchor::Same),
            offset: Offset::Backward(1),
        };

        let result = buffer.get_object_position(Position::new(0, 2), obj);

        assert_eq!(result.unwrap(), Position::new(0, 1));
    }

    #[test]
    fn move_line_up_past_first_line() {
        let buffer = setup_buffer("Some\ntest\ncontent");
        let obj = TextObject {
            kind: Kind::Line(Anchor::Same),
            offset: Offset::Backward(1),
        };

        let result = buffer.get_object_position(Position::origin(), obj);

        assert_eq!(result.unwrap(), Position::origin());
    }

    #[test]
    fn test_insert() {
        let mut buffer = setup_buffer("ABDE");

        buffer.insert_char(Position::new(2, 0), 'C');

        assert_eq!(buffer.num_lines(), 1);
        assert_eq!(buffer.slice_full().lines().next().unwrap(), "ABCDE");
    }

    #[test]
    fn test_remove() {
        let mut buffer = setup_buffer("ABCD");

        buffer.remove(Position::new(1, 0), Position::new(2, 0));

        assert_eq!(buffer.to_string(), "ACD");
    }

    #[test]
    fn test_to_lines() {
        let buffer = setup_buffer("Test\nA\nTest");
        let mut lines = buffer.slice_full().lines();

        assert_eq!(lines.next().unwrap(), "Test\n");
        assert_eq!(lines.next().unwrap(), "A\n");
        assert_eq!(lines.next().unwrap(), "Test");
        assert!(lines.next().is_none());
    }

    #[test]
    fn test_to_lines_from() {
        let buffer = setup_buffer("Test\nA\nTest");
        let mut lines = buffer.slice_from(Position::new(1, 1)).lines();

        assert_eq!(lines.next().unwrap(), "\n");
        assert_eq!(lines.next().unwrap(), "Test");
        assert!(lines.next().is_none());
    }

    #[test]
    fn test_to_chars() {
        let buffer = setup_buffer("Test𐍈Test");
        let mut chars = buffer.slice_full().chars();
        assert!(chars.next().unwrap() == 'T');
        assert!(chars.next().unwrap() == 'e');
        assert!(chars.next().unwrap() == 's');
        assert!(chars.next().unwrap() == 't');
        assert!(chars.next().unwrap() == '𐍈');
        assert!(chars.next().unwrap() == 'T');
    }

    #[test]
    fn test_to_chars_from() {
        let buffer = setup_buffer("Test𐍈Test");
        let mut chars = buffer.slice_from(Position::new(2, 0)).chars();
        assert!(chars.next().unwrap() == 's');
        assert!(chars.next().unwrap() == 't');
        assert!(chars.next().unwrap() == '𐍈');
    }

    #[test]
    fn test_to_chars_rev() {
        let buffer = setup_buffer("Test𐍈Test");
        let mut chars = buffer.slice_to(Position::new(6, 0)).chars().rev();
        assert!(chars.next().unwrap() == 'T');
        assert!(chars.next().unwrap() == '𐍈');
        assert!(chars.next().unwrap() == 't');
    }
}
