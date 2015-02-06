pub use super::keyboard::Key;
pub use super::keymap::KeyMap;
pub use super::editor::Command;
pub use super::keymap::KeyMapState;
pub use super::buffer::WordEdgeMatch;
pub use super::overlay::{Overlay, OverlayType, OverlayEvent};

pub use self::standard::StandardMode;
pub use self::normal::NormalMode;

use command;

mod standard;
mod normal;


/// The concept of Iota's modes are taken from Vi.
/// 
/// A mode is a mechanism for interpreting key events and converting them into
/// commands which the Editor will interpret.
pub trait Mode {
    /// Given a Key, return a Command for the Editor to interpret
    fn handle_key_event(&mut self, key: Key) -> command::BuilderEvent;
}
