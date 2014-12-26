pub use super::keyboard::Key;
pub use super::keymap::KeyMap;
pub use super::editor::EventStatus;
pub use super::editor::Command;
pub use super::view::View;
pub use super::keymap::KeyMapState;
pub use super::log::LogEntries;
pub use super::cursor::Direction;
pub use super::Response;

pub use self::standard::StandardMode;

mod standard;


pub trait Mode {
    fn handle_key_event(&mut self, key: Option<Key>, view: &mut View, log: &mut LogEntries) -> EventStatus;
}
