use std::cell::RefCell;

use buffer::Line;

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

#[deriving(Clone)]
pub struct Cursor<'c> {
    pub offset: uint,
    line: Option<&'c RefCell<Line>>,
}

impl<'c> Cursor<'c> {
    /// Create a new cursor instance
    pub fn new() -> Cursor<'c> {
        Cursor {
            offset: 0,
            line: None,
        }
    }

    pub fn get_position(&self) -> (uint, uint) {
        (self.offset, self.get_linenum())
    }

    pub fn get_linenum(&self) -> uint {
        self.line.unwrap().borrow().linenum
    }

    pub fn get_offset(&self) -> uint {
        self.offset
    }

    pub fn set_offset(&mut self, offset: uint) {
        self.offset = offset;
    }

    pub fn set_line(&mut self, line: Option<&'c RefCell<Line>>) {
        self.line = line;

        // check that the current offset is longer than the length of the line
        let offset = self.get_offset();
        let line_length = self.get_line().borrow().len();
        if offset > line_length {
            self.set_offset(line_length);
        }
    }

    pub fn get_line(&self) -> &'c RefCell<Line> {
        self.line.unwrap()
    }

    pub fn get_line_length(&self) -> uint {
        self.get_line().borrow().len()
    }

    pub fn delete_backward_char(&mut self) {
        let offset = self.get_offset();
        let line = self.get_line();
        line.borrow_mut().data.remove(offset-1);
        self.set_offset(offset-1);
    }

    pub fn delete_forward_char(&mut self) {
        let offset = self.get_offset();
        let line = self.get_line();
        line.borrow_mut().data.remove(offset);
        self.set_offset(offset);
    }

    pub fn insert_char(&mut self, ch: char) {
        let offset = self.get_offset();
        let line = self.get_line();
        line.borrow_mut().data.insert(offset, ch);
        self.set_offset(offset+1)
    }

    pub fn move_right(&mut self) {
        let line_len = self.get_line().borrow().len();
        let current_offset = self.get_offset();
        if line_len > current_offset {
            self.set_offset(current_offset + 1);
        }
    }

    pub fn move_left(&mut self) {
        let current_offset = self.get_offset();
        if current_offset > 0 {
            self.set_offset(current_offset - 1);
        }
    }

    pub fn get_status_text(&self) -> String {
        let (offset, line_num) = self.get_position();
        format!("({}, {})", offset, line_num)
    }
}


#[cfg(test)]
mod tests {

    use std::cell::RefCell;

    use cursor::Cursor;
    use buffer::Line;

    fn setup_cursor<'c>() -> Cursor<'c> {
        let mut cursor = Cursor::new();
        let line = RefCell::new(Line::new("test".to_string(), 1));
        cursor.set_line(Some(&line));
        return cursor
    }

    #[test]
    fn test_moving_right() {
        let mut cursor = setup_cursor();

        assert_eq!(cursor.offset, 0);
        cursor.move_right();
        assert_eq!(cursor.offset, 1);
    }

    #[test]
    fn test_moving_left() {
        let mut cursor = setup_cursor();
        cursor.set_offset(1);

        assert_eq!(cursor.offset, 1);
        cursor.move_left();
        assert_eq!(cursor.offset, 0);
    }

    #[test]
    fn test_get_position() {
        let mut cursor = Cursor::new();
        let line = RefCell::new(Line::new("test".to_string(), 1));
        cursor.set_line(Some(&line));
        assert_eq!(cursor.get_position(), (0, 1));
    }

    #[test]
    fn test_get_linenum() {
        let mut cursor = Cursor::new();
        let line = RefCell::new(Line::new("test".to_string(), 1));
        cursor.set_line(Some(&line));

        assert_eq!(cursor.get_linenum(), 1);
    }

    #[test]
    fn test_get_offset() {
        let cursor = setup_cursor();
        assert_eq!(cursor.get_offset(), 0)
    }

    #[test]
    fn test_set_offset() {
        let mut cursor = setup_cursor();
        cursor.set_offset(3);

        assert_eq!(cursor.offset, 3);
    }


    #[test]
    fn test_moving_to_end_of_line_when_set() {
        let mut cursor = Cursor::new();
        let line = RefCell::new(Line::new("test".to_string(), 1));

        cursor.set_offset(10);
        cursor.set_line(Some(&line));

        assert_eq!(cursor.offset, 4);
    }

    #[test]
    fn test_get_line_length() {
        let mut cursor = Cursor::new();
        let line = RefCell::new(Line::new("test".to_string(), 1));

        cursor.set_line(Some(&line));

        assert_eq!(cursor.get_line_length(), 4);
    }

    #[test]
    fn test_delete_backward_char() {
        let mut cursor = Cursor::new();
        let line = RefCell::new(Line::new("test".to_string(), 1));

        cursor.set_line(Some(&line));
        cursor.set_offset(1);
        cursor.delete_backward_char();

        assert_eq!(line.borrow().data, "est".to_string());
    }

    #[test]
    fn test_delete_forward_char() {
        let mut cursor = Cursor::new();
        let line = RefCell::new(Line::new("test".to_string(), 1));

        cursor.set_line(Some(&line));
        cursor.delete_forward_char();

        assert_eq!(line.borrow().data, "est".to_string());
    }

    #[test]
    fn test_insert_char() {
        let mut cursor = Cursor::new();
        let line = RefCell::new(Line::new("test".to_string(), 1));

        cursor.set_line(Some(&line));
        cursor.insert_char('x');

        assert_eq!(line.borrow().data, "xtest".to_string());
    }

    #[test]
    fn test_get_status_text() {
        let cursor = setup_cursor();
        assert_eq!(cursor.get_status_text(), "(0, 1)".to_string());
    }

}
