use keyboard::Key;
use keymap::{KeyMap, KeyMapState};
use command::{Builder, BuilderEvent, Command};

use super::Mode;


/// NormalMode mimics Vi's Normal mode.
pub struct NormalMode {
    keymap: KeyMap<Command>,
    builder: Builder,
}

impl NormalMode {

    /// Create a new instance of NormalMode
    pub fn new() -> NormalMode {
        NormalMode {
            keymap: NormalMode::key_defaults(),
            builder: Builder::new(),
        }
    }

    /// Creates a KeyMap with default NormalMode key bindings
    fn key_defaults() -> KeyMap<Command> {
        let mut keymap = KeyMap::new();

        keymap.bind_key(Key::Char('u'), Command::undo());
        keymap.bind_key(Key::Ctrl('r'), Command::redo());

        keymap
    }

}

impl Mode for NormalMode {
    fn handle_key_event(&mut self, key: Key) -> BuilderEvent {
        match self.builder.check_key(key) {
            // builder gives us a full command, return that
            BuilderEvent::Complete(cmd) => BuilderEvent::Complete(cmd),

            // no command from the builder, check the internal keymap
            BuilderEvent::Incomplete => {
                if let KeyMapState::Match(c) = self.keymap.check_key(key) {
                    BuilderEvent::Complete(c)
                } else {
                    BuilderEvent::Incomplete
                }
            }

            // invalid result from builder, return invalid
            BuilderEvent::Invalid => { BuilderEvent::Invalid }
        }
    }
}
