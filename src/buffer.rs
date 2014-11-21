use std::io::{File, BufferedReader};
use std::cell::RefCell;

use utils;
use cursor::{Direction, Cursor};


pub struct Buffer<'b> {
    pub file_path: String,
    pub lines: Vec<RefCell<Line>>,

    pub cursor: Cursor<'b>,
}

impl<'b> Buffer<'b> {
    /// Create a new buffer instance
    pub fn new() -> Buffer<'b> {
        Buffer {
            file_path: String::new(),
            lines: Vec::new(),
            cursor: Cursor::new(),
        }
    }

    /// Create a new buffer instance and load the given file
    pub fn new_from_file(path: &Path) -> Buffer<'b> {
        let mut file = BufferedReader::new(File::open(path));
        let lines: Vec<String> = file.lines().map(|x| x.unwrap()).collect();
        let mut buffer = Buffer::new();

        buffer.file_path = path.as_str().unwrap().to_string();

        // for every line in the file we add a corresponding line to the buffer
        for line in lines.iter() {
            let mut data = line.clone();
            // remove \n chars
            data.pop();
            buffer.lines.push(RefCell::new(Line::new(data)));
        }
        buffer.cursor.set_line(Some(&buffer.lines[0]));

        buffer
    }

    /// Draw the contents of the buffer
    ///
    /// Loops over each line in the buffer and draws it to the screen
    pub fn draw_contents(&self) {
        for (index, line) in self.lines.iter().enumerate() {
            let ln = line.borrow();
            utils::draw(index, ln.data.clone());
        }
    }

    pub fn draw_status(&self) {
        let height = utils::get_term_height();
        let cursor = self.cursor.get_position();
        let data = self.file_path.clone();
        let line_count = self.lines.len();
        utils::draw(
            height - 1,
            format!("{}, cursor: {}, term: {}, lines: {}",
                    data, cursor, utils::get_term_stats(), line_count));
    }

    pub fn adjust_cursor(&mut self, dir: Direction) {
        let (mut x, mut y) = self.cursor.get_position();
        match dir {
            Direction::Up => {
                let line = self.move_cursor_to(y-1);
                if line.is_some() {
                    y -= 1;
                    // if the current cursor offset is after the end of the
                    // previous line, move the offset back to the end of the line
                    let line_len = line.unwrap().borrow().data.len();
                    if x > line_len {
                        x = line_len;
                    }
                }
            }
            Direction::Down => {
                let line = self.move_cursor_to(y+1);
                if line.is_some() {
                    y += 1;
                    // if the current cursor offset is after the end of the
                    // next line, move the offset back to the end of the line
                    let line_len = line.unwrap().borrow().data.len();
                    if x > line_len {
                        x = line_len;
                    }
                }
            }
            Direction::Right => {
                let line = self.cursor.get_line();
                if line.borrow().len() > x {
                    x += 1
                }
            }
            Direction::Left => {
                if x > 0 {
                    x -= 1
                }
            }
        }
        self.cursor.set_position(x, y);
    }

    pub fn delete_char(&mut self) {
        let (offset, line_num) = self.cursor.get_position();

        if offset == 0 {
            return self.join_line_with_previous(line_num);
        }

        self.cursor.delete_char()
    }

    pub fn insert_char(&mut self, ch: char) {
        self.cursor.insert_char(ch);
    }

    pub fn insert_line(&mut self) {
        let (offset, mut line_num) = self.cursor.get_position();

        // split the current line at the cursor position
        let bits = &self.split_line();
        {
            // truncate the current line
            let line = &self.move_cursor_to(line_num);
            line.unwrap().borrow_mut().data.truncate(offset);
        }

        line_num += 1;

        // add new line below current
        utils::clear_line(line_num);
        self.lines.insert(line_num, RefCell::new(Line::new(bits.clone().remove(1).unwrap())));

        // move the cursor down and to the start of the next line
        self.cursor.set_position(0, line_num);
    }

    /// Join the line identified by `line_num` with the one at `line_num - 1 `.
    fn join_line_with_previous(&mut self, line_num: uint) {
        let mut current_line_data: String;
        let mut prev_line_data: String;
        let line_len: uint;
        {
            let prev_line = self.move_cursor_to(line_num - 1);
            if prev_line.is_none() {
                return
            }
            prev_line_data = prev_line.unwrap().borrow().data.clone();
            line_len = prev_line_data.len();
        }

        // get current line data
        let current_line = self.cursor.get_line();
        current_line_data = current_line.borrow().data.clone();

        {
            // append current line data to prev line
            // FIXME: this is duplicated above in a different scope...
            let prev_line = self.move_cursor_to(line_num - 1).unwrap();

            let new_data = format!("{}{}", prev_line_data, current_line_data);
            prev_line.borrow_mut().data = new_data;
        }

        utils::clear_line(line_num);
        self.lines.remove(line_num);
        self.cursor.set_position(line_len, line_num - 1);
    }

    fn split_line(&mut self) -> Vec<String> {
        let (x, _) = self.cursor.get_position();
        let line = self.cursor.get_line();

        let data = line.borrow().data.clone().into_bytes();
        let old_data = data.slice_to(x);
        let new_data = data.slice_from(x);

        vec!(
            String::from_utf8_lossy(old_data).into_string(),
            String::from_utf8_lossy(new_data).into_string(),
        )
    }

    fn move_cursor_to(&mut self, line_num: uint) -> Option<&RefCell<Line>> {
        // get the line identified by line_num
        let num_lines = self.lines.len() -1;
        if line_num > num_lines { return None }
        let line = &self.lines[line_num];

        // set it on the cursor
        let mut cursor = self.cursor.clone();
        cursor.set_line(Some(line));
        self.cursor = cursor;

        Some(cursor.get_line())
    }

}


pub struct Line {
    pub data: String,
}

impl Line {
    /// Create a new line instance
    pub fn new(data: String) -> Line {
        Line{
            data: data,
        }
    }

    /// Get the length of the current line
    pub fn len(&self) -> uint {
        self.data.len()
    }
}


