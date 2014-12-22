use rustbox::{Color, RustBox};

use buffer::{Direction, Buffer};
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
    pub buffer: Buffer,
    // the line number of the topmost `Line` for the View to render
    pub top_line_num: uint,
    pub linenum: uint,
    pub offset: uint,

    uibuf: UIBuffer,
    threshold: int,
}

impl<'v> View<'v> {

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

        View {
            buffer: buffer,
            top_line_num: 0,
            uibuf: uibuf,
            threshold: 5,
            linenum: 0,
            offset: 0,
        }
    }

    /// Clear the buffer
    ///
    /// Fills every cell in the UIBuffer with the space (' ') char.
    pub fn clear(&mut self, rb: &RustBox) {
        self.uibuf.fill(' ');
        self.uibuf.draw_everything(rb);
    }

    pub fn get_height(&self) -> uint {
        // NOTE(greg): when the status bar needs to move up, this value should be changed
        self.uibuf.get_height() -1
    }

    pub fn get_width(&self) -> uint {
        self.uibuf.get_width()
    }


    //FIXME
    pub fn draw(&mut self, rb: &RustBox) {
        let end_line = self.get_height();
        let num_lines = self.buffer.lines.len();
        let lines_to_draw = self.buffer.lines.slice(self.top_line_num, num_lines);

        for (index, line) in lines_to_draw.iter().enumerate() {
            if index < end_line {
                draw_line(&mut self.uibuf, line, self.top_line_num)
            }
        }

        self.uibuf.draw_everything(rb);
    }

    //FIXME
    pub fn draw_status(&mut self, rb: &RustBox) {
        let buffer_status = self.buffer.get_status_text();
        let cursor_status = self.cursor().get_status_text();
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

    //FIXME
    pub fn draw_cursor(&mut self, rb: &RustBox) {
        let offset = self.cursor().get_visible_offset();
        let linenum = self.cursor().get_linenum();

        utils::draw_cursor(rb, offset, linenum-self.top_line_num);
    }

    pub fn resize(&mut self, rb: &RustBox) {
        let width = self.uibuf.get_width();
        self.clear(rb);
        self.uibuf = UIBuffer::new(width, 15);
    }

    pub fn move_cursor(&mut self, direction: Direction) {
        self.buffer.shift_cursor(direction);
    }

    //FIXME
    pub fn move_cursor_to_line_end(&mut self) {
        self.cursor().set_offset(::std::uint::MAX);
    }

    //FIXME
    pub fn move_cursor_to_line_start(&mut self) {
        self.cursor().set_offset(0);
    }

    //FIXME
    fn set_cursor_line<'b>(&'b mut self, linenum: uint) {
        let vis_width = self.cursor().get_visible_offset();
        let mut offset = 0;
        let mut vis_acc = 0;
        let line = &*self.buffer.lines[linenum].data;
        for _ in line.char_indices().take_while(|&(i, c)| {
            offset = line.char_range_at(i).next;
            vis_acc += ::utils::char_width(c, false, 4, vis_acc).unwrap_or(0);
            vis_acc < vis_width
        }) {}
        if self.offset == 0 { offset = 0 }
        self.offset = offset;
        self.linenum = linenum;
    }

    //FIXME
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
        match direction {
            Direction::Left => {
                self.remove_char();
                self.buffer.shift_cursor(direction);
            }
            Direction::Right => {
                self.buffer.shift_cursor(direction);
                self.remove_char();
            }
            _
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
        self.shift_cursor(Right);
    }

}

//FIXME
pub fn draw_line(buf: &mut UIBuffer, line: &Line, top_line_num: uint) {
    let width = buf.get_width() -1;
    let index = line.linenum - top_line_num;
    let mut internal_index = 0;
    for ch in line.data.chars() {
        if internal_index < width {
            match ch {
                '\t' => {
                    let w = 4 - internal_index%4;
                    for _ in range(0, w) {
                        buf.update_cell_content(internal_index, index, ' ');
                        internal_index += 1;
                    }
                }
                _ => {
                    // draw the character
                    buf.update_cell_content(internal_index, index, ch);
                    internal_index += ch.width(false).unwrap_or(1);
                }
            }
        }

        // if the line is longer than the width of the view, draw a special char
        if internal_index == width {
            buf.update_cell_content(internal_index, index, '→');
            break;
        }
    }
}
