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
    buffer: Buffer,         //Text buffer
    first_char: Mark,       //First character to be displayed
    cursor: Mark,           //Cursor displayed by this buffer.
    uibuf: UIBuffer,        //UIBuffer
}

impl<'v> View<'v> {

    //----- CONSTRUCTORS ---------------------------------------------------------------------------

    pub fn new(source: Input, rb: &RustBox) -> View<'v> {
        let buffer = match source {
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
        buffer.add_mark(cursor, 0);
        let first_char = Mark::DisplayMark(0);
        buffer.add_mark(first_char, 0);

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
                                .lines_from(self.buffer.get_mark_idx(self.first_char).unwrap())
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
        self.buffer.move_mark_to_line_term(self.cursor, Direction::Right);
        self.cursor_movement();
    }

    pub fn move_cursor_to_line_start(&mut self) {
        self.buffer.move_mark_to_line_term(self.cursor, Direction::Left);
        self.cursor_movement();
    }

    fn cursor_movement(&mut self) {

        //Update the point to be at the cursor.
        self.buffer.update_point(self.cursor);

        //Update the first_char mark if necessary to keep the cursor on the screen.
        let cursor_y = self.buffer.get_mark_coords(self.cursor).unwrap().val1();
        let first_char_y = self.buffer.get_mark_coords(self.first_char).unwrap().val1();
        if cursor_y < first_char_y {
            self.buffer.shift_mark(self.first_char, Direction::Up);
        } else if cursor_y > first_char_y + self.get_height() {
            self.buffer.shift_mark(self.first_char, Direction::Down);
        }

    }

    //----- TEXT EDIT METHODS ----------------------------------------------------------------------

    pub fn delete_char(&mut self, direction: Direction) {
        match direction {
            Direction::Left => {
                self.buffer.remove_char();
                self.move_cursor(direction);
            }
            Direction::Right => {
                self.move_cursor(direction);
                self.buffer.remove_char();
            }
        }
    }

    pub fn insert_tab(&mut self) {
        // A tab is just 4 spaces
        for _ in range(0i, 4) {
            self.insert_char(' ');
        }
    }

    pub fn insert_char(&mut self, ch: char) {
        self.buffer.insert_char(ch);
        self.move_cursor(Direction::Right)
    }

}

pub fn draw_line(buf: &mut UIBuffer, line: &[u8], idx: uint) {
    let width = buf.get_width() - 1;
    let mut wide_chars = 0;
    for line_idx in range(0, width) {
        if line_idx < line.len() {
            match line[line_idx] {
                '\t'    => {
                    let w = 4 - line_idx % 4;
                    for _ in range(0, w) {
                        buf.update_cell_content(line_idx + wide_chars, idx, ' ');
                        line_idx += 1;
                    }
                }
                '\n'    => buf.update_cell_content(line_idx + wide_chars, idx, ' '),
                _       => buf.update_cell_content(line_idx + wide_chars, idx, line[line_idx]),
            }
            wide_chars += line[line_idx].width(false).unwrap_or(1) - 1;
        } else { buf.update_cell_content(line_idx + wide_chars, idx, ' '); }
    }
    if line.len() >= width {
        buf.update_cell_content(width + wide_chars, idx, '→');
    }

}
