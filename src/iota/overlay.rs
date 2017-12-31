use unicode_width::UnicodeWidthChar;
use rustbox::{Style, Color, RustBox};
use command::{Command, BuilderEvent};

use keyboard::Key;


#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum OverlayType {
    CommandPrompt,
}

pub trait Overlay {
    fn draw(&self, rb: &mut RustBox);
    fn draw_cursor(&mut self, rb: &mut RustBox);
    fn handle_key_event(&mut self, key: Key) -> BuilderEvent;
}

pub struct CommandPrompt {
    cursor_x: usize,
    data: String,
    prefix: String,
}

impl CommandPrompt {
    pub fn new() -> CommandPrompt {
        CommandPrompt {
            cursor_x: 1,
            data: String::new(),
            prefix: String::from(":"),
        }
    }
}


impl Overlay for CommandPrompt {
    fn draw(&self, rb: &mut RustBox) {
        let height = rb.height() - 1;
        let offset = self.prefix.len();

        // draw the given prefix
        for (index, ch) in self.prefix.chars().enumerate() {
            rb.print_char(index, height, Style::empty(), Color::White, Color::Black, ch);
        }

        // draw the overlay data
        for (index, ch) in self.data.chars().enumerate() {
            rb.print_char(index + offset, height, Style::empty(), Color::White, Color::Black, ch);
        }
    }

    fn draw_cursor(&mut self, rb: &mut RustBox) {
        // Prompt is always on the bottom, so we can use the
        // height given by the frontend here
        let height = rb.height() - 1;
        rb.set_cursor(self.cursor_x as isize, height as isize)
    }

    fn handle_key_event(&mut self, key: Key) -> BuilderEvent {
        match key {
            Key::Esc => return BuilderEvent::Invalid,
            Key::Backspace => {
                if let Some(c) = self.data.pop() {
                    if let Some(width) = UnicodeWidthChar::width(c) {
                        self.cursor_x -= width;
                    }
                }
            }
            Key::Enter => {
                match &*self.data {
                    // FIXME: need to find a better system for these commands
                    //        They should be chainable
                    //          ie: wq - save & quit
                    //        They should also take arguments
                    //          ie w file.txt - write buffer to file.txt
                    "q" | "quit" => return BuilderEvent::Complete(Command::exit_editor()),
                    "w" | "write" => return BuilderEvent::Complete(Command::save_buffer()),

                    _ => return BuilderEvent::Incomplete
                }
            }
            Key::Char(c) => {
                if let Some(width) = UnicodeWidthChar::width(c) {
                    self.data.push(c);
                    self.cursor_x += width;
                }
            }
            _ => {}
        }
        return BuilderEvent::Incomplete;
    }
}
