use std::borrow::Cow;
use std::path::Path;
use std::path::PathBuf;
use std::io::Write;
use std::fs::{File, rename};
use std::sync::{Mutex, Arc};
use std::iter;

use tempdir::TempDir;
use unicode_width::UnicodeWidthChar;

use buffer::{Buffer, Position};
use uibuf::{UIBuffer, CharColor};
use frontends::Frontend;
use overlay::{Overlay, OverlayType};
use textobject::TextObject;

/// A View is an abstract Window (into a Buffer).
///
/// It draws a portion of a Buffer to a UIBuffer which in turn is drawn to the
/// screen. It maintains the status bar for the current view, the "dirty status"
/// which is whether the buffer has been modified or not and a number of other
/// pieces of information.
pub struct View {
    pub buffer: Arc<Mutex<Buffer>>,
    pub last_buffer: Option<Arc<Mutex<Buffer>>>,
    pub overlay: Overlay,

    /// The character in the upper-left corner
    top_left: Position,

    /// The position of the current View's cursor
    cursor: Position,

    /// The UIBuffer to which the View is drawn
    uibuf: UIBuffer,

    /// Number of lines from the top/bottom of the View after which vertical
    /// scrolling begins.
    threshold: usize,
}

impl View {

    pub fn new(buffer: Arc<Mutex<Buffer>>, width: usize, height: usize) -> View {
        View {
            buffer: buffer,
            last_buffer: None,
            top_left: Position::origin(),
            cursor: Position::origin(),
            uibuf: UIBuffer::new(width, height),
            overlay: Overlay::None,
            threshold: 5,
        }
    }

    pub fn set_buffer(&mut self, buffer: Arc<Mutex<Buffer>>) {
        self.last_buffer = Some(self.buffer.clone());
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
        let status_bar_height = 1;
        let uibuf_height = self.uibuf.get_height();

        uibuf_height - status_bar_height
    }

    /// Get the width of the View.
    pub fn get_width(&self) -> usize {
        self.uibuf.get_width()
    }

    /// Resize the view
    ///
    /// This involves simply changing the size of the associated UIBuffer
    pub fn resize(&mut self, width: usize, height: usize) {
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
        {
            let buffer = self.buffer.lock().unwrap();
            let height = self.get_height() - 1;

            let lines = buffer.slice_from(Position::new(0, self.top_left.y))
                              .lines().chain(iter::repeat("".to_string()))
                              .take(height);
            for (y, line) in lines.enumerate() {
                draw_line(&mut self.uibuf, line.as_bytes(), y, self.top_left.x);
            }
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
        let buffer = self.buffer.lock().unwrap();
        let buffer_status = buffer.status_text();
        let status_text = format!("{} ({}, {})", buffer_status, self.cursor.x + 1, self.cursor.y + 1).into_bytes();
        let status_text_len = status_text.len();
        let width = self.get_width();
        let height = self.get_height() - 1;


        for index in 0..width {
            let mut ch: char = ' ';
            if index < status_text_len {
                ch = status_text[index] as char;
            }
            self.uibuf.update_cell(index, height, ch, CharColor::Black, CharColor::Blue);
        }

        self.uibuf.draw_range(frontend, height, height+1);
    }

    fn draw_cursor<T: Frontend>(&mut self, frontend: &mut T) {
        let x = self.cursor.x - self.top_left.x;
        let y = self.cursor.y - self.top_left.y;
        // FIXME: support for multi-width chars
        // (x is in units of chars, we want units of cells)
        frontend.draw_cursor(x as isize, y as isize);
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

            OverlayType::SelectFile => {
                let prefix = "Enter file path:";

                self.overlay = Overlay::SelectFile {
                    cursor_x: prefix.len(),
                    prefix: prefix,
                    data: String::new(),
                };
            }
            _ => {}
        }
    }

    pub fn set_cursor_to_object(&mut self, object: TextObject) {
        let buffer = self.buffer.lock().unwrap();
        if let Some(pos) = buffer.get_object_position(self.cursor, object) {
            self.cursor = pos;
        }
    }

    /// Scroll (vertically and horizontally) if necessary to keep the cursor
    /// on the screen.
    fn maybe_move_screen(&mut self) {
        let cursor_x = self.cursor.x;
        let cursor_y = self.cursor.y;
        let left = self.top_left.x;
        let top = self.top_left.y;
        let width = self.get_width();
        let height = self.get_height();
        let threshold = self.threshold;

        // Horizontal scrolling
        if cursor_x < left + threshold {
            // Scroll left
            let amount = left + threshold - cursor_x;
            if amount <= left {
                self.top_left.x -= amount;
            } else {
                // can't scroll that far
                self.top_left.x = 0;
            }
        } else if cursor_x > left + width - threshold {
            // Scroll right
            let amount = left + width - threshold - cursor_x;
            self.top_left.x += amount;
        }

        // Vertical scrolling
        if cursor_y < top + threshold {
            // Scroll up
            let amount = top + threshold - cursor_y;
            if amount <= top {
                self.top_left.y -= amount;
            } else {
                // can't scroll that far
                self.top_left.y = 0;
            }
        } else if cursor_y > top + height - threshold {
            // Scroll down
            let amount = top + height - threshold - cursor_y;
            self.top_left.y += amount;
        }
    }

    // Delete chars from the first index of object to the last index of object
    pub fn delete_object(&mut self, object: TextObject) {
        self.buffer.lock().unwrap().remove_object(self.cursor, object);
    }

    pub fn delete_from_cursor_to_object(&mut self, object: TextObject) {
        let mut buffer = self.buffer.lock().unwrap();
        match buffer.get_object_position(self.cursor, object) {
            Some(pos) if pos > self.cursor => {
                buffer.remove(self.cursor, pos);
            },
            Some(pos) if pos < self.cursor => {
                buffer.remove(pos, self.cursor);
                self.cursor = pos;
            },
            _ => ()
        }
    }

    /// Insert a chacter into the buffer & update cursor position accordingly.
    pub fn insert_char(&mut self, ch: char) {
        let mut buf = self.buffer.lock().unwrap();
        buf.insert_char(self.cursor, ch);
        if let Some(pos) = buf.next_position(self.cursor) {
            self.cursor = pos;
        }
    }

    pub fn undo(&mut self) {
        {
            let mut buffer = self.buffer.lock().unwrap();
            let point = if let Some(pt) = buffer.undo() { pt }
                        else { return; };
            self.cursor = point;
        }
        self.maybe_move_screen();
    }

    pub fn redo(&mut self) {
        {
            let mut buffer = self.buffer.lock().unwrap();
            let point = if let Some(pt) = buffer.redo() { pt }
                        else { return; };
            self.cursor = point;
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

        let result = file.write_all(buffer.to_string().as_bytes());
        if result.is_err() {
            // TODO(greg): figure out what to do here.
            panic!("Something went wrong while writing the file");
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
                Some(_) => { should_save = true }
                None => {
                    let prefix = "Enter file name: ";
                    self.overlay = Overlay::SavePrompt {
                        cursor_x: prefix.len(),
                        prefix: prefix,
                        data: String::new(),
                    };
                },
            }
        }

        if should_save { self.save_buffer() }
    }

}

pub fn draw_line(buf: &mut UIBuffer, line: &[u8], idx: usize, left: usize) {
    let width = buf.get_width() - 1;
    let mut x = 0;
    
    for ch in line.iter().skip(left) {
        let ch = *ch as char;
        match ch {
            '\t' => {
                let w = 4 - x % 4;
                for _ in 0..w {
                    buf.update_cell_content(x, idx, ' ');
                    x += 1;
                }
            }
            '\n' => {}
            _ => {
                buf.update_cell_content(x, idx, ch);
                x += UnicodeWidthChar::width(ch).unwrap_or(1);
            }
        }
        if x >= width {
            break;
        }
    }
    
    // Replace any cells after end of line with ' '
    while x < width {
        buf.update_cell_content(x, idx, ' ');
        x += 1;
    }
    
    // If the line is too long to fit on the screen, show an indicator
    let indicator = if line.len() > width + left { '→' } else { ' ' };
    buf.update_cell_content(width, idx, indicator);
}

#[cfg(test)]
mod tests {

    use std::sync::{Arc, Mutex};

    use view::View;
    use input::Input;
    use buffer::Buffer;

    fn setup_view(testcase: &'static str) -> View {
        let mut buffer = Arc::new(Mutex::new(Buffer::new()));
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
            let mut buffer = view.buffer.lock().unwrap();
            assert_eq!(buffer.lines().next().unwrap(), b"ttest\n");
        }
    }
}
