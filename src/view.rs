use cursor::Direction;
use buffer::Buffer;
use cursor::Cursor;

use utils;

/// A View is an abstract Window (into a Buffer).
///
/// It draws a portion of a Buffer to a UIBuffer which in turn is drawn to the
/// screen. It maintains the status bar for the current view, the "dirty status"
/// which is whether the buffer has been modified or not and a number of other
/// pieces of information.
pub struct View<'v> {
    pub buffer: Buffer<'v>,
    pub top_line_num: uint,
    pub cursor: Cursor<'v>,

    threshold: int,
}

impl<'v> View<'v> {
    pub fn new(path: &Path) -> View<'v> {
        let buffer = Buffer::new_from_file(path);
        let mut cursor = Cursor::new();
        cursor.set_line(Some(&buffer.lines[0]));

        View {
            buffer: buffer,
            top_line_num: 0,
            cursor: cursor,
            threshold: 5,
        }
    }

    /// Get the height of the view in which content can be drawn
    ///
    /// Excludes the status bar height
    pub fn get_internal_height(&self) -> uint {
        let term_height = utils::get_term_height();

        term_height - 2
    }

    pub fn draw(&self) {
        let height = self.get_internal_height();

        let num_lines = self.buffer.lines.len();
        let lines_to_draw = self.buffer.lines.slice(self.top_line_num, num_lines);

        for (index, line) in lines_to_draw.iter().enumerate() {
            if index <= height {
                let ln = line.borrow();
                utils::draw(index, ln.data.clone());
            }
        }
    }

    pub fn draw_status(&self) {
        let buffer_status = self.buffer.get_status_text();
        let cursor_status = self.cursor.get_status_text();
        let term_height = utils::get_term_height();

        let status_text = format!("{} {}", buffer_status, cursor_status);

        utils::draw(term_height-1, status_text);
    }

    pub fn draw_cursor(&self) {
        let offset = self.cursor.get_offset();
        let linenum = self.cursor.get_linenum();

        utils::draw_cursor(offset, linenum-self.top_line_num);
    }

    pub fn move_cursor(&mut self, direction: Direction) {
        match direction {
            Direction::Up    => { self.move_cursor_up(); },
            Direction::Down  => { self.move_cursor_down(); },
            Direction::Right => { self.cursor.move_right(); },
            Direction::Left  => { self.cursor.move_left(); },
        }
    }

    // TODO(greg): refactor this method with move_cursor_down
    pub fn move_cursor_up(&mut self) {
        let cursor_linenum = self.cursor.get_linenum();
        let prev_linenum = cursor_linenum - 1;

        let num_lines = self.buffer.lines.len() - 1;
        if prev_linenum > num_lines { return }

        {
            let line = &self.buffer.lines[prev_linenum];
            self.cursor.set_line(Some(line));
        }

        let cursor_linenum = self.cursor.get_linenum() as int;
        let cursor_offset = cursor_linenum - self.top_line_num as int;

        if cursor_offset < self.threshold {
            let times = cursor_offset - self.threshold;
            self.move_top_line_n_times(times);
        }

    }

    // TODO(greg): refactor this method with move_cursor_up
    pub fn move_cursor_down(&mut self) {
        let cursor_linenum = self.cursor.get_linenum();
        let next_linenum = cursor_linenum + 1;

        let num_lines = self.buffer.lines.len() - 1;
        if next_linenum > num_lines { return }

        {
            let line = &self.buffer.lines[next_linenum];
            self.cursor.set_line(Some(line));
        }

        let cursor_linenum = self.cursor.get_linenum() as int;
        let cursor_offset = cursor_linenum - self.top_line_num as int;
        let height = self.get_internal_height() as int;

        // TODO(greg) move this value
        // let threshold: int = 5;

        if cursor_offset >= (height - self.threshold) {
            let times = cursor_offset - (height - self.threshold) + 1;
            self.move_top_line_n_times(times);
        }
    }

    fn move_top_line_n_times(&mut self, mut num_times: int) {
        if num_times == 0 { return }

        // moving down
        if num_times > 0 {
            while num_times > 0 {
                self.top_line_num += 1;
                num_times -= 1;
            }
            return
        }

        // moving up
        if num_times < 0 {
            while num_times < 0 {
                if self.top_line_num == 0 { return }
                self.top_line_num -= 1;
                num_times += 1;
            }
            return
        }
    }

    pub fn delete_char(&mut self, direction: Direction) {
        let (offset, line_num) = self.cursor.get_position();

        if offset == 0 && direction.is_left() {
            let offset = self.buffer.join_line_with_previous(offset, line_num);
            self.move_cursor_up();
            self.cursor.set_offset(offset);
            return
        }

        let line_len = self.cursor.get_line_length();
        if offset == line_len && direction.is_right() {
            self.buffer.join_line_with_previous(offset, line_num+1);
            return
        }

        match direction {
            Direction::Left  => self.cursor.delete_backward_char(),
            Direction::Right => self.cursor.delete_forward_char(),
            _                => {}
        }
    }

    pub fn insert_char(&mut self, ch: char) {
        self.cursor.insert_char(ch);
    }

    pub fn insert_line(&mut self) {
        let (offset, line_num) = self.cursor.get_position();
        self.buffer.insert_line(offset, line_num);

        self.move_cursor_down();
        self.cursor.set_offset(0);
    }
}

#[cfg(test)]
mod tests {

    use std::cell::RefCell;

    use buffer::{Line, Buffer};
    use cursor::{Cursor, Direction};
    use view::View;
    use utils;

    fn setup_view<'v>() -> View<'v> {
        let mut view = View {
            buffer: Buffer::new(),
            top_line_num: 0,
            cursor: Cursor::new(),
            threshold: 5,
        };

        let first_line = RefCell::new(Line::new("test".to_string(), 0));
        let second_line = RefCell::new(Line::new("second".to_string(), 1));

        view.buffer.lines = vec!(first_line, second_line);
        view.cursor.set_line(Some(&view.buffer.lines[0]));

        return view
    }

    #[test]
    fn test_move_cursor_down() {
        let mut view = setup_view();
        view.move_cursor_down();

        assert_eq!(view.cursor.get_linenum(), 1);
        assert_eq!(view.cursor.get_line().borrow().data, "second".to_string());
    }

    #[test]
    fn test_move_cursor_up() {
        let mut view = setup_view();
        view.move_cursor_down();
        view.move_cursor_up();
        assert_eq!(view.cursor.get_linenum(), 0);
        assert_eq!(view.cursor.get_line().borrow().data, "test".to_string());
    }

    #[test]
    fn test_insert_line() {
        let mut view = setup_view();
        view.cursor.move_right();
        view.insert_line();

        assert_eq!(view.buffer.lines.len(), 3);
        assert_eq!(view.cursor.get_offset(), 0);
        assert_eq!(view.cursor.get_line().borrow().linenum, 1);
    }

    #[test]
    fn test_insert_char() {
        let mut view = setup_view();
        view.insert_char('t');

        assert_eq!(view.cursor.get_line().borrow().data, "ttest".to_string());
    }

    #[test]
    fn test_delete_char_to_right() {
        let mut view = setup_view();
        view.delete_char(Direction::Right);

        assert_eq!(view.cursor.get_line().borrow().data, "est".to_string());
    }

    #[test]
    fn test_delete_char_to_left() {
        let mut view = setup_view();
        view.cursor.move_right();
        view.delete_char(Direction::Left);

        assert_eq!(view.cursor.get_line().borrow().data, "est".to_string());
    }

    #[test]
    fn test_delete_char_at_start_of_line() {
        let mut view = setup_view();
        view.move_cursor_down();
        view.delete_char(Direction::Left);

        assert_eq!(view.cursor.get_line().borrow().data, "testsecond".to_string());
    }

    #[test]
    fn test_delete_char_at_end_of_line() {
        let mut view = setup_view();
        view.cursor.set_offset(4);
        view.delete_char(Direction::Right);

        assert_eq!(view.cursor.get_line().borrow().data, "testsecond".to_string());
    }

    #[test]
    fn test_get_internal_height() {
        let view = setup_view();
        let term_height = utils::get_term_height();
        assert_eq!(view.get_internal_height(), term_height - 2);
    }
}
