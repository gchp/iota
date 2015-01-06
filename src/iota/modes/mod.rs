pub use super::keyboard::Key;
pub use super::keymap::KeyMap;
pub use super::editor::EventStatus;
pub use super::editor::Command;
pub use super::view::View;
pub use super::keymap::KeyMapState;
pub use super::buffer::Direction;
pub use super::Response;
pub use super::utils;
pub use super::overlay::{Overlay, OverlayType, OverlayEvent};

pub use self::standard::StandardMode;
pub use self::normal::NormalMode;

mod standard;
mod normal;


pub trait Mode {
    fn handle_key_event(&mut self, key: Option<Key>, view: &mut View) -> EventStatus;
    fn interpret_input(&mut self, input: String, view: &mut View) -> Response;
}
