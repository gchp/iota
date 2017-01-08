use unicode_width::UnicodeWidthChar;
use std::path::PathBuf;

use uibuf::CharColor;
use uibuf::UIBuffer;
use keyboard::Key;
use frontends::Frontend;
use command::BuilderEvent;
use command::Command;


#[derive(Copy, Clone, Debug)]
pub enum OverlayType {
    SelectFile
}


pub trait Overlay {
    fn draw(&self, uibuf: &mut UIBuffer);
    fn handle_key_event(&mut self, key: Key) -> BuilderEvent;

    fn needs_cursor(&self) -> bool {
        false
    }

    fn get_cursor_x(&self) -> isize { 0 }
    fn get_cursor_y(&self) -> isize { 0 }
}

use std::io;
use std::fs::{self, DirEntry};
use std::path::Path;

pub struct SelectFilePrompt {
    pub buffer: String,
    pub prefix: &'static str,
    pub cursor_x: usize,
    pub cursor_y: usize,
}

impl Overlay for SelectFilePrompt {
    fn draw(&self, uibuf: &mut UIBuffer) {
        let height = uibuf.get_height() - 1;
        let offset = self.prefix.len() + 1;

        // draw the given prefix
        for (index, ch) in self.prefix.chars().enumerate() {
            uibuf.update_cell_content(index, height, ch);
        }

        // draw the overlay data
        for (index, ch) in self.buffer.chars().enumerate() {
            uibuf.update_cell_content(index + offset, height, ch);
        }

        if self.buffer.len() == 0 {
            return
        }

        let path = PathBuf::from(&*self.buffer);
        let mut entries = Vec::new();
        if path.is_dir() {
            match fs::read_dir(path) {
                Ok(iter) => {
                    for entry in iter {
                        let entry = entry.unwrap();
                        let p = entry.path();
                        entries.push(p)
                    }
                }
                Err(_) => {
                    return
                }
            }
        }

        use std::io::stderr;
        use std::io::Write;
        let mut out = stderr();
        writeln!(&mut out, "{:?}", entries);

        let top_line_index = uibuf.get_height() - 8;
        let left_side = self.prefix.len() + 1;
        let right_side = left_side + 20;

        for index in left_side..right_side {
            uibuf.update_cell_content(index, top_line_index, '-');
        }

        for index in (top_line_index + 1)..height {
            for i in left_side..(right_side + 1) {
                let ch = if i == left_side || i == right_side { '|' } else { ' ' };
                uibuf.update_cell(i, index, ch, CharColor::White, CharColor::Black);
            }
        }
    }

    fn handle_key_event(&mut self, key: Key) -> BuilderEvent {
        match key {
            Key::Char(c) => {
                self.buffer.push(c);
                // TODO: calculate proper char width
                self.cursor_x += 1;
                BuilderEvent::Complete(Command::noop())
            }

            Key::Enter => {
                BuilderEvent::Complete(Command::clear_overlay())
            }

            Key::Esc => {
                BuilderEvent::Complete(Command::clear_overlay())
            }

            _ => {
                BuilderEvent::Complete(Command::noop())
            }
        }
    }

    fn needs_cursor(&self) -> bool { true }
    fn get_cursor_x(&self) -> isize { self.cursor_x as isize }
    fn get_cursor_y(&self) -> isize { self.cursor_x as isize }
}
