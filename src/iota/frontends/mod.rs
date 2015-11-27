pub use super::keyboard::Key;
pub use super::uibuf::{CharStyle, CharColor};

pub use self::rb::RustboxFrontend;

/// A general means of representing events from different frontends.
///
/// A frontend should translate its own events into an EditorEvent to be used
/// elsewhere in the program.
pub enum EditorEvent {
    KeyEvent(Option<Key>),
    Resize(usize, usize),
    UnSupported
}

/// A Frontend is a means of representing the Editor to the user.
///
/// For example, there could be frontends for:
/// - a terminal
/// - a GUI
/// - a web browser
///
/// Frontends are chose at startup and can't be changed while the Editor is
/// running.
pub trait Frontend {
    fn draw_char(&mut self, offset: usize, linenum: usize, ch: char, fg: CharColor, bg: CharColor, style: CharStyle);
}

mod rb;
