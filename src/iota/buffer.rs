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
use strings::rope::Rope;

// local dependencies
use log::{Log, Change, LogEntry};
use utils::is_alpha_or_;
use input::Input;
use iterators::{Lines, Chars};
use textobject::{TextObject, Kind, Offset, Anchor};


#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum Mark {
    /// For keeping track of cursors.
    Cursor(usize),

    /// For using in determining some display of characters
    DisplayMark(usize),
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum WordEdgeMatch {
    Alphabet,
    Whitespace,
}

pub struct Buffer {
    /// Current buffers text
    text: Rope,

    /// Table of marked indices in the text
    /// KEY: mark id => VALUE : (absolute index, line index)
    ///
    /// - absolute index is the offset from the start of the buffer
    /// - line index is the offset from the start of the current line
    marks: HashMap<Mark, (usize, usize)>,

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
            text: Rope::new(),
            marks: HashMap::new(),
            log: Log::new(),
        }
    }

    /// Length of the text stored in this buffer.
    pub fn len(&self) -> usize {
        self.text.len() + 1
    }

    /// The x,y coordinates of a mark within the file. None if not a valid mark.
    pub fn get_mark_coords(&self, mark: Mark) -> Option<(usize, usize)> {
        if let Some(idx) = self.get_mark_idx(mark) {
            if let Some(line) = get_line(idx, &self.text) {
                // let offset  = (0..idx).filter(|i| -> bool { self.text[*i] == b'\n' })
                //                                .count();

                // FIXME: shouldn't need this
                if self.text.len() == 0 {
                    return Some((0, 0))
                }

                let chars: Vec<u8> = self.text.chars().map(|(ch, idx)| ch as u8).collect();
                let offset  = (0..idx).filter(|i| -> bool { chars[*i] == b'\n' })
                                               .count();

                Some((idx - line, offset))
            } else { None }
        } else { None }
    }

    /// The absolute index of a mark within the file. None if not a valid mark.
    pub fn get_mark_idx(&self, mark: Mark) -> Option<usize> {
        if let Some(&(idx, _)) = self.marks.get(&mark) {
            if idx < self.len() {
                Some(idx)
            } else { None }
        } else { None }
    }

    /// Creates an iterator on the text by chars.
    // pub fn chars(&self) -> Chars {
    //     Chars {
    //         buffer: &self.text,
    //         idx: 0,
    //         forward: true,
    //     }
    // }

    /// Creates an iterator on the text by chars that begins at the specified index.
    pub fn chars_from_idx(&self, idx: usize) -> Option<Chars> {
        if idx < self.len() {
            Some(Chars {
                buffer: &self.text,
                idx: idx,
                forward: true,
            })
        } else { None }
    }

    /// Creates an iterator on the text by chars that begins at the specified mark.
    // pub fn chars_from(&self, mark: Mark) -> Option<Chars> {
    //     if let Some(&(idx, _)) = self.marks.get(&mark) {
    //         self.chars_from_idx(idx)
    //     } else { None }
    // }

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
        if let Some(&(idx, _)) = self.marks.get(&mark) {
            if idx < self.len() {
                Some(Lines {
                    buffer: &self.text,
                    tail: idx,
                    head: self.len(),
                })
            } else { None }
        } else { None }
    }

    /// Return the buffer index of a TextObject
    pub fn get_object_index(&self, obj: TextObject) -> Option<(usize, usize)> {
        match obj.kind {
            Kind::Char => self.get_char_index(obj.offset),
            Kind::Line(anchor) => self.get_line_index(obj.offset, anchor),
            Kind::Word(anchor) => self.get_word_index(obj.offset, anchor),
        }
    }

    /// Get the absolute index of a specific character in the buffer
    ///
    /// This character can be at an absolute position, or a postion relative
    /// to a given mark.
    ///
    /// The absolute offset is in the form (index, line_index) where:
    ///     index = the offset from the start of the buffer
    ///     line_index = the offset from the start of the current line
    ///
    /// ie: get the index of the 7th character after the cursor
    /// or: get the index of the 130th character from the start of the buffer
    fn get_char_index(&self, offset: Offset) -> Option<(usize, usize)> {
        let text = &self.text;

        match offset {
            // get the index of the char `offset` chars in front of `mark`
            //
            // ie: get the index of the char which is X chars in front of the MARK
            // or: get the index of the char which is 5 chars in front of the Cursor
            Offset::Forward(offset, from_mark) => {
                let last = self.len() - 1;
                if let Some(tuple)= self.marks.get(&from_mark) {
                    let (index, _) = *tuple;
                    let absolute_index = index + offset;
                    if absolute_index < last {
                        Some((absolute_index, absolute_index - get_line(absolute_index, text).unwrap()))
                    } else {
                        Some((last, last - get_line(last, text).unwrap()))
                    }
                } else {
                    None
                }
            }

            // get the index of the char `offset` chars before of `mark`
            //
            // ie: get the index of the char which is X chars before the MARK
            // or: get the index of the char which is 5 chars before the Cursor
            Offset::Backward(offset, from_mark) => {
                if let Some(tuple)= self.marks.get(&from_mark) {
                    let (index, _) = *tuple;
                    if index >= offset {
                        let absolute_index = index - offset;
                        Some((absolute_index, absolute_index - get_line(absolute_index, text).unwrap()))
                    } else {
                        None
                    }
                } else {
                    None
                }
            }

            // get the index of the char at position `offset` in the buffer
            //
            // ie: get the index of the 5th char in the buffer
            Offset::Absolute(absolute_char_offset) => {
                Some((absolute_char_offset, absolute_char_offset - get_line(absolute_char_offset, text).unwrap()))
            },
        }
    }

    /// Get the absolute index of a specific line in the buffer
    ///
    /// This line can be at an absolute position, or a postion relative
    /// to a given mark.
    ///
    /// The absolute offset is in the form (index, line_index) where:
    ///     index = the offset from the start of the buffer
    ///     line_index = the offset from the start of the current line
    ///
    /// The index is calculated based on a given Anchor. This Anchor determines
    /// where in the line the index is calculated. For instance, if you want
    /// the index of the start of the line, you would use Anchor::Start. If you
    /// are on the 5th char in a line, and want to get the index of the 5th char
    /// in another line, you can use Anchor::Same.
    ///
    /// ie: get the index of the middle of the 7th line after the cursor
    /// or: get the index of the start of the 130th line from the start of the buffer
    fn get_line_index(&self, offset: Offset, anchor: Anchor) -> Option<(usize, usize)> {
        match offset {
            Offset::Forward(offset, from_mark)  => { self.get_line_index_forward(anchor, offset, from_mark) }
            Offset::Backward(offset, from_mark) => { self.get_line_index_backward(anchor, offset, from_mark) }
            Offset::Absolute(line_number)       => { self.get_line_index_absolute(anchor, line_number) }
        }
    }

    /// Get the index of the line identified by line_number
    ///
    /// ie. Get the index of Anchor inside the 23th line in the buffer
    /// or: Get the index of the start of the 23th line
    fn get_line_index_absolute(&self, anchor: Anchor, line_number: usize) -> Option<(usize, usize)> {
        let chars = self.text.chars();

        // let nlines = (0..text.len()).filter(|i| text[*i] == b'\n')
        //                             .take(line_number + 1)
        //                             .collect::<Vec<usize>>();
        let nlines = chars.filter(|&(ch, _index)| ch == '\n')
                          .take(line_number + 1)
                          .map(|(_ch, index)| { index })
                          .collect::<Vec<usize>>();

        match anchor {
            Anchor::Start => {
                let end_offset = nlines[line_number - 1];
                let start = get_line(end_offset, &self.text).unwrap();
                Some((start, 0))
            }

            Anchor::End => {
                let end_offset = nlines[line_number - 1];
                Some((end_offset, end_offset))
            }

            _ => {
                print!("Unhandled line anchor: {:?} ", anchor);
                None
            },
        }
    }


    fn get_line_index_backward(&self, anchor: Anchor, offset: usize, from_mark: Mark) -> Option<(usize, usize)> {
        let text = &self.text;
        let chars: Vec<(char, usize)> = self.text.chars().collect();

        if let Some(tuple) = self.marks.get(&from_mark) {
            let (index, line_index) = *tuple;

            let nlines = chars.into_iter()
                              .rev()
                              .filter(|&(ch, _index)| ch == '\n')
                              .take(offset + 1)
                              .map(|(_ch, index)| { index })
                              .collect::<Vec<usize>>();

            // let nlines = (0..index).rev().filter(|i| text[*i] == b'\n')
            //                              .take(offset + 1)
            //                              .collect::<Vec<usize>>();

            match anchor {
                // Get the index of the start of the desired line
                Anchor::Start => {
                    // if this is the first line in the buffer
                    if nlines.len() == 0 {
                        return Some((0, 0))
                    }
                    let start_offset = cmp::min(line_index + nlines[offset] + 1, nlines[offset]);
                    Some((start_offset + 1, 0))
                }

                // ie. If the current line_index is 5, then the line_index
                // returned will be the fifth index from the start of the
                // desired line.
                Anchor::Same => {
                    if offset == 0 {
                        Some((0, 0)) // going to start of the first line
                    } else if offset == nlines.len() {
                        Some((cmp::min(line_index, nlines[0]), line_index))
                    } else if offset > nlines.len() {
                        Some((0, 0)) // trying to move up from the first line
                    } else {
                        Some((cmp::min(line_index + nlines[offset] + 1, nlines[offset-1]), line_index))
                    }
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

    fn get_line_index_forward(&self, anchor: Anchor, offset: usize, from_mark: Mark) -> Option<(usize, usize)> {
        let text = &self.text;
        let last = self.len() - 1;
        let chars = self.text.chars();

        if let Some(tuple) = self.marks.get(&from_mark) {
            let (index, line_index) = *tuple;
            let nlines = chars.filter(|&(ch, _index)| ch == '\n')
                              .take(offset + 1)
                              .map(|(_ch, index)| { index })
                              .collect::<Vec<usize>>();

            // let nlines = (index..text.len()).filter(|i| text[*i] == b'\n')
            //                                 .take(offset + 1)
            //                                 .collect::<Vec<usize>>();

            match anchor {
                // Get the same index as the current line_index
                //
                // ie. If the current line_index is 5, then the line_index
                // returned will be the fifth index from the start of the
                // desired line.
                Anchor::Same => {
                    if offset == nlines.len() {
                        Some((cmp::min(line_index + nlines[offset-1] + 1, last), line_index))
                    } else {
                        if offset > nlines.len() {
                            Some((last, last - get_line(last, text).unwrap()))
                        } else {
                            Some((cmp::min(line_index + nlines[offset-1] + 1, nlines[offset]), line_index))
                        }
                    }
                }

                // Get the index of the end of the desired line
                Anchor::End => {
                    // if this is the last line in the buffer
                    if nlines.len() == 0 {
                        return Some((last, offset))
                    }
                    let end_offset = cmp::min(line_index + nlines[offset] + 1, nlines[offset]);
                    Some((end_offset, end_offset - get_line(end_offset, text).unwrap()))
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

    fn get_word_index(&self, offset: Offset, anchor: Anchor) -> Option<(usize, usize)> {
        match offset {
            // Offset::Forward(nth_word, from_mark)  => { self.get_word_index_forward(anchor, nth_word, from_mark) }
            // Offset::Backward(nth_word, from_mark) => { self.get_word_index_backward(anchor, nth_word, from_mark) }
            // Offset::Absolute(word_number)         => { self.get_word_index_absolute(anchor, word_number) }

            _ => {None}
        }
    }

    // fn get_word_index_forward(&self, anchor: Anchor, nth_word: usize, from_mark: Mark) -> Option<(usize, usize)> {
    //     let text = &self.text;
    //     let last = self.len() - 1;
    //     // TODO: use anchor to determine this
    //     let edger = WordEdgeMatch::Whitespace;
    //
    //
    //     if let Some(tuple) = self.marks.get(&from_mark) {
    //         let (index, _) = *tuple;
    //         match anchor {
    //             Anchor::Start => {
    //                 // move to the start of nth_word from the mark
    //                 if let Some(new_index) = get_words(index, nth_word, edger, text) {
    //                     Some((new_index, new_index - get_line(new_index, text).unwrap()))
    //                 } else {
    //                     Some((last, last - get_line(last, text).unwrap()))
    //                 }
    //             }
    //
    //             _ => {
    //                 print!("Unhandled word anchor: {:?} ", anchor);
    //                 Some((last, last - get_line(last, text).unwrap()))
    //             }
    //         }
    //     } else {
    //         None
    //     }
    // }

    // fn get_word_index_backward(&self, anchor: Anchor, nth_word: usize, from_mark: Mark) -> Option<(usize, usize)> {
    //     let text = &self.text;
    //     // TODO: use anchor to determine this
    //     let edger = WordEdgeMatch::Whitespace;
    //
    //
    //     if let Some(tuple) = self.marks.get(&from_mark) {
    //         let (index, _) = *tuple;
    //         match anchor {
    //             Anchor::Start => {
    //                 // move to the start of the nth_word before the mark
    //                 if let Some(new_index) = get_words_rev(index, nth_word, edger, text) {
    //                     Some((new_index, new_index - get_line(new_index, text).unwrap()))
    //                 } else {
    //                     Some((0, 0))
    //                 }
    //             }
    //
    //             _ => {
    //                 print!("Unhandled word anchor: {:?} ", anchor);
    //                 None
    //             },
    //         }
    //     } else {
    //         None
    //     }
    // }

    // fn get_word_index_absolute(&self, anchor: Anchor, word_number: usize) -> Option<(usize, usize)> {
    //     let text = &self.text;
    //     // TODO: use anchor to determine this
    //     let edger = WordEdgeMatch::Whitespace;
    //
    //
    //     match anchor {
    //         Anchor::Start => {
    //             let new_index = get_words(0, word_number - 1, edger, text).unwrap();
    //
    //             Some((new_index, new_index - get_line(new_index, text).unwrap()))
    //         }
    //
    //         _ => {
    //             print!("Unhandled word anchor: {:?} ", anchor);
    //             None
    //         }
    //     }
    // }

    /// Returns the status text for this buffer.
    pub fn status_text(&self) -> String {
        match self.file_path {
            Some(ref path)  =>  format!("[{}] ", path.display()),
            None            =>  format!("untitled "),
        }
    }

    /// Sets the mark to the location of a given TextObject, if it exists.
    /// Adds a new mark or overwrites an existing mark.
    pub fn set_mark_to_object(&mut self, mark: Mark, obj: TextObject) {
        if let Some(tuple) = self.get_object_index(obj) {
            self.marks.insert(mark, tuple);
        }
    }

    /// Sets the mark to a given absolute index. Adds a new mark or overwrites an existing mark.
    pub fn set_mark(&mut self, mark: Mark, idx: usize) {
        if let Some(line) = get_line(idx, &self.text) {
            if let Some(tuple) = self.marks.get_mut(&mark) {
                *tuple = (idx, idx - line);
                return;
            }
            self.marks.insert(mark, (idx, idx - line));
        }
    }

    // Remove the chars in the range from start to end
    pub fn remove_range(&mut self, start: usize, end: usize)  {
        // let text = &mut self.text;
        // let mut transaction = self.log.start(start);
        // let mut vec = (start..end)
        //     .rev()
        //     .filter_map(|idx| text.remove(idx, end).map(|ch| (idx, ch)))
        //     .inspect(|&(idx, ch)| transaction.log(Change::Remove(idx, ch), idx))
        //     .map(|(_, ch)| ch)
        //     .collect::<Vec<String>>();
        // vec.reverse();
        // Some(vec)
    }

    // Remove the chars between mark and object
    pub fn remove_from_mark_to_object(&mut self, mark: Mark, object: TextObject) {
        if let Some(&(mark_idx, _)) = self.marks.get(&mark) {
            let object_index = self.get_object_index(object);

            if let Some((obj_idx, _)) = object_index {
                if mark_idx != obj_idx {
                    let (start, end) = if mark_idx < obj_idx { (mark_idx, obj_idx) } else { (obj_idx, mark_idx) };
                    self.remove_range(start, end);
                    return;
                }
            }
        }
        // None
    }

    pub fn remove_object(&mut self, object: TextObject) {
        let object_start = TextObject { kind: object.kind.with_anchor(Anchor::Start), offset: object.offset };
        let object_end = TextObject { kind: object.kind.with_anchor(Anchor::End), offset: object.offset };

        let start = self.get_object_index(object_start);
        let end = self.get_object_index(object_end);

        if let (Some((start_index, _)), Some((end_index, _))) = (start, end) {
            return self.remove_range(start_index, end_index);
        }
        // None
    }

    /// Insert a char at the mark.
    pub fn insert_char(&mut self, mark: Mark, ch: char) {
        if let Some(&(idx, _)) = self.marks.get(&mark) {
            let mut data = String::new();
            data.push(ch);
            self.text.insert_copy(idx, &*data);
            let mut transaction = self.log.start(idx);
            transaction.log(Change::Insert(idx, ch as u8), idx);
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
            buff.text.insert(0, contents);
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

// fn get_words(mark: usize, n_words: usize, edger: WordEdgeMatch, text: &Rope) -> Option<usize> {
//     // (mark + 1..text.len() - 1)
//     //     .filter(|idx| edger.is_word_edge(&text[*idx - 1], &text[*idx]))
//     //     .take(n_words)
//     //     .last()
//
//     let chars: Vec<(char, usize)> = text.chars()
//                                .filter(|&(ch, idx)| {
//                                    idx > mark &&
//                                    edger.is_word_edge(&(chars[idx-1].0 as u8), &(ch as u8))
//                                })
//                             //    .map(|(ch, idx)| ch)
//                       .filter(|&(ch, idx)| {
//                       })
//                       .take(n_words).last();
//     match result {
//         Some((ch, idx)) => Some(idx),
//         None => None,
//     }
// }

// fn get_words_rev(mark: usize, n_words: usize, edger: WordEdgeMatch, text: &Rope) -> Option<usize> {
//     // (1..mark)
//     //     .rev()
//     //     .filter(|idx| edger.is_word_edge(&text[*idx - 1], &text[*idx]))
//     //     .take(n_words)
//     //     .last()
//
//     let chars = text.chars().collect::<Vec<(char, usize)>>();
//
//     let result = chars.into_iter().rev()
//                       .filter(|&(ch, idx)| {
//                           idx <= mark && idx >=1 &&
//                           edger.is_word_edge(&(chars[idx-1].0 as u8), &(ch as u8))
//                       })
//                       .take(n_words)
//                       .last();
//
//     match result {
//         Some((ch, idx)) => Some(idx),
//         None => None,
//     }
// }

/// Returns the index of the first character of the line the mark is in.
/// Newline prior to mark (EXCLUSIVE) + 1.
fn get_line(mark: usize, text: &Rope) -> Option<usize> {
    let val = cmp::min(mark, text.len());

    // FIXME: shouldn't need this?
    if text.len() == 0 {
        return Some(0)
    }

    let chars: Vec<(char, usize)> = text.chars().collect();

    // FIXME: this would be better if Rope allowed for char slices
    //        e.g. text.char_range(1..4).filter(...)
    (0..val + 1).rev().filter(|idx| *idx == 0 || chars[*idx - 1].0 == '\n')
        .take(1).next()
}

/// Returns the index of the newline character at the end of the line mark is in.
/// Newline after mark (INCLUSIVE).
// fn get_line_end(mark: usize, text: &GapBuffer<u8>) -> Option<usize> {
//     let val = cmp::min(mark, text.len());
//     (val..text.len()+1).filter(|idx| *idx == text.len() || text[*idx] == b'\n')
//                             .take(1)
//                             .next()
// }

/// Performs a transaction on the passed in buffer.
fn commit(transaction: &LogEntry, text: &mut Rope) {
    for change in transaction.changes.iter() {
        match change {
            &Change::Insert(idx, ch) => {
                let mut data = String::new();
                data.push(ch as char);
                text.insert(idx, data);
            }
            &Change::Remove(idx, _) => {
                // FIXME: this could be wrong, perhaps idx-1 ?
                text.remove(idx, idx+1);
            }
        }
    }
}

#[cfg(test)]
mod test {

    use buffer::{Buffer, Mark};
    use textobject::{TextObject, Offset, Kind, Anchor};

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

        assert_eq!(buffer.marks.get(&mark).unwrap(), &(1, 1));
        assert_eq!(buffer.get_mark_coords(mark).unwrap(), (1, 0));
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

        assert_eq!(buffer.marks.get(&mark).unwrap(), &(2, 2));
        assert_eq!(buffer.get_mark_coords(mark).unwrap(), (2, 0));
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

        assert_eq!(buffer.marks.get(&mark).unwrap(), &(5, 5));
        assert_eq!(buffer.get_mark_coords(mark).unwrap(), (5, 0));
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

        assert_eq!(buffer.marks.get(&mark).unwrap(), &(18, 0));
        assert_eq!(buffer.get_mark_coords(mark).unwrap(), (0, 1));
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

        assert_eq!(buffer.marks.get(&mark).unwrap(), &(0, 0));
        assert_eq!(buffer.get_mark_coords(mark).unwrap(), (0, 0));
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

        assert_eq!(buffer.marks.get(&mark).unwrap(), &(27, 0));
        assert_eq!(buffer.get_mark_coords(mark).unwrap(), (0, 2));
    }

    #[test]
    fn move_mark_line_down_to_shorter_line() {
        let mut buffer = setup_buffer("Some test content\nwith new\nlines!");
        let mark = Mark::Cursor(0);
        let obj = TextObject {
            kind: Kind::Line(Anchor::Same),
            offset: Offset::Forward(1, mark),
        };

        buffer.set_mark(mark, 15);
        buffer.set_mark_to_object(mark, obj);

        assert_eq!(buffer.marks.get(&mark).unwrap(), &(26, 15));
        assert_eq!(buffer.get_mark_coords(mark).unwrap(), (8, 1));

        let obj = TextObject {
            kind: Kind::Line(Anchor::Same),
            offset: Offset::Backward(1, mark),
        };
        buffer.set_mark_to_object(mark, obj);

        assert_eq!(buffer.marks.get(&mark).unwrap(), &(15, 15));
        assert_eq!(buffer.get_mark_coords(mark).unwrap(), (15, 0));
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

        assert_eq!(buffer.marks.get(&mark).unwrap(), &(10, 10));
        assert_eq!(buffer.get_mark_coords(mark).unwrap(), (10, 0));
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

        assert_eq!(buffer.marks.get(&mark).unwrap(), &(5, 5));
        assert_eq!(buffer.get_mark_coords(mark).unwrap(), (5, 0));
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

        assert_eq!(buffer.marks.get(&mark).unwrap(), &(0, 0));
        assert_eq!(buffer.get_mark_coords(mark).unwrap(), (0, 0));
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

        assert_eq!(buffer.marks.get(&mark).unwrap(), &(33, 6));
        assert_eq!(buffer.get_mark_coords(mark).unwrap(), (6, 2));
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

        assert_eq!(buffer.marks.get(&mark).unwrap(), &(5, 5));
        assert_eq!(buffer.get_mark_coords(mark).unwrap(), (5, 0));
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

        assert_eq!(buffer.marks.get(&mark).unwrap(), &(23, 5));
        assert_eq!(buffer.get_mark_coords(mark).unwrap(), (5, 1));
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

        assert_eq!(buffer.marks.get(&mark).unwrap(), &(18, 0));
        assert_eq!(buffer.get_mark_coords(mark).unwrap(), (0, 1));
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

        assert_eq!(buffer.marks.get(&mark).unwrap(), &(2, 2));
        assert_eq!(buffer.get_mark_coords(mark).unwrap(), (2, 0));
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

        assert_eq!(buffer.marks.get(&mark).unwrap(), &(26, 8));
        assert_eq!(buffer.get_mark_coords(mark).unwrap(), (8, 1));
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

        assert_eq!(buffer.marks.get(&mark).unwrap(), &(18, 0));
        assert_eq!(buffer.get_mark_coords(mark).unwrap(), (0, 1));
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

        assert_eq!(buffer.marks.get(&mark).unwrap(), &(18, 0));
        assert_eq!(buffer.get_mark_coords(mark).unwrap(), (0, 1));
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

        assert_eq!(buffer.marks.get(&mark).unwrap(), &(5, 0));
        assert_eq!(buffer.get_mark_coords(mark).unwrap(), (0, 1));
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

        assert_eq!(buffer.marks.get(&mark).unwrap(), &(0, 0));
        assert_eq!(buffer.get_mark_coords(mark).unwrap(), (0, 0));
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
    fn test_to_chars() {
        let mut buffer = setup_buffer("Test𐍈Test");
        buffer.set_mark(Mark::Cursor(0), 0);
        let mut chars = buffer.chars();
        assert!(chars.next().unwrap() == 'T');
        assert!(chars.next().unwrap() == 'e');
        assert!(chars.next().unwrap() == 's');
        assert!(chars.next().unwrap() == 't');
        assert!(chars.next().unwrap() == '𐍈');
        assert!(chars.next().unwrap() == 'T');
    }

    #[test]
    fn test_to_chars_from() {
        let mut buffer = setup_buffer("Test𐍈Test");
        buffer.set_mark(Mark::Cursor(0), 2);
        let mut chars = buffer.chars_from(Mark::Cursor(0)).unwrap();
        assert!(chars.next().unwrap() == 's');
        assert!(chars.next().unwrap() == 't');
        assert!(chars.next().unwrap() == '𐍈');
    }

    #[test]
    fn test_to_chars_rev() {
        // 𐍈 encodes as utf8 in 4 bytes... we need a solution for buffer offsets by byte/char
        let mut buffer = setup_buffer("Test𐍈Test");
        buffer.set_mark(Mark::Cursor(0), 8);
        let mut chars = buffer.chars_from(Mark::Cursor(0)).unwrap().reverse();
        assert!(chars.next().unwrap() == 'T');
        assert!(chars.next().unwrap() == '𐍈');
        assert!(chars.next().unwrap() == 't');
    }

}
