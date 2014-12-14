use buffer::Line;

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

pub struct Cursor<'c> {
    /// The number of bytes the cursor is along the line. This must always be on a character
    /// boundary.
    pub offset: uint,
    line: &'c mut Line,
}

impl<'c> Cursor<'c> {
    /// Create a new cursor instance
    pub fn new(line: &'c mut Line, offset: uint) -> Cursor<'c> {
        let mut cursor = Cursor {
            offset: offset,
            line: line,
        };

        // check that the current offset is longer than the length of the line
        let offset = cursor.get_offset();
        let line_length = cursor.get_line().len();
        if offset > line_length {
            cursor.set_offset(line_length);
        }
        cursor
    }

    pub fn get_position(&self) -> (uint, uint) {
        (self.offset, self.get_linenum())
    }

    pub fn get_linenum(&self) -> uint {
        self.line.linenum
    }

    pub fn get_offset(&self) -> uint {
        self.offset
    }

    pub fn get_visible_offset(&self) -> uint {
        self.get_line().data.slice_to(self.get_offset()).width(false)
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
        let range = self.get_line().data.char_range_at(self.offset);
        self.set_offset(range.next);
    }

    /// Moves the cursor back one character.
    ///
    /// See `inc_offset` for why this method is needed.
    pub fn dec_offset(&mut self) {
        let range = self.get_line().data.char_range_at_reverse(self.offset);
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

    pub fn delete_backward_char(&mut self) {
        let offset = self.get_offset();
        let range = self.get_line().data.char_range_at_reverse(self.offset);
        let back = self.offset - range.next;
        self.dec_offset();
        self.get_line_mut().data.remove(offset-back);
    }

    pub fn delete_forward_char(&mut self) {
        let offset = self.get_offset();
        self.get_line_mut().data.remove(offset);
    }

    pub fn insert_char(&mut self, ch: char) {
        let offset = self.get_offset();
        self.get_line_mut().data.insert(offset, ch);
        self.inc_offset();
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
        let (offset, line_num) = self.get_position();
        format!("({}, {})", offset, line_num)
    }
}


#[cfg(test)]
mod tests {

    use cursor::Cursor;
    use buffer::Line;
    use utils::data_from_str;

    fn setup_cursor<F>(mut f: F) where F: FnMut(Cursor) {
        let ref mut line = Line::new(data_from_str("test"), 1);
        let cursor = Cursor::new(line, 0);
        f(cursor);
    }

    #[test]
    fn test_moving_right() {
        setup_cursor( |mut cursor| {
            assert_eq!(cursor.offset, 0);
            cursor.move_right();
            assert_eq!(cursor.offset, 1);
        });
    }

    #[test]
    fn test_moving_left() {
        setup_cursor( |mut cursor| {
            cursor.set_offset(1);

            assert_eq!(cursor.offset, 1);
            cursor.move_left();
            assert_eq!(cursor.offset, 0);
        });
    }

    #[test]
    fn test_get_position() {
        let ref mut line = Line::new(data_from_str("test"), 1);
        let cursor = Cursor::new(line, 0);
        assert_eq!(cursor.get_position(), (0, 1));
    }

    #[test]
    fn test_get_linenum() {
        let ref mut line = Line::new(data_from_str("test"), 1);
        let cursor = Cursor::new(line, 0);

        assert_eq!(cursor.get_linenum(), 1);
    }

    #[test]
    fn test_get_offset() {
        setup_cursor( |cursor| {
            assert_eq!(cursor.get_offset(), 0)
        });
    }

    #[test]
    fn test_set_offset() {
        setup_cursor( |mut cursor| {
            cursor.set_offset(3);

            assert_eq!(cursor.offset, 3);
        });
    }


    #[test]
    fn test_moving_to_end_of_line_when_set() {
        let ref mut line = Line::new(data_from_str("test"), 1);
        let cursor = Cursor::new(line, 10);

        assert_eq!(cursor.offset, 4);
    }

    #[test]
    fn test_get_line_length() {
        let ref mut line = Line::new(data_from_str("test"), 1);
        let cursor = Cursor::new(line, 0);

        assert_eq!(cursor.get_line_length(), 4);
    }

    #[test]
    fn test_delete_backward_char() {
        let ref mut line = Line::new(data_from_str("test"), 1);

        {
            let mut cursor = Cursor::new(line, 1);
            cursor.delete_backward_char();
        }

        assert_eq!(line.data, data_from_str("est"));
    }

    #[test]
    fn test_delete_forward_char() {
        let ref mut line = Line::new(data_from_str("test"), 1);

        {
            let mut cursor = Cursor::new(line, 0);
            cursor.delete_forward_char();
        }

        assert_eq!(line.data, data_from_str("est"));
    }

    #[test]
    fn test_insert_char() {
        let ref mut line = Line::new(data_from_str("test"), 1);

        {
            let mut cursor = Cursor::new(line, 0);
            cursor.insert_char('x');
        }

        assert_eq!(line.data, data_from_str("xtest"));
    }

    #[test]
    fn test_get_status_text() {
        setup_cursor( |cursor| {
            assert_eq!(cursor.get_status_text(), "(0, 1)".to_string());
        } );
    }

}
