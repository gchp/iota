use unicode_width::UnicodeWidthChar;

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

pub struct SelectFilePrompt {
    pub buffer: String,
    pub prefix: &'static str,
    pub cursor_x: usize,
    pub cursor_y: usize,
}

impl Overlay for SelectFilePrompt {
    fn draw(&self, uibuf: &mut UIBuffer) {
        let height = uibuf.get_height() - 1;
        let offset = self.prefix.len();

        // draw the given prefix
        for (index, ch) in self.prefix.chars().enumerate() {
            uibuf.update_cell_content(index, height, ch);
        }

        // draw the overlay data
        for (index, ch) in self.buffer.chars().enumerate() {
            uibuf.update_cell_content(index + offset, height, ch);
        }

        // uibuf.draw_range(frontend, height, height+1);
    }

    fn handle_key_event(&mut self, key: Key) -> BuilderEvent {
        match key {
            Key::Char(c) => {
                self.buffer.push(c);
                // TODO: calculate proper char width
                self.cursor_x += 1;
                BuilderEvent::Complete(Command::noop())
            }

            Key::Enter | Key::Esc  => {
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
