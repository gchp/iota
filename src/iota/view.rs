use std::cmp;
use std::borrow::Cow;
use std::path::Path;
use std::path::PathBuf;
use std::io::Write;
use std::fs::{File, rename};
use std::sync::{Mutex, Arc};
use std::time::SystemTime;
use rustbox::{Color, RustBox, Style as RustBoxStyle};

use tempdir::TempDir;
use unicode_width::UnicodeWidthChar;

use buffer::{Buffer, Mark};
use overlay::{CommandPrompt, Overlay, OverlayType};
use textobject::{Anchor, TextObject, Kind, Offset};


/// A View is an abstract Window (into a Buffer).
///
/// It draws a portion of a Buffer to a `UIBuffer` which in turn is drawn to the
/// screen. It maintains the status bar for the current view, the "dirty status"
/// which is whether the buffer has been modified or not and a number of other
/// pieces of information.
pub struct View<'v> {
    pub buffer: Arc<Mutex<Buffer>>,
    pub last_buffer: Option<Arc<Mutex<Buffer>>>,
    pub overlay: Option<Box<Overlay + 'v>>,

    height: usize,
    width: usize,

    /// First character of the top line to be displayed
    top_line: Mark,

    /// Index into the top_line - used for horizontal scrolling
    left_col: usize,

    /// The current View's cursor - a reference into the Buffer
    cursor: Mark,

    /// Number of lines from the top/bottom of the View after which vertical
    /// scrolling begins.
    threshold: usize,

    /// Message to be displayed in the status bar along with the time it
    /// was displayed.
    message: Option<(&'static str, SystemTime)>,
}

impl<'v> View<'v> {

    pub fn new(buffer: Arc<Mutex<Buffer>>, width: usize, height: usize) -> View<'v> {
        let cursor = Mark::Cursor(0);
        let top_line = Mark::DisplayMark(0);

        {
            let mut b = buffer.lock().unwrap();

            b.set_mark(cursor, 0);
            b.set_mark(top_line, 0);
        }

        View {
            buffer: buffer,
            last_buffer: None,
            top_line: top_line,
            left_col: 0,
            cursor: cursor,
            overlay: None,
            threshold: 5,
            message: None,
            height: height,
            width: width,
        }
    }

    pub fn selection_start() -> TextObject {
        TextObject {
            kind: Kind::Line(Anchor::Start),
            offset: Offset::Backward(0, Mark::Cursor(0)),
        }
    }
    pub fn selection_end() -> TextObject {
        TextObject {
            kind: Kind::Line(Anchor::Same), //Anchor::Start makes more sense, but isn't implemented
            offset: Offset::Forward(1, Mark::Cursor(0)),
        }
    }

    pub fn set_buffer(&mut self, buffer: Arc<Mutex<Buffer>>) {
        self.last_buffer = Some(self.buffer.clone());

        {
            let mut b = buffer.lock().unwrap();

            b.set_mark(self.cursor, 0);
            b.set_mark(self.top_line, 0);
        }

        self.buffer = buffer;
    }

    pub fn switch_last_buffer(&mut self) {
        let buffer = self.buffer.clone();
        let last_buffer = match self.last_buffer.clone() {
            Some(buf) => buf,
            None => return
        };

        self.buffer = last_buffer;
        self.last_buffer = Some(buffer);
    }

    /// Get the height of the View.
    ///
    /// This is the height of the UIBuffer minus the status bar height.
    pub fn get_height(&self) -> usize {
        self.height - 1
    }

    /// Get the width of the View.
    pub fn get_width(&self) -> usize {
        self.width
    }

    /// Resize the view
    ///
    /// This involves simply changing the size of the associated UIBuffer
    pub fn resize(&mut self, width: usize, height: usize) {
        self.height = height;
        self.width = width;
    }

    /// Clear the buffer
    ///
    /// Fills every cell in the UIBuffer with the space (' ') char.
    pub fn clear(&mut self, rb: &mut RustBox) {
        for row in 0..self.height {
            for col in 0..self.width {
                rb.print_char(col, row, RustBoxStyle::empty(), Color::White, Color::Black, ' ');
            }
        }
    }

    pub fn draw(&mut self, rb: &mut RustBox) {
        self.clear(rb);
        {
            let buffer = self.buffer.lock().unwrap();
            let height = self.get_height() - 1;
            let width = self.get_width() - 1;

            // FIXME: don't use unwrap here
            //        This will fail if for some reason the buffer doesnt have
            //        the top_line mark
            let mut lines = buffer.lines_from(self.top_line).unwrap().take(height);
            for y_position in 0..height {
                let line = lines.next();
                draw_line(rb, line.unwrap_or(String::from("")), y_position, self.left_col);
            }

        }

        self.draw_status(rb);

        match self.overlay {
            None => self.draw_cursor(rb),
            Some(ref mut overlay) => {
                overlay.draw(rb);
                overlay.draw_cursor(rb);
            }
        }
    }

    #[cfg_attr(feature="clippy", allow(needless_range_loop))]
    fn draw_status(&mut self, rb: &mut RustBox) {
        let buffer = self.buffer.lock().unwrap();
        let buffer_status = buffer.status_text();
        let mut cursor_status = buffer.get_mark_display_coords(self.cursor).unwrap_or((0,0));
        cursor_status = (cursor_status.0 + 1, cursor_status.1 + 1);
        let status_text = format!("{} ({}, {})", buffer_status, cursor_status.0, cursor_status.1).into_bytes();
        let status_text_len = status_text.len();
        let width = self.get_width();
        let height = self.get_height() - 1;


        for index in 0..width {
            let ch: char = if index < status_text_len {
                status_text[index] as char
            } else { ' ' };
            rb.print_char(index, height, RustBoxStyle::empty(), Color::Black, Color::Byte(19), ch);

        }

        if buffer.dirty {
            let data = ['[', '*', ']'];
            for (idx, ch) in data.iter().enumerate() {
                rb.print_char(status_text_len + idx + 1, height, RustBoxStyle::empty(), Color::Black, Color::Red, *ch);
            }
        }
        if let Some((message, _time)) = self.message {
            for (offset, ch) in message.chars().enumerate() {
                rb.print_char(offset, height + 1, RustBoxStyle::empty(), Color::White, Color::Black, ch);
            }
        }
    }

    fn draw_cursor(&mut self, rb: &mut RustBox) {
        let buffer = self.buffer.lock().unwrap();
        if let Some(top_line) = buffer.get_mark_display_coords(self.top_line) {
            if let Some((x, y)) = buffer.get_mark_display_coords(self.cursor) {
                rb.set_cursor((x - self.left_col) as isize, y as isize - top_line.1 as isize);
            }
        }
    }

    pub fn set_overlay(&mut self, overlay_type: OverlayType) {
        match overlay_type {
            OverlayType::CommandPrompt => {
                self.overlay = Some(Box::new(CommandPrompt::new()));
            }
        }
    }

    /// Display the given message
    pub fn show_message(&mut self, message: &'static str) {
        self.message = Some((message, SystemTime::now()));
    }

    /// Clear the currently displayed message if it has been there for 5 or more seconds
    ///
    /// Does nothing if there is no message, or of the message has been there for
    /// less that five seconds.
    pub fn maybe_clear_message(&mut self) {
        if let Some((_message, time)) = self.message {
            if let Ok(elapsed) = time.elapsed() {
                if elapsed.as_secs() >= 5 {
                    self.message = None;
                }
            }
        }
    }

    pub fn move_mark(&mut self, mark: Mark, object: TextObject) {
        self.buffer.lock().unwrap().set_mark_to_object(mark, object);
        self.maybe_move_screen();
    }

    /// Update the top_line mark if necessary to keep the cursor on the screen.
    fn maybe_move_screen(&mut self) {
        let mut buffer = self.buffer.lock().unwrap();
        if let (Some(cursor), Some((_, top_line))) = (buffer.get_mark_display_coords(self.cursor),
                                                      buffer.get_mark_display_coords(self.top_line)) {

            let width  = (self.get_width()  - self.threshold) as isize;
            let height = (self.get_height() - self.threshold) as isize;

            //left-right shifting
            self.left_col = match cursor.0 as isize - self.left_col as isize {
                x_offset if x_offset < self.threshold as isize => {
                    cmp::max(0, self.left_col as isize - (self.threshold as isize - x_offset)) as usize
                }
                x_offset if x_offset >= width => {
                    self.left_col + (x_offset - width + 1) as usize
                }
                _ => { self.left_col }
            };

            //up-down shifting
            match cursor.1 as isize - top_line as isize {
                y_offset if y_offset < self.threshold as isize && top_line > 0 => {
                    let amount = (self.threshold as isize - y_offset) as usize;
                    let obj = TextObject {
                        kind: Kind::Line(Anchor::Same),
                        offset: Offset::Backward(amount, self.top_line)
                    };
                    buffer.set_mark_to_object(self.top_line, obj);
                }
                y_offset if y_offset >= height => {
                    let amount = (y_offset - height + 1) as usize;
                    let obj = TextObject {
                        kind: Kind::Line(Anchor::Same),
                        offset: Offset::Forward(amount, self.top_line)
                    };
                    buffer.set_mark_to_object(self.top_line, obj);
                }
                _ => { }
            }
        }
    }

    // Delete chars from the first index of object to the last index of object
    pub fn delete_object(&mut self, object: TextObject) {
        self.buffer.lock().unwrap().remove_object(object);
    }

    pub fn delete_from_mark_to_object(&mut self, mark: Mark, object: TextObject) -> Option<Vec<char>> {
        let mut buffer = self.buffer.lock().unwrap();
        let mut ret: Option<Vec<char>> = None;
        
        if let Some(mark_pos) = buffer.get_object_index(object) {
            if let Some(midx) = buffer.get_mark_idx(mark) {
                ret = buffer.remove_from_mark_to_object(mark, object);
                buffer.set_mark(mark, cmp::min(mark_pos.absolute, midx));
            }
        }

        ret
    }

    pub fn delete_selection(&mut self) -> Option<Vec<char>> {
        // TODO: Implement proper selection? Lines are used for now.
        self.move_mark(Mark::Cursor(0), View::selection_start());

        // TODO: Implement copy/paste buffer
        self.delete_from_mark_to_object(Mark::Cursor(0), View::selection_end())
    }

    pub fn cut_selection(&mut self) {
        // TODO Store the deleted content
        self.delete_selection();
    }

    pub fn move_selection(&mut self, down: bool) {
        if down {
            self.move_mark(Mark::Cursor(0), TextObject {
                kind: Kind::Selection(Anchor::End),
                offset: Offset::Forward(0, Mark::Cursor(0)),
            });

            self.move_mark(Mark::Cursor(0), TextObject {
                kind: Kind::Char,
                offset: Offset::Forward(1, Mark::Cursor(0)),
            });

            let content = self.delete_selection();
                                    
            self.move_mark(Mark::Cursor(0), TextObject {
                kind: Kind::Selection(Anchor::Start),
                offset: Offset::Backward(1, Mark::Cursor(0)),
            });

            self.insert_string(content.unwrap().into_iter().collect());
        } else {
            let content = self.delete_selection();

            self.move_mark(Mark::Cursor(0), TextObject {
                kind: Kind::Selection(Anchor::Start),
                offset: Offset::Backward(1, Mark::Cursor(0)),
            });
            
            self.insert_string(content.unwrap().into_iter().collect());

            self.move_mark(Mark::Cursor(0), TextObject {
                kind: Kind::Selection(Anchor::Start),
                offset: Offset::Backward(1, Mark::Cursor(0)),
            });
        };
    } 

    /// Insert a chacter into the buffer & update cursor position accordingly.
    pub fn insert_char(&mut self, ch: char) {
        self.insert_string(ch.to_string())
    }

    /// Insert a string into the buffer & update cursor position accordingly.
    pub fn insert_string(&mut self, s: String) {
        let len = self.buffer.lock().unwrap().insert_string(self.cursor, s);
        // NOTE: the last param to char_width here may not be correct
        if len.unwrap() > 0 {
            let obj = TextObject {
                kind: Kind::Char,
                offset: Offset::Forward(len.unwrap(), Mark::Cursor(0))
            };
            self.move_mark(Mark::Cursor(0), obj)
        }
    }

    pub fn undo(&mut self) {
        {
            let mut buffer = self.buffer.lock().unwrap();
            let point = if let Some(transaction) = buffer.undo() { transaction.end_point }
                        else { return; };
            buffer.set_mark(self.cursor, point);
        }
        self.maybe_move_screen();
    }

    pub fn redo(&mut self) {
        {
            let mut buffer = self.buffer.lock().unwrap();
            let point = if let Some(transaction) = buffer.redo() { transaction.end_point }
                        else { return; };
            buffer.set_mark(self.cursor, point);
        }
        self.maybe_move_screen();
    }

    fn save_buffer(&mut self) {
        let buffer = self.buffer.lock().unwrap();
        let path = match buffer.file_path {
            Some(ref p) => Cow::Borrowed(p),
            None => {
                // NOTE: this should never happen, as the file path
                // should have been set inside the try_save_buffer method.
                //
                // If this runs, it probably means save_buffer has been called
                // directly, rather than try_save_buffer.
                //
                // TODO: ask the user to submit a bug report on how they hit this.
                Cow::Owned(PathBuf::from("untitled"))
            },
        };
        let tmpdir = match TempDir::new_in(&Path::new("."), "iota") {
            Ok(d) => d,
            Err(e) => panic!("file error: {}", e)
        };

        let tmppath = tmpdir.path().join(Path::new("tmpfile"));
        let mut file = match File::create(&tmppath) {
            Ok(f) => f,
            Err(e) => {
                panic!("file error: {}", e)
            }
        };

        //TODO (lee): Is iteration still necessary in this format?
        for line in buffer.lines() {
            let result = file.write_all(line.into_bytes().as_slice());

            if result.is_err() {
                // TODO(greg): figure out what to do here.
                panic!("Something went wrong while writing the file");
            }
        }

        if let Err(e) = rename(&tmppath, &*path) {
            panic!("file error: {}", e);
        }
    }

    pub fn try_save_buffer(&mut self) {
        let mut should_save = false;
        {
            let buffer = self.buffer.lock().unwrap();

            match buffer.file_path {
                Some(_) => { should_save = true; }
                None => {
                    self.message = Some(("No file name", SystemTime::now()));
                }
            }
        }

        if should_save {
            self.save_buffer();
            let mut buffer = self.buffer.lock().unwrap();
            buffer.dirty = false;
        }
    }

    /// Whether or not the current buffer has unsaved changes
    pub fn buffer_is_dirty(&mut self) -> bool {
        self.buffer.lock().unwrap().dirty
    }

}

pub fn draw_line(rb: &mut RustBox, line: String, idx: usize, left: usize) {
    let width = rb.width() - 1;
    let mut x = 0;

    for ch in line.chars().skip(left) {
        match ch {
            '\t' => {
                let w = 4 - x % 4;
                for _ in 0..w {
                    rb.print_char(x, idx, RustBoxStyle::empty(), Color::White, Color::Black, ' ');
                    x += 1;
                }
            }
            '\n' => {}
            _ => {
                rb.print_char(x, idx, RustBoxStyle::empty(), Color::White, Color::Black, ch);
                x += UnicodeWidthChar::width(ch).unwrap_or(1);
            }
        }
        if x >= width {
            break;
        }
    }

    // Replace any cells after end of line with ' '
    while x < width {
        rb.print_char(x, idx, RustBoxStyle::empty(), Color::White, Color::Black, ' ');
        x += 1;
    }

    // If the line is too long to fit on the screen, show an indicator
    let indicator = if line.len() > width + left { '→' } else { ' ' };
    rb.print_char(width, idx, RustBoxStyle::empty(), Color::White, Color::Black, indicator);
}

#[cfg(test)]
mod tests {

    use std::sync::{Arc, Mutex};
    use std::rc::Rc;

    use view::View;
    use buffer::Buffer;

    fn setup_view(testcase: &'static str) -> View {
        let buffer = Arc::new(Mutex::new(Buffer::new()));
        let mut view = View::new(buffer.clone(), 50, 50);
        for ch in testcase.chars() {
            view.insert_char(ch);
        }

        let mut buffer = buffer.lock().unwrap();
        buffer.set_mark(view.cursor, 0);
        view
    }

    #[test]
    fn test_insert_char() {
        let mut view = setup_view("test\nsecond");
        view.insert_char('t');

        {
            let buffer = view.buffer.lock().unwrap();
            assert_eq!(buffer.lines().next().unwrap(), "ttest\n");
        }
    }

    #[test]
    fn test_insert_string() {
        let mut view = setup_view("test\nsecond");
        view.insert_string(String::from("test!"));

        {
            let buffer = view.buffer.lock().unwrap();
            assert_eq!(buffer.lines().next().unwrap(), "test!test\n");
        }
    }

    #[test]
    fn test_delete_selection() {
        let mut view = setup_view("test\nsecond");
        view.delete_selection();

        {
            let buffer = view.buffer.lock().unwrap();
            assert_eq!(buffer.lines().next().unwrap(), "second");
        }
    }
    
    #[test]
    fn test_move_selection() {
        let mut view = setup_view("test\nsecond\nthird");
        view.move_selection(true);

        {
            let buffer = view.buffer.lock().unwrap();
            assert_eq!(buffer.lines().next().unwrap(), "second\n");
        }
    }
    
    #[test]
    fn test_move_selection_small() {
        let mut view = setup_view("test\nsecond\n");
        view.move_selection(true);

        {
            let buffer = view.buffer.lock().unwrap();
            assert_eq!(buffer.lines().next().unwrap(), "second\n");
        }
    }
}
