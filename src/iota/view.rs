use std::cmp;
use std::borrow::Cow;
use std::path::Path;
use std::path::PathBuf;
use std::io::Write;
use std::fs::{File, rename};
use std::sync::{Mutex, Arc};

use tempdir::TempDir;
use unicode_width::UnicodeWidthChar;

use buffer::{Buffer, Mark};
use uibuf::{UIBuffer, CharColor};
use frontends::Frontend;
use overlay::{Overlay, OverlayType};
use utils;
use textobject::{Anchor, TextObject, Kind, Offset};

/// A View is an abstract Window (into a Buffer).
///
/// It draws a portion of a Buffer to a `UIBuffer` which in turn is drawn to the
/// screen. It maintains the status bar for the current view, the "dirty status"
/// which is whether the buffer has been modified or not and a number of other
/// pieces of information.
pub struct View {
    pub buffer: Arc<Mutex<Buffer>>,
    pub last_buffer: Option<Arc<Mutex<Buffer>>>,
    pub overlay: Overlay,

    /// First character of the top line to be displayed
    top_line: Mark,

    /// Index into the top_line - used for horizontal scrolling
    left_col: usize,

    /// The current View's cursor - a reference into the Buffer
    cursor: Mark,

    /// The UIBuffer to which the View is drawn
    uibuf: UIBuffer,

    /// Number of lines from the top/bottom of the View after which vertical
    /// scrolling begins.
    threshold: usize,
}

impl View {

    pub fn new(buffer: Arc<Mutex<Buffer>>, width: usize, height: usize) -> View {
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
            uibuf: UIBuffer::new(width, height),
            overlay: Overlay::None,
            threshold: 5,
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

            // FIXME: don't use unwrap here
            //        This will fail if for some reason the buffer doesnt have
            //        the top_line mark
            let mut lines = buffer.lines_from(self.top_line).unwrap().take(height);
            for y_position in 0..height {
                let line = lines.next().unwrap_or_else(Vec::new);
                draw_line(&mut self.uibuf, &line, y_position, self.left_col);
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

    #[cfg_attr(feature="clippy", allow(needless_range_loop))]
    fn draw_status<T: Frontend>(&mut self, frontend: &mut T) {
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
            self.uibuf.update_cell(index, height, ch, CharColor::Black, CharColor::Blue);
        }

        self.uibuf.draw_range(frontend, height, height+1);
    }

    fn draw_cursor<T: Frontend>(&mut self, frontend: &mut T) {
        let buffer = self.buffer.lock().unwrap();
        if let Some(top_line) = buffer.get_mark_display_coords(self.top_line) {
            if let Some((x, y)) = buffer.get_mark_display_coords(self.cursor) {
                frontend.draw_cursor((x - self.left_col) as isize, y as isize - top_line.1 as isize);
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

            OverlayType::SelectFile => {
                let prefix = "Enter file path:";

                self.overlay = Overlay::SelectFile {
                    cursor_x: prefix.len(),
                    prefix: prefix,
                    data: String::new(),
                };
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

    pub fn delete_from_mark_to_object(&mut self, mark: Mark, object: TextObject) {
        let mut buffer = self.buffer.lock().unwrap();
        if let Some(mark_pos) = buffer.get_object_index(object) {
            if let Some(midx) = buffer.get_mark_idx(mark) {
                buffer.remove_from_mark_to_object(mark, object);
                buffer.set_mark(mark, cmp::min(mark_pos.absolute, midx));
            }
        }
    }

    /// Insert a chacter into the buffer & update cursor position accordingly.
    pub fn insert_char(&mut self, ch: char) {
        self.buffer.lock().unwrap().insert_char(self.cursor, ch as u8);
        // NOTE: the last param to char_width here may not be correct
        if let Some(ch_width) = utils::char_width(ch, false, 4, 1) {
            let obj = TextObject {
                kind: Kind::Char,
                offset: Offset::Forward(ch_width, Mark::Cursor(0))
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
            let result = file.write_all(&*line);

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
            let prefix = "Enter file name: ";

            match buffer.file_path {
                Some(ref p) => {
                    if p.is_dir() {
                        // If it's a dir, we still need a file name. This is a tmp fix, 
                        // should ideally be handled better
                        let path_str = p.to_str().unwrap_or("");

                        self.overlay = Overlay::SavePrompt {
                            cursor_x: prefix.len() + path_str.len(),
                            prefix: prefix,
                            data: String::from(path_str),
                        };
                    } else {
                        should_save = true 
                    }
                },
                None => {
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
            let mut buffer = view.buffer.lock().unwrap();
            assert_eq!(buffer.lines().next().unwrap(), b"ttest\n");
        }
    }
}
