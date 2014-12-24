use std::io::stdio;
use std::char;

use rustbox::{InitOption, RustBox, Event};
use rustbox::{Style, Color};

use super::Frontend;
use super::Key;
use super::EditorEvent;
use super::View;


pub struct RustboxFrontend<'f> {
    rb: &'f RustBox,
}

impl<'f> RustboxFrontend<'f> {
    pub fn new(rb: &'f RustBox) -> RustboxFrontend<'f> {
        RustboxFrontend {
            rb: rb,
        }
    }
}

impl<'f> Frontend for RustboxFrontend<'f> {
    fn poll_event(&self) -> EditorEvent {
        match self.rb.poll_event().unwrap() {
            Event::KeyEvent(_, key, ch) => {
                let k = match key {
                    0 => char::from_u32(ch).map(|c| Key::Char(c)),
                    a => Key::from_special_code(a),
                };
                EditorEvent::KeyEvent(k)
            }
            _ => EditorEvent::UnSupported
        }
    }

    fn draw_cursor(&mut self, offset: int, linenum: int) {
        self.rb.set_cursor(offset, linenum)
    }

    fn draw_char(&mut self, offset: uint, linenum: uint, ch: char) {
        self.rb.print_char(offset, linenum, Style::empty(), Color::Default, Color::Default, ch);
    }
}
