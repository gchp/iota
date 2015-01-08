use std::cmp;

use buffer::{Buffer, Direction, Mark};
use input::Input;
use uibuf::{UIBuffer, CharColor};
use frontends::Frontend;
use overlay::{Overlay, OverlayType};
use utils;

/// A View is an abstract Window (into a Buffer).
///
/// It draws a portion of a Buffer to a UIBuffer which in turn is drawn to the
/// screen. It maintains the status bar for the current view, the "dirty status"
/// which is whether the buffer has been modified or not and a number of other
/// pieces of information.
pub struct View<'v> {
    pub buffer: Buffer,
    pub overlay: Overlay,

    /// First character of the top line to be displayed
    top_line: Mark,

    /// Index into the top_line - used for horizontal scrolling
    left_col: uint,

    /// The current View's cursor - a reference into the Buffer
    cursor: Mark,

    /// The UIBuffer to which the View is drawn
    uibuf: UIBuffer,

    /// Number of lines from the top/bottom of the View after which vertical
    /// scrolling begins.
    threshold: uint,
}

impl<'v> View<'v> {

    pub fn new(source: Input, width: uint, height: uint) -> View<'v> {
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

        let cursor = Mark::Cursor(0);
        let top_line = Mark::DisplayMark(0);

        buffer.set_mark(cursor, 0);
        buffer.set_mark(top_line, 0);

        View {
            buffer: buffer,
            top_line: top_line,
            left_col: 0,
            cursor: cursor,
            uibuf: UIBuffer::new(width, height),
            overlay: Overlay::None,
            threshold: 5,
        }
    }

    /// Get the height of the View.
    ///
    /// This is the height of the UIBuffer minus the status bar height.
    pub fn get_height(&self) -> uint {
        let status_bar_height = 1;
        let uibuf_height = self.uibuf.get_height();

        uibuf_height - status_bar_height
    }

    /// Get the width of the View.
    pub fn get_width(&self) -> uint {
        self.uibuf.get_width()
    }

    /// Resize the view
    ///
    /// This involves simply changing the size of the associated UIBuffer
    pub fn resize(&mut self, width: uint, height: uint) {
        let uibuf = UIBuffer::new(width, height);
        self.uibuf = uibuf;
    }

    /// Clear the buffer
    ///
    /// Fills every cell in the UIBuffer with the space (' ') char.
    pub fn clear<T: Frontend>(&mut self, frontend: &mut T) {
        self.uibuf.fill(' ');
        self.uibuf.draw_everything(frontend);
    }

    pub fn draw<T: Frontend>(&mut self, frontend: &mut T) {
        for (index,line) in self.buffer
                                .lines_from(self.top_line)
                                .unwrap()
                                .take(self.get_height())
                                .enumerate() {
            draw_line(&mut self.uibuf, line, index, self.left_col);
            if index == self.get_height() { break; }
        }

        match self.overlay {
            Overlay::None => self.draw_cursor(frontend),
            _ => {
                self.overlay.draw(frontend, &mut self.uibuf);
                self.overlay.draw_cursor(frontend);
            }
        }
        self.draw_status(frontend);
        self.uibuf.draw_everything(frontend);
    }

    fn draw_status<T: Frontend>(&mut self, frontend: &mut T) {
        let buffer_status = self.buffer.status_text();
        let mut cursor_status = self.buffer.get_mark_coords(self.cursor).unwrap_or((0,0));
        cursor_status = (cursor_status.0 + 1, cursor_status.1 + 1);
        let status_text = format!("{} {}", buffer_status, cursor_status).into_bytes();
        let status_text_len = status_text.len();
        let width = self.get_width();
        let height = self.get_height() - 1;


        for index in range(0, width) {
            let mut ch: char = ' ';
            if index < status_text_len {
                ch = status_text[index] as char;
            }
            self.uibuf.update_cell(index, height, ch, CharColor::Black, CharColor::Blue);
        }

        self.uibuf.draw_range(frontend, height, height+1);
    }

    fn draw_cursor<T: Frontend>(&mut self, frontend: &mut T) {
        if let Some(top_line) = self.buffer.get_mark_coords(self.top_line) {
            if let Some((x, y)) = self.buffer.get_mark_coords(self.cursor) {
                frontend.draw_cursor((x - self.left_col) as int, y as int - top_line.1 as int);
            }
        }
    }

    pub fn set_overlay(&mut self, overlay_type: OverlayType) {
        match overlay_type {
            OverlayType::Prompt => {
                self.overlay = Overlay::Prompt {
                    cursor_x: 1,
                    prefix: ":",
                    data: String::new(),
                };
            }
        }
    }

    pub fn move_cursor(&mut self, direction: Direction, amount: uint) {
        self.buffer.shift_mark(self.cursor, direction, amount);
        self.maybe_move_screen();
    }

    pub fn move_cursor_to_line_end(&mut self) {
        self.buffer.shift_mark(self.cursor, Direction::LineEnd, 0);
        self.maybe_move_screen();
    }

    pub fn move_cursor_to_line_start(&mut self) {
        self.buffer.shift_mark(self.cursor, Direction::LineStart, 0);
        self.maybe_move_screen();
    }

    /// Update the top_line mark if necessary to keep the cursor on the screen.
    fn maybe_move_screen(&mut self) {
        if let (Some(cursor), Some((_, top_line))) = (self.buffer.get_mark_coords(self.cursor),
                                                      self.buffer.get_mark_coords(self.top_line)) {

            let width  = (self.get_width()  - self.threshold) as int;
            let height = (self.get_height() - self.threshold) as int;

            //left-right shifting
            self.left_col = match cursor.0 as int - self.left_col as int {
                x_offset if x_offset < self.threshold as int => {
                    cmp::max(0, self.left_col as int - (self.threshold as int - x_offset)) as uint
                }
                x_offset if x_offset >= width => {
                    self.left_col + (x_offset - width + 1) as uint
                }
                _ => { self.left_col }
            };

            //up-down shifting
            match cursor.1 as int - top_line as int {
                y_offset if y_offset < self.threshold as int && top_line > 0 => {
                    self.buffer.shift_mark(self.top_line,
                                           Direction::Up,
                                           (self.threshold as int - y_offset) as uint);
                }
                y_offset if y_offset >= height => {
                    self.buffer.shift_mark(self.top_line,
                                           Direction::Down,
                                           (y_offset - height + 1) as uint);
                }
                _ => { }
            }
        }
    }

    pub fn delete_chars(&mut self, direction: Direction, num_chars: uint) {
        let chars = self.buffer.remove_chars(self.cursor, direction, num_chars);
        match (chars, direction) {
            (Some(chars), Direction::Left) => {
                self.move_cursor(Direction::Left, chars.len());
            }
            (Some(chars), Direction::LeftWord(..)) => {
                self.move_cursor(Direction::Left, chars.len());
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

    /// Insert a chacter into the buffer & update cursor position accordingly.
    pub fn insert_char(&mut self, ch: char) {
        // NOTE: the last param to char_width here may not be correct
        if let Some(ch_width) = utils::char_width(ch, false, 4, 0) {
            self.buffer.insert_char(self.cursor, ch as u8);
            self.move_cursor(Direction::Right, ch_width)
        }
    }

    pub fn undo(&mut self) {
        let point = if let Some(transaction) = self.buffer.undo() { transaction.end_point }
                    else { return; };
        self.buffer.set_mark(self.cursor, point);
        self.maybe_move_screen();
    }

    pub fn redo(&mut self) {
        let point = if let Some(transaction) = self.buffer.redo() { transaction.end_point }
                    else { return; };
        self.buffer.set_mark(self.cursor, point);
        self.maybe_move_screen();
    }

}

pub fn draw_line(buf: &mut UIBuffer, line: &[u8], idx: uint, left: uint) {
    let width = buf.get_width() - 1;
    let mut wide_chars = 0;
    for line_idx in range(left, left + width) {
        if line_idx < line.len() {
            let special_char = line[line_idx] as char;
            match special_char {
                '\t'   => {
                    let w = 4 - line_idx % 4;
                    for _ in range(0, w) {
                        buf.update_cell_content(line_idx + wide_chars - left, idx, ' ');
                    }
                }
                '\n'   => buf.update_cell_content(line_idx + wide_chars - left, idx, ' '),
                _       => buf.update_cell_content(line_idx + wide_chars - left, idx,
                                                   line[line_idx] as char),
            }
            wide_chars += (line[line_idx] as char).width(false).unwrap_or(1) - 1;
        } else { buf.update_cell_content(line_idx + wide_chars - left, idx, ' '); }
    }
    if line.len() >= width {
        buf.update_cell_content(width + wide_chars, idx, '→');
    }

}

#[cfg(test)]
mod tests {

    use buffer::Direction;
    use view::View;
    use input::Input;

    fn setup_view<'v>(testcase: &'static str) -> View<'v> {
        let mut view = View::new(Input::Filename(None), 50, 50);
        for ch in testcase.chars() {
            view.insert_char(ch);
        }
        view.buffer.set_mark(view.cursor, 0);
        view
    }

    #[test]
    fn test_move_cursor_down() {
        let mut view = setup_view("test\nsecond");
        view.move_cursor(Direction::Down, 1);
        assert_eq!(view.buffer.get_mark_coords(view.cursor).unwrap().1, 1);
        assert_eq!(view.buffer.lines_from(view.cursor).unwrap().next().unwrap(), b"second"[]);
    }

    #[test]
    fn test_move_cursor_up() {
        let mut view = setup_view("test\nsecond");
        view.move_cursor(Direction::Down, 1);
        view.move_cursor(Direction::Up, 1);
        assert_eq!(view.buffer.get_mark_coords(view.cursor).unwrap().1, 0);
        assert_eq!(view.buffer.lines_from(view.cursor).unwrap().next().unwrap(), b"test\n"[]);
    }

    #[test]
    fn test_insert_line() {
        let mut view = setup_view("test\nsecond");
        view.move_cursor(Direction::Right, 1);
        view.insert_char('\n');

        assert_eq!(view.buffer.get_mark_coords(view.cursor).unwrap(), (0, 1))
    }

    #[test]
    fn test_insert_char() {
        let mut view = setup_view("test\nsecond");
        view.insert_char('t');

        assert_eq!(view.buffer.lines().next().unwrap(), b"ttest\n"[]);
    }

    #[test]
    fn test_delete_char_to_right() {
        let mut view = setup_view("test\nsecond");
        view.delete_chars(Direction::Right, 1);

        assert_eq!(view.buffer.lines().next().unwrap(), b"est\n"[]);
    }

    #[test]
    fn test_delete_char_to_left() {
        let mut view = setup_view("test\nsecond");
        view.move_cursor(Direction::Right, 1);
        view.delete_chars(Direction::Left, 1);

        assert_eq!(view.buffer.lines().next().unwrap(), b"est\n"[]);
    }


    #[test]
    fn test_delete_char_at_start_of_line() {
        let mut view = setup_view("test\nsecond");
        view.move_cursor(Direction::Down, 1);
        view.delete_chars(Direction::Left, 1);

        assert_eq!(view.buffer.lines().next().unwrap(), b"testsecond"[]);
    }

    #[test]
    fn test_delete_char_at_end_of_line() {
        let mut view = setup_view("test\nsecond");
        view.move_cursor(Direction::Right, 4);
        view.delete_chars(Direction::Right, 1);

        assert_eq!(view.buffer.lines().next().unwrap(), b"testsecond"[]);
    }

    #[test]
    fn deleting_backward_at_start_of_first_line_does_nothing() {
        let mut view = setup_view("test\nsecond");
        view.delete_chars(Direction::Left, 1);

        let lines: Vec<&[u8]> = view.buffer.lines().collect();

        assert_eq!(lines.len(), 2);
        assert_eq!(view.buffer.lines().next().unwrap(), b"test\n");
    }
}
