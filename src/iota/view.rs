use rustbox::{Color, RustBox};

use buffer::{Buffer, Direction, Mark};
use input::Input;
use uibuf::UIBuffer;

use utils;

/// A View is an abstract Window (into a Buffer).
///
/// It draws a portion of a Buffer to a UIBuffer which in turn is drawn to the
/// screen. It maintains the status bar for the current view, the "dirty status"
/// which is whether the buffer has been modified or not and a number of other
/// pieces of information.
pub struct View<'v> {
    pub buffer: Buffer,     //Text buffer
    first_char: Mark,       //First character to be displayed
    cursor: Mark,           //Cursor displayed by this buffer.
    uibuf: UIBuffer,        //UIBuffer
}

impl<'v> View<'v> {

    //----- CONSTRUCTORS ---------------------------------------------------------------------------

    pub fn new(source: Input, rb: &RustBox) -> View<'v> {
        let mut buffer = match source {
            Input::Filename(path) => {
                match path {
                    Some(s) => Buffer::new_from_file(Path::new(s)),
                    None    => Buffer::new(),
                }
            },
            Input::Stdin(reader) => {
                Buffer::new_from_reader(reader)
            },
        };

        let height: uint = utils::get_term_height(rb);
        let width: uint = utils::get_term_width(rb);

        // NOTE(greg): this may not play well with resizing
        let uibuf = UIBuffer::new(width, height);

        let cursor = Mark::Cursor(0);
        buffer.set_mark(cursor, 0);
        let first_char = Mark::DisplayMark(0);
        buffer.set_mark(first_char, 0);

        View {
            buffer: buffer,
            first_char: first_char,
            cursor: cursor,
            uibuf: uibuf,
        }
    }

    //----- DRAWING METHODS ------------------------------------------------------------------------

    pub fn get_height(&self) -> uint {
        // NOTE(greg): when the status bar needs to move up, this value should be changed
        self.uibuf.get_height() -1
    }

    pub fn get_width(&self) -> uint {
        self.uibuf.get_width()
    }

    /// Clear the buffer
    ///
    /// Fills every cell in the UIBuffer with the space (' ') char.
    pub fn clear(&mut self, rb: &RustBox) {
        self.uibuf.fill(' ');
        self.uibuf.draw_everything(rb);
    }

    pub fn draw(&mut self, rb: &RustBox) {
        for (index,line) in self.buffer
                                .lines_from(self.first_char)
                                .unwrap()
                                .enumerate() {
            draw_line(&mut self.uibuf, line, index);
            if index == self.get_height() { break; }
        }
        self.uibuf.draw_everything(rb);
    }

    pub fn draw_status(&mut self, rb: &RustBox) {
        let buffer_status = self.buffer.status_text();
        let cursor_status = self.buffer.get_mark_coords(self.cursor);
        let status_text = format!("{} {}", buffer_status, cursor_status).into_bytes();
        let status_text_len = status_text.len();
        let width = self.get_width();
        let height = self.get_height();


        for index in range(0, width) {
            let mut ch: char = ' ';
            if index < status_text_len {
                ch = status_text[index] as char;
            }
            self.uibuf.update_cell(index, height, ch, Color::Black, Color::Blue);
        }

        self.uibuf.draw_range(rb, height, height+1);
    }

    pub fn draw_cursor(&mut self, rb: &RustBox) {
        if let Some((x, y)) = self.buffer.get_mark_coords(self.cursor) {
            utils::draw_cursor(rb, x, y);
        }
    }

    pub fn resize(&mut self, rb: &RustBox) {
        let width = self.uibuf.get_width();
        self.clear(rb);
        self.uibuf = UIBuffer::new(width, 15);
    }

    //----- CURSOR METHODS -------------------------------------------------------------------------

    pub fn move_cursor(&mut self, direction: Direction) {
        self.buffer.shift_mark(self.cursor, direction);
        self.cursor_movement();
    }

    pub fn move_cursor_to_line_end(&mut self) {
        self.buffer.shift_mark(self.cursor, Direction::LineEnd);
        self.cursor_movement();
    }

    pub fn move_cursor_to_line_start(&mut self) {
        self.buffer.shift_mark(self.cursor, Direction::LineStart);
        self.cursor_movement();
    }

    fn cursor_movement(&mut self) {

        //Update the point to be at the cursor.
        self.buffer.update_point(self.cursor);

        //Update the first_char mark if necessary to keep the cursor on the screen.
        let cursor_y = self.buffer.get_mark_coords(self.cursor).unwrap().val1();
        let first_char_y = self.buffer.get_mark_coords(self.first_char).unwrap().val1();
        if cursor_y < first_char_y {
            self.buffer.shift_mark(self.first_char, Direction::Up(1));
        } else if cursor_y > first_char_y + self.get_height() {
            self.buffer.shift_mark(self.first_char, Direction::Down(1));
        }

    }

    //----- TEXT EDIT METHODS ----------------------------------------------------------------------

    pub fn delete_char(&mut self, direction: Direction) {
        match direction {
            Direction::Left(0) => {
                self.buffer.remove_char();
                self.move_cursor(direction);
            }
            Direction::Right(0) => {
                self.move_cursor(direction);
                self.buffer.remove_char();
            }
            _ => {}
        }
    }

    pub fn insert_tab(&mut self) {
        // A tab is just 4 spaces
        for _ in range(0i, 4) {
            self.insert_char(' ');
        }
    }

    pub fn insert_char(&mut self, ch: char) {
        self.buffer.insert_char(ch as u8);
        self.move_cursor(Direction::Right(1))
    }

}

pub fn draw_line(buf: &mut UIBuffer, line: &[u8], idx: uint) {
    let width = buf.get_width() - 1;
    let mut wide_chars = 0;
    for line_idx in range(0, width) {
        if line_idx < line.len() {
            match line[line_idx] {
                b'\t'   => {
                    let w = 4 - line_idx % 4;
                    for _ in range(0, w) {
                        buf.update_cell_content(line_idx + wide_chars, idx, ' ');
                    }
                }
                b'\n'   => buf.update_cell_content(line_idx + wide_chars, idx, ' '),
                _       => buf.update_cell_content(line_idx + wide_chars, idx,
                                                   line[line_idx] as char),
            }
            wide_chars += (line[line_idx] as char).width(false).unwrap_or(1) - 1;
        } else { buf.update_cell_content(line_idx + wide_chars, idx, ' '); }
    }
    if line.len() >= width {
        buf.update_cell_content(width + wide_chars, idx, '→');
    }

}

#[cfg(test)]
mod tests {

    use buffer::{Buffer, Direction};
    use view::View;
    use uibuf::UIBuffer;
    use utils::data_from_str;

    fn setup_view<'v>() -> View<'v> {
        let mut view = View {
            buffer: Buffer::new(),
            top_line_num: 0,
            linenum: 0,
            offset: 0,
            uibuf: UIBuffer::new(50, 50),
            threshold: 5,
        };

        let first_line = Line::new(data_from_str("test"), 0);
        let second_line = Line::new(data_from_str("second"), 1);

        view.buffer.lines = vec!(first_line, second_line);
        view.set_cursor_line(0);

        return view
    }

    #[test]
    fn test_move_cursor_down() {
        let mut view = setup_view();
        view.move_cursor_down();

        assert_eq!(view.cursor().get_linenum(), 1);
        assert_eq!(view.cursor().get_line().data, data_from_str("second"));
    }

    #[test]
    fn test_move_cursor_up() {
        let mut view = setup_view();
        view.move_cursor_down();
        view.move_cursor_up();
        assert_eq!(view.cursor().get_linenum(), 0);
        assert_eq!(view.cursor().get_line().data, data_from_str("test"));
    }

    #[test]
    fn test_insert_line() {
        let mut view = setup_view();
        view.cursor().move_right();
        view.insert_line();

        assert_eq!(view.buffer.lines.len(), 3);
        assert_eq!(view.cursor().get_offset(), 0);
        assert_eq!(view.cursor().get_line().linenum, 1);
    }

    #[test]
    fn test_insert_char() {
        let mut view = setup_view();
        view.insert_char('t');

        assert_eq!(view.cursor().get_line().data, data_from_str("ttest"));
    }

    #[test]
    fn test_delete_char_to_right() {
        let mut view = setup_view();
        view.delete_char(Direction::Right);

        assert_eq!(view.cursor().get_line().data, data_from_str("est"));
    }

    #[test]
    fn test_delete_char_to_left() {
        let mut view = setup_view();
        view.cursor().move_right();
        view.delete_char(Direction::Left);

        assert_eq!(view.cursor().get_line().data, data_from_str("est"));
    }

    #[test]
    fn test_delete_char_at_start_of_line() {
        let mut view = setup_view();
        view.move_cursor_down();
        view.delete_char(Direction::Left);

        assert_eq!(view.cursor().get_line().data, data_from_str("testsecond"));
    }

    #[test]
    fn test_delete_char_at_end_of_line() {
        let mut view = setup_view();
        view.offset = 4;
        view.delete_char(Direction::Right);

        assert_eq!(view.cursor().get_line().data, data_from_str("testsecond"));
    }

    #[test]
    fn delete_char_when_line_is_empty_does_nothing() {
        let mut view = setup_view();
        view.buffer.lines = vec!(Line::new(String::new(), 0));
        view.linenum = 0;
        view.delete_char(Direction::Right);
        assert_eq!(view.cursor().get_line().data, data_from_str(""));
    }

    #[test]
    fn deleting_backward_at_start_of_first_line_does_nothing() {
        let mut view = setup_view();
        view.delete_char(Direction::Left);

        assert_eq!(view.buffer.lines.len(), 2);
        assert_eq!(view.cursor().get_line().data, data_from_str("test"));
    }
}
