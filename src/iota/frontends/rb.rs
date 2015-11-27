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
    /// Draw a given char & styles to the terminal
    fn draw_char(&mut self, offset: usize, linenum: usize, ch: char, fg: CharColor, bg: CharColor, style: CharStyle) {
        let bg = get_color(bg);
        let fg = get_color(fg);
        let style = get_style(style);

        self.rb.print_char(offset, linenum, style, fg, bg, ch);
    }
}

/// Translate a CharColor to rustbox::Color
fn get_color(c: CharColor) -> Color {
    match c {
        CharColor::Default => Color::Default,
        CharColor::Blue    => Color::Blue,
        CharColor::Black   => Color::Black,
    }
}

/// Translate a CharStyle to rustbox::Style
fn get_style(s: CharStyle) -> Style {
    match s {
        CharStyle::Normal => Style::empty(),
    }
}
