use std::char;

use rustbox::{RustBox, Event};
use rustbox::{Style, Color};

use super::Frontend;
use super::{CharStyle, CharColor};
use super::Key;
use super::EditorEvent;


/// Terminal-based front end using Rustbox
pub struct RustboxFrontend<'f> {
    rb: &'f RustBox,
}

impl<'f> RustboxFrontend<'f> {
    /// Create a new instance of the RustboxFrontend
    pub fn new(rb: &'f RustBox) -> RustboxFrontend<'f> {
        RustboxFrontend {
            rb: rb,
        }
    }
}

impl<'f> Frontend for RustboxFrontend<'f> {
    /// Poll Rustbox for events & translate them into an EditorEvent
    fn poll_event(&self) -> EditorEvent {
        match self.rb.poll_event(true).unwrap() {
            Event::KeyEventRaw(_, key, ch) => {
                let k = match key {
                    0 => char::from_u32(ch).map(Key::Char),
                    a => Key::from_special_code(a),
                };
                EditorEvent::KeyEvent(k)
            }
            Event::ResizeEvent(width, height) => {
                EditorEvent::Resize(width as usize, height as usize)
            }
            _ => EditorEvent::UnSupported
        }
    }

    /// Draw the cursor to the terminal
    fn draw_cursor(&mut self, offset: isize, linenum: isize) {
        self.rb.set_cursor(offset, linenum)
    }

    /// Draw a given char & styles to the terminal
    fn draw_char(&mut self, offset: usize, linenum: usize, ch: char, fg: CharColor, bg: CharColor, style: CharStyle) {
        let bg = get_color(bg);
        let fg = get_color(fg);
        let style = get_style(style);

        self.rb.print_char(offset, linenum, style, fg, bg, ch);
    }

    /// Present the newly drawn data (cursor / content) to the user
    fn present(&self) {
        self.rb.present()
    }

    /// Get the terminal height
    fn get_window_height(&self) -> usize {
        self.rb.height()
    }

    /// Get the terminal width
    fn get_window_width(&self) -> usize {
        self.rb.width()
    }
}

/// Translate a `CharColor` to `rustbox::Color`
fn get_color(c: CharColor) -> Color {
    match c {
        CharColor::Blue    => Color::Blue,
        CharColor::Red    => Color::Red,
        CharColor::Black   => Color::Black,
        CharColor::Magenta   => Color::Magenta,
        CharColor::Green   => Color::Green,
        CharColor::Yellow   => Color::Yellow,
        CharColor::White   => Color::White,
        CharColor::Orange => Color::Byte(0x10),
        CharColor::Gray => Color::Byte(0x08),
    }
}

/// Translate a `CharStyle` to `rustbox::Style`
fn get_style(s: CharStyle) -> Style {
    match s {
        CharStyle::Normal => Style::empty(),
    }
}
