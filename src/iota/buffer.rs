//FIXME: Check unicode support
// stdlib dependencies
use std::cmp;
use std::collections::HashMap;
use std::path::PathBuf;
use std::fs::File;
use std::io::{Stdin, Read};
use std::convert::From;

// external dependencies
use gapbuffer::GapBuffer;

// local dependencies
use log::{Log, Change, LogEntry};
use input::Input;
use iterators::Lines;
use textobject::{TextObject, Kind, Offset, Anchor};


#[derive(PartialEq, Debug)]
pub struct MarkPosition {
    pub absolute: usize,
    absolute_line_start: usize,
    line_number: usize,
}

impl MarkPosition {
    fn start() -> MarkPosition {
        MarkPosition {
            absolute: 0,
            line_number: 0,
            absolute_line_start: 0,
        }
    }
}

impl From<(usize, usize, usize)> for MarkPosition {
    fn from(tuple: (usize, usize, usize)) -> MarkPosition {
        let mut mark_pos = MarkPosition::start();

        mark_pos.absolute = tuple.0;
        mark_pos.absolute_line_start = tuple.1;
        mark_pos.line_number = tuple.2;

        mark_pos
    }
}


#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum Mark {
    /// For keeping track of cursors.
    Cursor(usize),

    /// For using in determining some display of characters
    DisplayMark(usize),
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum WordEdgeMatch {
    Whitespace,
}

pub struct Buffer {
    /// Current buffers text
    text: GapBuffer<u8>,

    /// Table of marked indices in the text
    marks: HashMap<Mark, MarkPosition>,

    /// Transaction history (used for undo/redo)
    pub log: Log,

    /// Location on disk where the current buffer should be written
    pub file_path: Option<PathBuf>,

    /// Whether or not the Buffer has unsaved changes
    pub dirty: bool,
}

#[cfg_attr(feature="clippy", allow(len_without_is_empty))]
impl Buffer {
    /// Constructor for empty buffer.
    pub fn new() -> Buffer {
        Buffer {
            file_path: None,
            text: GapBuffer::new(),
            marks: HashMap::new(),
            log: Log::new(),
            dirty: false,
        }
    }

    /// Length of the text stored in this buffer.
    pub fn len(&self) -> usize {
        self.text.len() + 1
    }

    /// The x,y coordinates of a mark within the file. None if not a valid mark.
    pub fn get_mark_display_coords(&self, mark: Mark) -> Option<(usize, usize)> {
        if let Some(mark_pos) = self.marks.get(&mark) {
            return Some((mark_pos.absolute - mark_pos.absolute_line_start, mark_pos.line_number))
        }

        None
    }

    /// The absolute index of a mark within the file. None if not a valid mark.
    pub fn get_mark_idx(&self, mark: Mark) -> Option<usize> {
        if let Some(mark_pos) = self.marks.get(&mark) {
            if mark_pos.absolute < self.len() {
                Some(mark_pos.absolute)
            } else { None }
        } else { None }
    }

    /// Creates an iterator on the text by lines.
    pub fn lines(&self) -> Lines {
        Lines {
            buffer: &self.text,
            tail: 0,
            head: self.len()
        }
    }

    /// Creates an iterator on the text by lines that begins at the specified mark.
    pub fn lines_from(&self, mark: Mark) -> Option<Lines> {
        if let Some(mark_pos) = self.marks.get(&mark) {
            if mark_pos.absolute < self.len() {
                return Some(Lines {
                    buffer: &self.text,
                    tail: mark_pos.absolute,
                    head: self.len(),
                })
            }
        }

        None
    }

    /// Return the buffer index of a TextObject
    pub fn get_object_index(&self, obj: TextObject) -> Option<MarkPosition> {
        match obj.kind {
            Kind::Char => self.get_char_index(obj.offset),
            Kind::Line(anchor) => self.get_line_index(obj.offset, anchor),
            Kind::Word(anchor) => self.get_word_index(obj.offset, anchor),
        }
    }

    /// Get the position of a specific character in the buffer
    ///
    /// This character can be at an absolute position, or a postion relative
    /// to a given mark.
    ///
    /// ie: get the index of the 7th character after the cursor
    /// or: get the index of the 130th character from the start of the buffer
    fn get_char_index(&self, offset: Offset) -> Option<MarkPosition> {
        let text = &self.text;

        match offset {
            // get the index of the char `offset` chars in front of `mark`
            //
            // ie: get the index of the char which is X chars in front of the MARK
            // or: get the index of the char which is 5 chars in front of the Cursor
            Offset::Forward(offset, from_mark) => {
                let last = self.len() - 1;
                if let Some(mark_pos) = self.marks.get(&from_mark) {
                    let new_absolute_position = mark_pos.absolute + offset;
                    if new_absolute_position < last {
                        // FIXME: it would be nice if we could avoid using get_line_info here...
                        let new_mark_pos = get_line_info(new_absolute_position, text).unwrap();
                        return Some(new_mark_pos)
                    } else {
                        // FIXME: it would be nice if we could avoid using get_line_info here...
                        let new_mark_pos = get_line_info(last, text).unwrap();
                        return Some(new_mark_pos)
                    }
                }

                None
            }

            // get the index of the char `offset` chars before of `mark`
            //
            // ie: get the index of the char which is X chars before the MARK
            // or: get the index of the char which is 5 chars before the Cursor
            Offset::Backward(offset, from_mark) => {
                if let Some(mark_pos) = self.marks.get(&from_mark) {
                    if mark_pos.absolute >= offset {
                        let new_absolute_position = mark_pos.absolute - offset;
                        // FIXME: it would be nice if we could avoid using get_line_info here...
                        let new_mark_pos = get_line_info(new_absolute_position, text).unwrap();
                        return Some(new_mark_pos);
                    } else {
                        return None
                    }
                }

                None
            }

            // get the index of the char at position `offset` in the buffer
            //
            // ie: get the index of the 5th char in the buffer
            Offset::Absolute(absolute_char_offset) => {
                let mut mark_pos = MarkPosition::start();
                mark_pos.absolute = absolute_char_offset;
                Some(mark_pos)
            },
        }
    }

    /// Get the position of a specific line in the buffer
    ///
    /// This line can be at an absolute position, or a postion relative
    /// to a given mark.
    ///
    /// The index is calculated based on a given Anchor. This Anchor determines
    /// where in the line the index is calculated. For instance, if you want
    /// the index of the start of the line, you would use Anchor::Start. If you
    /// are on the 5th char in a line, and want to get the index of the 5th char
    /// in another line, you can use Anchor::Same.
    ///
    /// ie: get the index of the middle of the 7th line after the cursor
    /// or: get the index of the start of the 130th line from the start of the buffer
    fn get_line_index(&self, offset: Offset, anchor: Anchor) -> Option<MarkPosition> {
        match offset {
            Offset::Forward(offset, from_mark)  => { self.get_line_index_forward(anchor, offset, from_mark) }
            Offset::Backward(offset, from_mark) => { self.get_line_index_backward(anchor, offset, from_mark) }
            Offset::Absolute(line_number)       => { self.get_line_index_absolute(anchor, line_number) }
        }
    }

    /// Get the position of the line identified by line_number
    ///
    /// ie. Get the index of Anchor inside the 23th line in the buffer
    /// or: Get the index of the start of the 23th line
    fn get_line_index_absolute(&self, anchor: Anchor, line_number: usize) -> Option<MarkPosition> {
        let text = &self.text;

        let nlines = (0..text.len()).filter(|i| text[*i] == b'\n')
                                    .take(line_number + 1)
                                    .collect::<Vec<usize>>();
        match anchor {
            Anchor::Start => {
                let mut mark_pos = MarkPosition::start();
                let line_start = nlines[0] + 1;

                mark_pos.absolute = line_start;
                mark_pos.absolute_line_start = line_start;
                mark_pos.line_number = line_number - 1;

                Some(mark_pos)
            }

            Anchor::End => {
                let mut mark_pos = MarkPosition::start();
                let end_offset = nlines[line_number - 1];

                mark_pos.absolute = end_offset;

                Some(mark_pos)
            }

            _ => {
                print!("Unhandled line anchor: {:?} ", anchor);
                None
            },
        }
    }


    fn get_line_index_backward(&self, anchor: Anchor, offset: usize, from_mark: Mark) -> Option<MarkPosition> {
        let text = &self.text;
        if let Some(mark_pos) = self.marks.get(&from_mark) {
            let nlines = (0..mark_pos.absolute).rev().filter(|i| text[*i] == b'\n')
                                         .take(offset + 1)
                                         .collect::<Vec<usize>>();

            match anchor {
                // Get the index of the start of the desired line
                Anchor::Start => {
                    let mut new_mark_pos = MarkPosition::start();

                    // if this is the first line in the buffer
                    if nlines.is_empty() {
                        return Some(new_mark_pos)
                    }

                    let start_offset = cmp::min(mark_pos.absolute - mark_pos.absolute_line_start + nlines[offset] + 1, nlines[offset]);
                    new_mark_pos.absolute = start_offset + 1;
                    new_mark_pos.line_number = nlines.len();
                    new_mark_pos.absolute_line_start = nlines[0] + 1;


                    Some(new_mark_pos)
                }

                // ie. If the current line_index is 5, then the line_index
                // returned will be the fifth index from the start of the
                // desired line.
                Anchor::Same => {
                    let mut new_mark_pos = MarkPosition::start();

                    if offset == nlines.len() {
                        new_mark_pos.absolute = cmp::min(mark_pos.absolute - mark_pos.absolute_line_start, nlines[0]);
                    } else if offset > nlines.len() || offset == 0 {
                        return Some(new_mark_pos)
                    } else {
                        new_mark_pos.absolute = cmp::min(mark_pos.absolute - mark_pos.absolute_line_start + nlines[offset] + 1, nlines[offset-1]);
                        new_mark_pos.line_number = mark_pos.line_number - offset;
                        new_mark_pos.absolute_line_start = nlines[nlines.len() - 1] + 1;
                    }

                    Some(new_mark_pos)
                }

                _ => {
                    print!("Unhandled line anchor: {:?} ", anchor);
                    None
                },
            }
        } else {
            None
        }
    }

    fn get_line_index_forward(&self, anchor: Anchor, offset: usize, from_mark: Mark) -> Option<MarkPosition> {
        let text = &self.text;
        let last = self.len() - 1;
        if let Some(mark_pos) = self.marks.get(&from_mark) {
            let nlines = (mark_pos.absolute..text.len()).filter(|i| text[*i] == b'\n')
                                            .take(offset + 1)
                                            .collect::<Vec<usize>>();
            if nlines.is_empty() { return None }

            match anchor {
                // Get the same index as the current line_index
                //
                // ie. If the current line_index is 5, then the line_index
                // returned will be the fifth index from the start of the
                // desired line.
                Anchor::Same => {
                    let mut new_pos = MarkPosition::start();
                    let new_line_start = nlines[0] + 1;

                    if offset == nlines.len() {
                        new_pos.absolute = cmp::min(mark_pos.absolute - mark_pos.absolute_line_start + nlines[offset-1] + 1, last);
                        new_pos.absolute_line_start = nlines[offset - 1] + 1;
                        new_pos.line_number = mark_pos.line_number + offset;
                    } else if offset > nlines.len() {
                        new_pos.absolute = last;
                        new_pos.line_number = (last - new_pos.absolute) + 1;
                        new_pos.absolute_line_start = new_line_start;
                    } else {
                        new_pos.absolute = cmp::min(mark_pos.absolute - mark_pos.absolute_line_start + nlines[offset-1] + 1, nlines[offset]);
                        new_pos.line_number = mark_pos.line_number + offset;
                        new_pos.absolute_line_start = new_line_start;
                    }


                    Some(new_pos)
                }

                // Get the index of the end of the desired line
                Anchor::End => {
                    // if this is the last line in the buffer
                    if nlines.is_empty() {
                        let mut new_mark_pos = MarkPosition::start();
                        new_mark_pos.absolute = last;

                        return Some(new_mark_pos)
                    }
                    let end_offset = cmp::min(mark_pos.absolute - mark_pos.absolute_line_start + nlines[offset] + 1, nlines[offset]);
                    let mut new_mark_pos = MarkPosition::start();
                    new_mark_pos.absolute = end_offset;
                    new_mark_pos.line_number = mark_pos.line_number;
                    new_mark_pos.absolute_line_start = mark_pos.absolute_line_start;

                    Some(new_mark_pos)

                }

                _ => {
                    print!("Unhandled line anchor: {:?} ", anchor);
                    None
                },
            }
        } else {
            None
        }
    }

    fn get_word_index(&self, offset: Offset, anchor: Anchor) -> Option<MarkPosition> {
        match offset {
            Offset::Forward(nth_word, from_mark)  => { self.get_word_index_forward(anchor, nth_word, from_mark) }
            Offset::Backward(nth_word, from_mark) => { self.get_word_index_backward(anchor, nth_word, from_mark) }
            Offset::Absolute(word_number)         => { self.get_word_index_absolute(anchor, word_number) }
        }
    }

    fn get_word_index_forward(&self, anchor: Anchor, nth_word: usize, from_mark: Mark) -> Option<MarkPosition> {
        let text = &self.text;
        let last = self.len() - 1;
        // TODO: use anchor to determine this
        let edger = WordEdgeMatch::Whitespace;

        if let Some(mark_pos) = self.marks.get(&from_mark) {
            match anchor {
                Anchor::Start => {
                    match get_words(mark_pos.absolute, nth_word, edger, text) {
                        Some(new_index) => {
                            let new_mark_pos = get_line_info(new_index, text).unwrap();
                            return Some(new_mark_pos);
                        }
                        None => {
                            let new_mark_pos = get_line_info(last, text).unwrap();
                            return Some(new_mark_pos);
                        }
                    }
                }

                _ => {
                    eprint!("Unhandled word anchor: {:?} ", anchor);
                    let mut new_mark_pos = MarkPosition::start();
                    new_mark_pos.absolute = last;

                    return Some(new_mark_pos);
                }
            }
        }

        None
    }

    fn get_word_index_backward(&self, anchor: Anchor, nth_word: usize, from_mark: Mark) -> Option<MarkPosition> {
        let text = &self.text;
        let last = self.len() - 1;

        // TODO: use anchor to determine this
        let edger = WordEdgeMatch::Whitespace;

        if let Some(mark_pos) = self.marks.get(&from_mark) {
            match anchor {
                Anchor::Start => {
                    // move to the start of the nth_word before the mark
                    match get_words_rev(mark_pos.absolute, nth_word, edger, text) {
                        Some(new_index) => {
                            let new_mark_pos = get_line_info(new_index, text).unwrap();
                            return Some(new_mark_pos);
                        }
                        None => {
                            return Some(MarkPosition::start());
                        }
                    }
                }

                _ => {
                    eprint!("Unhandled word anchor: {:?} ", anchor);
                    let mut new_mark_pos = MarkPosition::start();
                    new_mark_pos.absolute = last;
                    return Some(new_mark_pos);
                }
            }
        }

        None
    }

    fn get_word_index_absolute(&self, anchor: Anchor, word_number: usize) -> Option<MarkPosition> {
        let text = &self.text;
        // TODO: use anchor to determine this
        let edger = WordEdgeMatch::Whitespace;


        match anchor {
            Anchor::Start => {
                let new_index = get_words(0, word_number - 1, edger, text).unwrap();

                // let mut new_mark_pos = MarkPosition::start();
                // new_mark_pos.absolute = new_index;
                // new_mark_pos.line_start_offset = new_index - get_line(new_index, text).unwrap();
                let new_mark_pos = get_line_info(new_index, text).unwrap();

                Some(new_mark_pos)
            }

            _ => {
                print!("Unhandled word anchor: {:?} ", anchor);
                None
            }
        }
    }

    /// Returns the status text for this buffer.
    pub fn status_text(&self) -> String {
        match self.file_path {
            Some(ref path)  =>  format!("[{}] ", path.display()),
            None            =>  "untitled ".into(),
        }
    }

    /// Sets the mark to the location of a given TextObject, if it exists.
    /// Adds a new mark or overwrites an existing mark.
    pub fn set_mark_to_object(&mut self, mark: Mark, obj: TextObject) {
        if let Some(mark_pos) = self.get_object_index(obj) {
            self.set_mark(mark, mark_pos.absolute);
        }
    }

    /// Sets the mark to a given absolute index. Adds a new mark or overwrites an existing mark.
    pub fn set_mark(&mut self, mark: Mark, idx: usize) {
        if let Some(mark_pos) = get_line_info(idx, &self.text) {
            if let Some(existing_pos) = self.marks.get_mut(&mark) {
                existing_pos.absolute = mark_pos.absolute;
                existing_pos.line_number = mark_pos.line_number;
                existing_pos.absolute_line_start = mark_pos.absolute_line_start;
                return;
            }
            self.marks.insert(mark, mark_pos);
        }
    }

    // Remove the chars in the range from start to end
    pub fn remove_range(&mut self, start: usize, end: usize) -> Option<Vec<u8>> {
        self.dirty = true;
        let text = &mut self.text;
        let mut transaction = self.log.start(start);
        let mut vec = (start..end)
            .rev()
            .filter_map(|idx| text.remove(idx).map(|ch| (idx, ch)))
            .inspect(|&(idx, ch)| transaction.log(Change::Remove(idx, ch), idx))
            .map(|(_, ch)| ch)
            .collect::<Vec<u8>>();
        vec.reverse();
        Some(vec)
    }

    // Remove the chars between mark and object
    pub fn remove_from_mark_to_object(&mut self, mark: Mark, object: TextObject) -> Option<Vec<u8>> {

        let (start, end) = {
            let mark_pos = &self.marks[&mark];
            let obj_pos = self.get_object_index(object).unwrap();

            if mark_pos.absolute < obj_pos.absolute {
                (mark_pos.absolute, obj_pos.absolute)
            } else {
                (obj_pos.absolute, mark_pos.absolute)
            }
        };
        self.remove_range(start, end)
    }

    pub fn remove_object(&mut self, object: TextObject) -> Option<Vec<u8>> {
        let object_start = TextObject { kind: object.kind.with_anchor(Anchor::Start), offset: object.offset };
        let object_end = TextObject { kind: object.kind.with_anchor(Anchor::End), offset: object.offset };

        let start = self.get_object_index(object_start);
        let end = self.get_object_index(object_end);

        if let (Some(start_pos), Some(end_pos)) = (start, end) {
            return self.remove_range(start_pos.absolute, end_pos.absolute);
        }
        None
    }

    /// Insert a char at the mark.
    pub fn insert_char(&mut self, mark: Mark, ch: u8) {
        if let Some(mark_pos) = self.marks.get(&mark) {
            self.text.insert(mark_pos.absolute, ch);
            let mut transaction = self.log.start(mark_pos.absolute);
            transaction.log(Change::Insert(mark_pos.absolute, ch), mark_pos.absolute);
            self.dirty = true;
        }
    }

    /// Redo most recently undone action.
    pub fn redo(&mut self) -> Option<&LogEntry> {
        if let Some(transaction) = self.log.redo() {
            commit(transaction, &mut self.text);
            Some(transaction)
        } else { None }
    }

    /// Undo most recently performed action.
    pub fn undo(&mut self) -> Option<&LogEntry> {
        if let Some(transaction) = self.log.undo() {
            commit(transaction, &mut self.text);
            Some(transaction)
        } else { None }
    }

}


// This is a bit of a hack to get around an error I was getting when
// implementing From<R: Read> for Buffer with From<PathBuf> for Buffer.
// The compiler was telling me this was a conflicting implementation even
// though Read is not implemented for PathBuf. Changing R: Read to
// R: Read + BufferFrom fixes the error.
//
// TODO: investigate this further - possible compiler bug?
pub trait BufferFrom {}
impl BufferFrom for Stdin {}
impl BufferFrom for File {}

impl From<PathBuf> for Buffer {
    fn from(path: PathBuf) -> Buffer {
        match File::open(&path) {
            Ok(file) => {
                let mut buf = Buffer::from(file);
                buf.file_path = Some(path);
                buf
            }
            Err(_) => {
                Buffer::new()
            }
        }
    }
}

impl<R: Read + BufferFrom> From<R> for Buffer {
    fn from(mut reader: R) -> Buffer {
        let mut buff = Buffer::new();
        let mut contents = String::new();
        if reader.read_to_string(&mut contents).is_ok() {
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
            // (&WordEdgeMatch::Alphabet, c1, c2) if c1.is_whitespace() => !c2.is_whitespace(),
            // (&WordEdgeMatch::Alphabet, c1, c2) if is_alpha_or_(c1) => !is_alpha_or_(c2) && !c2.is_whitespace(),
            // (&WordEdgeMatch::Alphabet, c1, c2) if !is_alpha_or_(c1) => is_alpha_or_(c2) && !c2.is_whitespace(),
            // (&WordEdgeMatch::Alphabet, _, _) => false,
        }
    }
}

fn get_words(mark: usize, n_words: usize, edger: WordEdgeMatch, text: &GapBuffer<u8>) -> Option<usize> {
    let text_len = text.len();
    if text_len == 0 { return None; }

    (mark + 1..text_len - 1)
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

fn get_line_info(mark: usize, text: &GapBuffer<u8>) -> Option<MarkPosition> {
    let val = cmp::min(mark, text.len());
    let line_starts: Vec<usize> = (0..val + 1).rev().filter(|idx| *idx == 0 || text[*idx - 1] == b'\n').collect();


    if line_starts.is_empty() {
        None
    } else {
        let mut mark_pos = MarkPosition::start();
        mark_pos.absolute_line_start = line_starts[0];
        mark_pos.line_number = line_starts.len() - 1;
        mark_pos.absolute = mark;
        Some(mark_pos)
    }

}

/// Performs a transaction on the passed in buffer.
fn commit(transaction: &LogEntry, text: &mut GapBuffer<u8>) {
    for change in &transaction.changes {
        match *change {
            Change::Insert(idx, ch) => {
                text.insert(idx, ch);
            }
            Change::Remove(idx, _) => {
                text.remove(idx);
            }
        }
    }
}

#[cfg(test)]
mod test {

    use buffer::{Buffer, Mark, MarkPosition};
    use textobject::{TextObject, Offset, Kind, Anchor};
    use super::get_line_info;

    fn setup_buffer(testcase: &'static str) -> Buffer {
        let mut buffer = Buffer::new();
        buffer.text.extend(testcase.bytes());
        buffer.set_mark(Mark::Cursor(0), 0);
        buffer
    }

    #[test]
    fn move_mark_char_right() {
        let mut buffer = setup_buffer("Some test content");
        let mark = Mark::Cursor(0);
        let obj = TextObject {
            kind: Kind::Char,
            offset: Offset::Forward(1, mark),
        };

        buffer.set_mark_to_object(mark, obj);

        assert_eq!(*buffer.marks.get(&mark).unwrap(), MarkPosition::from((1, 0, 0)));
        assert_eq!(buffer.get_mark_display_coords(mark).unwrap(), (1, 0));
    }

    #[test]
    fn move_mark_char_left() {
        let mut buffer = setup_buffer("Some test content");
        let mark = Mark::Cursor(0);
        let obj = TextObject {
            kind: Kind::Char,
            offset: Offset::Backward(1, mark),
        };

        buffer.set_mark(mark, 3);
        buffer.set_mark_to_object(mark, obj);

        assert_eq!(*buffer.marks.get(&mark).unwrap(), MarkPosition::from((2, 0, 0)));
        assert_eq!(buffer.get_mark_display_coords(mark).unwrap(), (2, 0));
    }

    #[test]
    fn move_mark_five_chars_right() {
        let mut buffer = setup_buffer("Some test content");
        let mark = Mark::Cursor(0);
        let obj = TextObject {
            kind: Kind::Char,
            offset: Offset::Forward(5, mark),
        };

        buffer.set_mark_to_object(mark, obj);

        assert_eq!(*buffer.marks.get(&mark).unwrap(), MarkPosition::from((5, 0, 0)));
        assert_eq!(buffer.get_mark_display_coords(mark).unwrap(), (5, 0));
    }

    #[test]
    fn move_mark_line_down() {
        let mut buffer = setup_buffer("Some test content\nwith new\nlines!");
        let mark = Mark::Cursor(0);
        let obj = TextObject {
            kind: Kind::Line(Anchor::Same),
            offset: Offset::Forward(1, mark),
        };

        buffer.set_mark_to_object(mark, obj);

        assert_eq!(*buffer.marks.get(&mark).unwrap(), MarkPosition::from((18, 18, 1)));
        assert_eq!(buffer.get_mark_display_coords(mark).unwrap(), (0, 1));
    }

    #[test]
    fn move_mark_line_up() {
        let mut buffer = setup_buffer("Some test content\nnew lines!");
        let mark = Mark::Cursor(0);
        let obj = TextObject {
            kind: Kind::Line(Anchor::Same),
            offset: Offset::Backward(1, mark),
        };

        buffer.set_mark(mark, 18);
        buffer.set_mark_to_object(mark, obj);

        assert_eq!(*buffer.marks.get(&mark).unwrap(), MarkPosition::from((0, 0, 0)));
        assert_eq!(buffer.get_mark_display_coords(mark).unwrap(), (0, 0));
    }

    #[test]
    fn move_mark_two_lines_down() {
        let mut buffer = setup_buffer("Some test content\nwith new\nlines!");
        let mark = Mark::Cursor(0);
        let obj = TextObject {
            kind: Kind::Line(Anchor::Same),
            offset: Offset::Forward(2, mark),
        };

        buffer.set_mark_to_object(mark, obj);

        assert_eq!(*buffer.marks.get(&mark).unwrap(), MarkPosition::from((27, 27, 2)));
        assert_eq!(buffer.get_mark_display_coords(mark).unwrap(), (0, 2));
    }

    // FIXME: re-enable this
    #[test]
    #[ignore]
    fn move_mark_line_down_to_shorter_line() {
        let mut buffer = setup_buffer("Some test content\nwith new\nlines!");
        let mark = Mark::Cursor(0);
        let obj = TextObject {
            kind: Kind::Line(Anchor::Same),
            offset: Offset::Forward(1, mark),
        };

        buffer.set_mark(mark, 15);
        buffer.set_mark_to_object(mark, obj);

        assert_eq!(*buffer.marks.get(&mark).unwrap(), MarkPosition::from((26, 8, 1)));
        assert_eq!(buffer.get_mark_display_coords(mark).unwrap(), (8, 1));

        let obj = TextObject {
            kind: Kind::Line(Anchor::Same),
            offset: Offset::Backward(1, mark),
        };
        buffer.set_mark_to_object(mark, obj);

        // NOTE: this test could be wrong...
        assert_eq!(*buffer.marks.get(&mark).unwrap(), MarkPosition::from((15, 15, 0)));
        assert_eq!(buffer.get_mark_display_coords(mark).unwrap(), (15, 0));
    }

    #[test]
    fn move_mark_two_words_right() {
        let mut buffer = setup_buffer("Some test content\nwith new\nlines!");
        let mark = Mark::Cursor(0);
        let obj = TextObject {
            kind: Kind::Word(Anchor::Start),
            offset: Offset::Forward(2, mark),
        };

        buffer.set_mark_to_object(mark, obj);

        assert_eq!(*buffer.marks.get(&mark).unwrap(), MarkPosition::from((10, 0, 0)));
        assert_eq!(buffer.get_mark_display_coords(mark).unwrap(), (10, 0));
    }

    #[test]
    fn move_mark_two_words_left() {
        let mut buffer = setup_buffer("Some test content\nwith new\nlines!");
        let mark = Mark::Cursor(0);
        let obj = TextObject {
            kind: Kind::Word(Anchor::Start),
            offset: Offset::Backward(2, mark),
        };

        buffer.set_mark(mark, 18);
        buffer.set_mark_to_object(mark, obj);

        assert_eq!(*buffer.marks.get(&mark).unwrap(), MarkPosition::from((5, 0, 0)));
        assert_eq!(buffer.get_mark_display_coords(mark).unwrap(), (5, 0));
    }

    #[test]
    fn move_mark_move_word_left_at_start_of_buffer() {
        let mut buffer = setup_buffer("Some test content\nwith new\nlines!");
        let mark = Mark::Cursor(0);
        let obj = TextObject {
            kind: Kind::Word(Anchor::Start),
            offset: Offset::Backward(1, mark),
        };

        buffer.set_mark(mark, 5);
        buffer.set_mark_to_object(mark, obj);

        assert_eq!(*buffer.marks.get(&mark).unwrap(), MarkPosition::from((0, 0, 0)));
        assert_eq!(buffer.get_mark_display_coords(mark).unwrap(), (0, 0));
    }

    #[test]
    fn move_mark_move_word_right_past_end_of_buffer() {
        let mut buffer = setup_buffer("Some test content\nwith new\nlines!");
        let mark = Mark::Cursor(0);
        let obj = TextObject {
            kind: Kind::Word(Anchor::Start),
            offset: Offset::Forward(8, mark),
        };

        buffer.set_mark(mark, 28);
        buffer.set_mark_to_object(mark, obj);

        assert_eq!(*buffer.marks.get(&mark).unwrap(), MarkPosition::from((33, 27, 2)));
        assert_eq!(buffer.get_mark_display_coords(mark).unwrap(), (6, 2));
    }

    #[test]
    fn move_mark_second_word_in_buffer() {
        let mut buffer = setup_buffer("Some test content\nwith new\nlines!");
        let mark = Mark::Cursor(0);
        let obj = TextObject {
            kind: Kind::Word(Anchor::Start),
            offset: Offset::Absolute(2),
        };

        buffer.set_mark(mark, 18);
        buffer.set_mark_to_object(mark, obj);

        assert_eq!(*buffer.marks.get(&mark).unwrap(), MarkPosition::from((5, 0, 0)));
        assert_eq!(buffer.get_mark_display_coords(mark).unwrap(), (5, 0));
    }

    #[test]
    fn move_mark_fifth_word_in_buffer() {
        let mut buffer = setup_buffer("Some test content\nwith new\nlines!");
        let mark = Mark::Cursor(0);
        let obj = TextObject {
            kind: Kind::Word(Anchor::Start),
            offset: Offset::Absolute(5),
        };

        buffer.set_mark(mark, 18);
        buffer.set_mark_to_object(mark, obj);

        assert_eq!(*buffer.marks.get(&mark).unwrap(), MarkPosition::from((23, 18, 1)));
        assert_eq!(buffer.get_mark_display_coords(mark).unwrap(), (5, 1));
    }

    #[test]
    fn move_mark_second_line_in_buffer() {
        let mut buffer = setup_buffer("Some test content\nwith new\nlines!");
        let mark = Mark::Cursor(0);
        let obj = TextObject {
            kind: Kind::Line(Anchor::Start),
            offset: Offset::Absolute(2),
        };

        buffer.set_mark_to_object(mark, obj);

        assert_eq!(*buffer.marks.get(&mark).unwrap(), MarkPosition::from((18, 18, 1)));
        assert_eq!(buffer.get_mark_display_coords(mark).unwrap(), (0, 1));
    }

    #[test]
    fn move_mark_second_char_in_buffer() {
        let mut buffer = setup_buffer("Some test content\nwith new\nlines!");
        let mark = Mark::Cursor(0);
        let obj = TextObject {
            kind: Kind::Char,
            offset: Offset::Absolute(2),
        };

        buffer.set_mark(mark, 18);
        buffer.set_mark_to_object(mark, obj);

        assert_eq!(*buffer.marks.get(&mark).unwrap(), MarkPosition::from((2, 0, 0)));
        assert_eq!(buffer.get_mark_display_coords(mark).unwrap(), (2, 0));
    }

    #[test]
    fn move_mark_end_of_line() {
        let mut buffer = setup_buffer("Some test content\nwith new\nlines!");
        let mark = Mark::Cursor(0);
        let obj = TextObject {
            kind: Kind::Line(Anchor::End),
            offset: Offset::Forward(0, mark),
        };

        buffer.set_mark(mark, 19);
        buffer.set_mark_to_object(mark, obj);

        assert_eq!(*buffer.marks.get(&mark).unwrap(), MarkPosition::from((26, 18, 1)));
        assert_eq!(buffer.get_mark_display_coords(mark).unwrap(), (8, 1));
    }

    #[test]
    fn move_mark_start_of_line() {
        let mut buffer = setup_buffer("Some test content\nwith new\nlines!");
        let mark = Mark::Cursor(0);
        let obj = TextObject {
            kind: Kind::Line(Anchor::Start),
            offset: Offset::Backward(0, mark),
        };

        buffer.set_mark(mark, 19);
        buffer.set_mark_to_object(mark, obj);

        assert_eq!(*buffer.marks.get(&mark).unwrap(), MarkPosition::from((18, 18, 1)));
        assert_eq!(buffer.get_mark_display_coords(mark).unwrap(), (0, 1));
    }

    #[test]
    fn move_mark_past_last_line() {
        let mut buffer = setup_buffer("Some test content\n");
        let mark = Mark::Cursor(0);
        let obj = TextObject {
            kind: Kind::Line(Anchor::Same),
            offset: Offset::Forward(6, mark),
        };

        buffer.set_mark_to_object(mark, obj);

        assert_eq!(*buffer.marks.get(&mark).unwrap(), MarkPosition::from((18, 18, 1)));
        assert_eq!(buffer.get_mark_display_coords(mark).unwrap(), (0, 1));
    }

    #[test]
    fn move_mark_line_up_middle_of_file() {
        let mut buffer = setup_buffer("Some\ntest\ncontent");
        let mark = Mark::Cursor(0);
        let obj = TextObject {
            kind: Kind::Line(Anchor::Same),
            offset: Offset::Backward(1, mark),
        };

        buffer.set_mark(mark, 10);
        buffer.set_mark_to_object(mark, obj);

        assert_eq!(*buffer.marks.get(&mark).unwrap(), MarkPosition::from((5, 5, 1)));
        assert_eq!(buffer.get_mark_display_coords(mark).unwrap(), (0, 1));
    }

    #[test]
    fn move_mark_line_up_past_first_line() {
        let mut buffer = setup_buffer("Some\ntest\ncontent");
        let mark = Mark::Cursor(0);
        let obj = TextObject {
            kind: Kind::Line(Anchor::Same),
            offset: Offset::Backward(1, mark),
        };

        buffer.set_mark_to_object(mark, obj);

        assert_eq!(*buffer.marks.get(&mark).unwrap(), MarkPosition::from((0, 0, 0)));
        assert_eq!(buffer.get_mark_display_coords(mark).unwrap(), (0, 0));
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
        let mark = Mark::Cursor(0);
        let obj = TextObject {
            kind: Kind::Char,
            offset: Offset::Forward(1, mark)
        };
        buffer.remove_from_mark_to_object(mark, obj);

        assert_eq!(buffer.len(), 4);
        assert_eq!(buffer.lines().next().unwrap(), [b'B', b'C', b'D']);
    }

    #[test]
    fn test_set_mark() {
        let mut buffer = setup_buffer("Test");
        buffer.set_mark(Mark::Cursor(0), 2);

        assert_eq!(buffer.get_mark_idx(Mark::Cursor(0)).unwrap(), 2);
        assert_eq!(buffer.marks.get(&Mark::Cursor(0)).unwrap().absolute, 2);
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
    fn test_line_down() {
        let mut buffer = setup_buffer("Some\ntest\ncontent");
        let mark = Mark::Cursor(0);
        let obj = TextObject {
            kind: Kind::Line(Anchor::Same),
            offset: Offset::Forward(1, mark),
        };

        buffer.set_mark_to_object(mark, obj);

        assert_eq!(*buffer.marks.get(&mark).unwrap(), MarkPosition::from((5, 5, 1)));
        assert_eq!(buffer.get_mark_display_coords(mark).unwrap(), (0, 1));

        let obj = TextObject {
            kind: Kind::Char,
            offset: Offset::Forward(1, mark),
        };

        buffer.set_mark_to_object(mark, obj);

        assert_eq!(*buffer.marks.get(&mark).unwrap(), MarkPosition::from((6, 5, 1)));
        assert_eq!(buffer.get_mark_display_coords(mark).unwrap(), (1, 1));
    }

    #[test]
    fn test_get_line_info() {
        let mut buffer = setup_buffer("Test\nA\nTest");
        buffer.set_mark(Mark::Cursor(0), 10);

        let mark_pos = MarkPosition::from((10, 7, 2));

        assert_eq!(mark_pos, get_line_info(10, &buffer.text).unwrap());
    }

    #[test]
    fn get_move_word_forward_emtpy_buffer() {
        let mut buffer = setup_buffer("");
        let mark = Mark::Cursor(0);
        buffer.set_mark(Mark::Cursor(0), 0);

        let obj = TextObject {
            kind: Kind::Word(Anchor::Start),
            offset: Offset::Forward(1, mark),
        };

        buffer.set_mark_to_object(mark, obj);

        assert_eq!(*buffer.marks.get(&mark).unwrap(), MarkPosition::from((0, 0, 0)));
        assert_eq!(buffer.get_mark_display_coords(mark).unwrap(), (0, 0));
    }

}
