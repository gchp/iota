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

        let status_text = format!("{} {} {} {}",
                                  buffer_status,
                                  cursor_status,
                                  self.top_line_num,
                                  self.get_internal_height());

        utils::draw(term_height-1, status_text);
    }

    pub fn move_cursor(&mut self, direction: Direction) {
        let y = self.cursor.get_linenum();
        match direction {
            Direction::Up    => { self.move_cursor_to(y-1); },
            Direction::Down  => { self.move_cursor_to(y+1); },
            Direction::Right => { self.cursor.move_right(); },
            Direction::Left  => { self.cursor.move_left(); },
        }
    }

    pub fn move_cursor_to(&mut self, line_num: uint) {
        let internal_height = self.get_internal_height();
        let calculated_height = internal_height + self.top_line_num;

        let num_lines = self.buffer.lines.len() - 1;

        if line_num <= calculated_height {
            let line = &self.buffer.lines[line_num];
            self.cursor.set_line(Some(line));
            self.cursor.set_linenum(line_num);
        }

    }

    pub fn delete_char(&mut self) {
        let (offset, line_num) = self.cursor.get_position();

        if offset == 0 {
            let (offset, line_num) = self.buffer.join_line_with_previous(offset, line_num);
            self.move_cursor_to(line_num);
            self.cursor.set_offset(offset);
            return
        }

        self.cursor.delete_char()
    }

    pub fn insert_char(&mut self, ch: char) {
        self.cursor.insert_char(ch);
    }

    pub fn insert_line(&mut self) {
        let (offset, line_num) = self.cursor.get_position();
        let (offset, line_num) = self.buffer.insert_line(offset, line_num);

        self.move_cursor_to(line_num);
        self.cursor.set_offset(offset);
    }
}
