pub use super::keyboard::Key;
pub use super::uibuf::{CharStyle, CharColor};

pub use self::rb::RustboxFrontend;

/// A general means of representing events from different frontends.
///
/// A frontend should translate its own events into an `EditorEvent` to be used
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
    /// Poll the frontend for an event, translating it to an EditorEvent
    fn poll_event(&self) -> EditorEvent;
    /// Present the newly drawn data (cursor / content) to the user
    fn present(&self);
    /// Get the frontends window height or equivalent
    fn get_window_height(&self) -> usize;
    /// Get the frontends window width or equivalent
    fn get_window_width(&self) -> usize;
    /// Draw the cursor to the frontend
    fn draw_cursor(&mut self, offset: isize, linenum: isize);
    /// Draw the given char & styles to the frontend
    fn draw_char(&mut self, offset: usize, linenum: usize, ch: char, fg: CharColor, bg: CharColor, style: CharStyle);
}

mod rb;
