use std::char;

use rustbox::{RustBox, Event};
use rustbox::{Style, Color};

use super::Frontend;
use super::{CharStyle, CharColor};
use super::Key;
use super::EditorEvent;


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

    fn draw_char(&mut self, offset: uint, linenum: uint, ch: char, fg: CharColor, bg: CharColor, style: CharStyle) {
        let bg = get_color(bg);
        let fg = get_color(fg);
        let style = get_style(style);

        self.rb.print_char(offset, linenum, style, fg, bg, ch);
    }
}

fn get_color(c: CharColor) -> Color {
    match c {
        CharColor::Default => Color::Default,
        CharColor::Blue    => Color::Blue,
        CharColor::Black   => Color::Black,
    }
}

fn get_style(s: CharStyle) -> Style {
    match s {
        CharStyle::Normal => Style::empty(),
    }
}
