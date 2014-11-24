use std::io::{File, BufferedReader};
use std::cell::RefCell;

use utils;


pub struct Buffer<'b> {
    pub file_path: String,
    pub lines: Vec<RefCell<Line>>,
}

impl<'b> Buffer<'b> {
    /// Create a new buffer instance
    pub fn new() -> Buffer<'b> {
        Buffer {
            file_path: String::new(),
            lines: Vec::new(),
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

        buffer
    }

    pub fn get_status_text(&self) -> String {
        let file_path = self.file_path.clone();
        let line_count = self.lines.len();
        format!("{}, lines: {}", file_path, line_count)
    }

    pub fn insert_line(&mut self, offset: uint, mut line_num: uint) -> (uint, uint) {
        // split the current line at the cursor position
        let bits = &self.split_line(offset, line_num);
        {
            // truncate the current line
            let line = &self.get_line_at(line_num);
            line.unwrap().borrow_mut().data.truncate(offset);
        }

        line_num += 1;

        // add new line below current
        utils::clear_line(line_num);
        self.lines.insert(line_num, RefCell::new(Line::new(bits.clone().remove(1).unwrap())));

        return (0, line_num)
    }

    /// Join the line identified by `line_num` with the one at `line_num - 1 `.
    pub fn join_line_with_previous(&mut self, offset: uint, line_num: uint) -> (uint, uint ) {
        let mut current_line_data: String;
        let mut prev_line_data: String;
        let line_len: uint;
        {
            let prev_line = self.get_line_at(line_num - 1);
            if prev_line.is_none() {
                return (offset, line_num)
            }
            prev_line_data = prev_line.unwrap().borrow().data.clone();
            line_len = prev_line_data.len();
        }

        {
            // get current line data
            let current_line = self.get_line_at(line_num).unwrap();
            current_line_data = current_line.borrow().data.clone();
        }

        {
            // append current line data to prev line
            // FIXME: this is duplicated above in a different scope...
            let prev_line = self.get_line_at(line_num - 1).unwrap();

            let new_data = format!("{}{}", prev_line_data, current_line_data);
            prev_line.borrow_mut().data = new_data;
        }

        utils::clear_line(line_num);
        self.lines.remove(line_num);

        return (line_len, line_num - 1)
    }

    fn split_line(&mut self, offset: uint, line_num: uint) -> Vec<String> {
        let line = self.get_line_at(line_num).unwrap();

        let data = line.borrow().data.clone().into_bytes();
        let old_data = data.slice_to(offset);
        let new_data = data.slice_from(offset);

        vec!(
            String::from_utf8_lossy(old_data).into_string(),
            String::from_utf8_lossy(new_data).into_string(),
        )
    }

    fn get_line_at(&self, line_num: uint) -> Option<&RefCell<Line>> {
        let num_lines = self.lines.len() -1;
        if line_num > num_lines { return None }
        Some(&self.lines[line_num])
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


