use buffer::Line;
use log::{Change, Transaction};
use std::cmp;

#[deriving(Copy, Show)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    pub fn is_right(&self) -> bool {
        match *self {
            Direction::Right => true,
            _                => false
        }
    }
    pub fn is_left(&self) -> bool {
        match *self {
            Direction::Left => true,
            _               => false
        }
    }
}

#[deriving(Copy,PartialEq,Show)]
pub struct CursorData {
    /// The current line number of the cursor
    pub linenum: uint,
    /// The current offset of the cursor
    pub offset: uint,
}

pub struct Cursor<'c> {
    /// The number of bytes the cursor is along the line. This must always be on a character
    /// boundary.
    pub offset: uint,
    line: &'c mut Line,
}

impl<'c> Cursor<'c> {
    /// Create a new cursor instance
    pub fn new(line: &'c mut Line, offset: uint) -> Cursor<'c> {
        Cursor {
            offset: offset,
            line: line,
        }
    }

    pub fn get_position(&self) -> CursorData {
        CursorData {
            linenum: self.get_linenum(),
            offset: self.get_offset(),
        }
    }

    pub fn get_linenum(&self) -> uint {
        self.line.linenum
    }

    pub fn get_offset(&self) -> uint {
        cmp::min(self.line.len(), self.offset)
    }

    pub fn get_actual_offset(&self) -> uint {
        self.offset
    }

    pub fn get_visible_offset(&self) -> uint {
        ::utils::str_width(self.get_line().data.slice_to(self.get_offset()), false, 4)
    }

    pub fn set_offset(&mut self, offset: uint) {
        self.offset = offset;
    }

    /// Moves the cursor forward one character.
    ///
    /// This can’t simply be `self.offset += 1`, because not all UTF-8 codepoints are exactly one
    /// byte long. This function calculates the width of the current codepoint and increments the
    /// offset by that width, ensuring that the cursor will always be on a character boundary.
    pub fn inc_offset(&mut self) {
        let range = self.get_line().data.char_range_at(self.get_offset());
        self.set_offset(range.next);
    }

    /// Moves the cursor back one character.
    ///
    /// See `inc_offset` for why this method is needed.
    pub fn dec_offset(&mut self) {
        let range = self.get_line().data.char_range_at_reverse(self.get_offset());
        self.set_offset(range.next);
    }

    pub fn get_line(&self) -> &Line {
        &*self.line
    }

    pub fn get_line_mut(&mut self) -> &mut Line {
        self.line
    }

    pub fn get_line_length(&self) -> uint {
        self.get_line().len()
    }

    pub fn delete_backward_char(&mut self, transaction: &mut Transaction) {
        self.move_left();
        self.delete_forward_char(transaction);
    }

    pub fn delete_forward_char(&mut self, transaction: &mut Transaction) {
        let offset = self.get_offset();
        let old_line = self.get_line_mut().clone();
        self.get_line_mut().data.remove(offset);
        transaction.log(Change::Update(old_line, self.line.data.clone()), self.get_position());
    }

    pub fn insert_char(&mut self, transaction: &mut Transaction, ch: char) {
        let offset = self.get_offset();
        let old_line = self.get_line_mut().clone();
        self.get_line_mut().data.insert(offset, ch);
        self.move_right();
        transaction.log(Change::Update(old_line, self.line.data.clone()), self.get_position());
    }

    pub fn move_right(&mut self) {
        let line_len = self.get_line().len();
        let current_offset = self.get_offset();
        if line_len > current_offset {
            self.inc_offset();
        }
    }

    pub fn move_left(&mut self) {
        let current_offset = self.get_offset();
        if current_offset > 0 {
            self.dec_offset();
        }
    }

    pub fn get_status_text(&self) -> String {
        let CursorData { offset, linenum } = self.get_position();
        format!("({}, {})", offset + 1, linenum + 1)
    }
}


#[cfg(test)]
mod tests {

    use cursor::{Cursor, CursorData};
    use buffer::Line;
    use log::{LogEntries, Transaction};
    use utils::data_from_str;

    fn setup_cursor<F>(mut f: F) where F: FnMut(Cursor, Transaction) {
        let ref mut line = Line::new(data_from_str("test"), 1);
        let cursor = Cursor::new(line, 0);
        let mut log = LogEntries::new();
        let position = cursor.get_position();
        f(cursor, log.start(position));
    }

    #[test]
    fn test_moving_right() {
        setup_cursor( |mut cursor, _| {
            assert_eq!(cursor.offset, 0);
            cursor.move_right();
            assert_eq!(cursor.offset, 1);
        });
    }

    #[test]
    fn test_moving_left() {
        setup_cursor( |mut cursor, _| {
            cursor.set_offset(1);

            assert_eq!(cursor.offset, 1);
            cursor.move_left();
            assert_eq!(cursor.offset, 0);
        });
    }

    #[test]
    fn test_get_position() {
        setup_cursor( |cursor, _| {
            assert_eq!(cursor.get_position(), CursorData {
                linenum: 1,
                offset: 0
            });
        });
    }

    #[test]
    fn test_get_linenum() {
        setup_cursor( |cursor, _| {
            assert_eq!(cursor.get_linenum(), 1);
        });
    }

    #[test]
    fn test_get_offset() {
        setup_cursor( |cursor, _| {
            assert_eq!(cursor.get_offset(), 0)
        });
    }

    #[test]
    fn test_set_offset() {
        setup_cursor( |mut cursor, _| {
            cursor.set_offset(3);

            assert_eq!(cursor.offset, 3);
        });
    }


    #[test]
    fn test_moving_to_end_of_line_when_set() {
        let ref mut line = Line::new(data_from_str("test"), 1);
        let cursor = Cursor::new(line, 10);

        assert_eq!(cursor.offset, 10);
        assert_eq!(cursor.get_offset(), 4);
    }

    #[test]
    fn test_get_line_length() {
        setup_cursor( |cursor, _| {
            assert_eq!(cursor.get_line_length(), 4);
        });
    }

    #[test]
    fn test_delete_backward_char() {
        setup_cursor( |mut cursor, ref mut transaction| {
            cursor.offset = 1;
            cursor.delete_backward_char(transaction);
            assert_eq!(cursor.line.data, data_from_str("est"));
        });
    }

    #[test]
    fn test_delete_forward_char() {
        setup_cursor( |mut cursor, ref mut transaction| {
            cursor.delete_forward_char(transaction);
            assert_eq!(cursor.line.data, data_from_str("est"));
        });
    }

    #[test]
    fn test_insert_char() {
        setup_cursor( |mut cursor, ref mut transaction| {
            cursor.insert_char(transaction, 'x');
            assert_eq!(cursor.line.data, data_from_str("xtest"));
        });
    }

    #[test]
    fn test_get_status_text() {
        setup_cursor( |cursor, _| {
            assert_eq!(cursor.get_status_text(), "(1, 2)".to_string());
        } );
    }

    #[test]
    fn test_tab_width() {
        let ref mut line = Line::new(data_from_str("a\ttest"), 1);
        let cursor = Cursor::new(line, 2);
        assert_eq!(cursor.get_visible_offset(), 4)
    }

}
