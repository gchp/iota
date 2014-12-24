pub use super::keyboard::Key;
pub use super::view::View;

pub use self::rb::RustboxFrontend;

pub enum EditorEvent {
    KeyEvent(Option<Key>),
    UnSupported
}

impl EditorEvent {
    fn unwrap(self) -> Option<Key> {
        match self {
            EditorEvent::KeyEvent(k) => k,
            EditorEvent::UnSupported => panic!("Unsupported event from RustboxFrontend"),
        }
    }
}

pub trait Frontend {
    fn poll_event(&self) -> EditorEvent;
    fn draw_cursor(&mut self, offset: int, linenum: int);
    fn draw_char(&mut self, offset: uint, linenum: uint, ch: char);
}

mod rb;
