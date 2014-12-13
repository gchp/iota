use std::io::{File, Reader, BufferedReader};

pub struct Buffer {
    pub file_path: Option<Path>,
    pub lines: Vec<Line>,
}

impl Buffer {
    /// Create a new buffer instance
    pub fn new() -> Buffer {
        Buffer {
            file_path: None,
            lines: Vec::new(),
        }
    }

    /// Create a new buffer with a single line
    pub fn new_empty() -> Buffer {
        let mut buffer = Buffer::new();
        buffer.lines.push(Line::new(String::new(), 0));

        buffer
    }

    fn lines_from_reader<R: Reader>(reader: &mut BufferedReader<R>) -> Vec<Line> {
        let mut v = vec![];
        // for every line in the reader we add a corresponding line to the buffer
        for (index, line) in reader.lines().enumerate() {
            let mut data = line.unwrap();
            let last_index = data.len() - 1;
            if data.is_char_boundary(last_index) && data.char_at(last_index) == '\n' {
                data.pop();
            }
            v.push(Line::new(data.trim_right_chars('\n').into_string(), index));
        }
        v
    }

    pub fn new_from_reader<R: Reader>(reader: R) -> Buffer {
        let mut r = BufferedReader::new(reader);
        Buffer {
            lines: Buffer::lines_from_reader(&mut r),
            file_path: None
        }
    }

    /// Create a new buffer instance and load the given file
    pub fn new_from_file(path: Path) -> Buffer {
        let mut buffer = Buffer::new();

        if let Ok(file) = File::open(&path) {
            buffer.lines = Buffer::lines_from_reader(&mut BufferedReader::new(file));
        } else {
            buffer.lines.push(Line::new(String::new(), 0));
        }

        buffer.file_path = Some(path);
        buffer
    }

    pub fn get_status_text(&self) -> String {
        match self.file_path {
            Some(ref path) => format!("{}, lines: {}", path.display(), self.lines.len()),
            None => format!("untitled, lines: {}", self.lines.len())
        }
    }

    fn fix_linenums(&mut self) {
        for (index, line) in self.lines.iter_mut().enumerate() {
            line.linenum = index;
        }
    }

    pub fn insert_line(&mut self, offset: uint, mut line_num: uint) {
        // split the current line at the cursor position
        let (_, new_data) = self.split_line(offset, line_num);
        {
            // truncate the current line
            let line = self.get_line_at_mut(line_num);
            line.unwrap().data.truncate(offset);
        }

        line_num += 1;

        self.lines.insert(line_num, Line::new(new_data, line_num));

        self.fix_linenums();
    }

    /// Join the line identified by `line_num` with the one at `line_num - 1 `.
    pub fn join_line_with_previous(&mut self, offset: uint, line_num: uint) -> uint {
        // if the line_num is 0 (ie the first line), don't do anything
        if line_num == 0 { return offset }

        let mut current_line_data: String;
        {
            let current_line = match self.get_line_at(line_num) {
                Some(line) => line,
                None => return offset,
            };
            current_line_data = current_line.data.clone();
        }

        // update the previous line
        let new_cursor_offset = match self.get_line_at_mut(line_num -1) {
            None => offset,
            Some(line) => {
                let line_len = line.data.len();
                line.data.push_str(&*current_line_data);
                line_len
            }
        };

        self.lines.remove(line_num);
        self.fix_linenums();

        return new_cursor_offset
    }

    // TODO(greg): refactor this to use Vec::partition
    /// Split the line identified by `line_num` at `offset`
    fn split_line(&mut self, offset: uint, line_num: uint) -> (String, String) {
        let line = self.get_line_at(line_num).unwrap();

        let data = line.data.clone();
        let old_data = data.slice_to(offset);
        let new_data = data.slice_from(offset);

        let mut new = String::new(); new.push_str(new_data);
        let mut old = String::new(); old.push_str(old_data);

        return (old, new)
    }

    fn get_line_at(&self, line_num: uint) -> Option<&Line> {
        let num_lines = self.lines.len() -1;
        if line_num > num_lines { return None }
        Some(&self.lines[line_num])
    }

    fn get_line_at_mut(&mut self, line_num: uint) -> Option<&mut Line> {
        let num_lines = self.lines.len() -1;
        if line_num > num_lines { return None }
        Some(&mut self.lines[line_num])
    }

}


pub struct Line {
    pub data: String,
    pub linenum: uint,
}

impl Line {
    /// Create a new line instance
    pub fn new(data: String, line_num: uint) -> Line {
        Line{
            data: data,
            linenum: line_num,
        }
    }

    /// Get the length of the current line
    pub fn len(&self) -> uint {
        self.data.len()
    }
}


#[cfg(test)]
mod tests {

    use buffer::Buffer;
    use buffer::Line;
    use utils::data_from_str;

    fn setup_buffer() -> Buffer {
        let mut buffer = Buffer::new();
        buffer.file_path = Some(Path::new("/some/file.txt"));
        buffer.lines = vec!(
            Line::new(data_from_str("test"), 0),
            Line::new(String::new(), 1),
            Line::new(data_from_str("text file"), 2),
            Line::new(data_from_str("content"), 3),
        );
        buffer
    }

    #[test]
    fn test_get_status_text() {
        let buffer = setup_buffer();
        assert_eq!(buffer.get_status_text(), "/some/file.txt, lines: 4".to_string())
    }

    #[test]
    fn test_insert_line() {
        let mut buffer = setup_buffer();
        buffer.insert_line(0, 0);
        assert_eq!(buffer.lines.len(), 5);
    }

    #[test]
    fn test_insert_line_in_middle_of_other_line() {
        let mut buffer = setup_buffer();
        buffer.insert_line(1, 0);
        assert_eq!(buffer.lines.len(), 5);

        let ref line = buffer.lines[1];
        assert_eq!(line.data, data_from_str("est"));
    }

    #[test]
    fn test_line_numbers_are_fixed_after_adding_new_line() {
        let mut buffer = setup_buffer();
        buffer.insert_line(1, 2);
        assert_eq!(buffer.lines.len(), 5);

        // check that all linenums are sequential
        for (index, line) in buffer.lines.iter().enumerate() {
            assert_eq!(index, line.linenum);
        }
    }

    #[test]
    fn test_join_line_with_previous() {
        let mut buffer = setup_buffer();

        let offset = buffer.join_line_with_previous(0, 3);

        assert_eq!(buffer.lines.len(), 3);
        assert_eq!(buffer.lines[2].data, data_from_str("text filecontent"));
        assert_eq!(offset, 9);
    }

    #[test]
    fn join_line_with_previous_does_nothing_on_line_zero_offset_zero() {
        let mut buffer = setup_buffer();
        buffer.join_line_with_previous(0, 0);

        assert_eq!(buffer.lines.len(), 4);
        assert_eq!(buffer.lines[0].data, data_from_str("test"));
    }

    #[test]
    fn test_split_line() {
        let mut buffer = setup_buffer();
        let (old, new) = buffer.split_line(3, 3);

        assert_eq!(old, data_from_str("con"));
        assert_eq!(new, data_from_str("tent"));
    }

    #[test]
    fn joining_line_with_non_existant_next_line_does_nothing() {
        let mut buffer = setup_buffer();
        buffer.lines = vec!(Line::new(String::new(), 0));
        buffer.join_line_with_previous(0, 1);
    }

}

