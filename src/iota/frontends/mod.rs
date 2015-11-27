pub use super::keyboard::Key;
pub use super::uibuf::{CharStyle, CharColor};

/// A general means of representing events from different frontends.
///
/// A frontend should translate its own events into an EditorEvent to be used
/// elsewhere in the program.
pub enum EditorEvent {
    KeyEvent(Option<Key>),
    Resize(usize, usize),
    UnSupported
}
