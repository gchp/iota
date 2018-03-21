use keyboard::Key;
use keymap::{KeyMap, KeyMapState};
use command::{BuilderEvent, BuilderArgs };

use super::Mode;



/// Standard mode is Iota's default mode.
///
/// Standard mode uses non-vi-like keybindings.
/// Unlike Normal, Command and Visual modes which are all used together, Standard
/// mode is used on its own.
///
/// Standard mode allows Iota to be used in a non-modal way, similar to mainstream
/// editors like Atom or Sublime.
pub struct StandardMode {
    keymap: KeyMap,
    match_in_progress: bool,
}

impl StandardMode {

    /// Create a new instance of StandardMode
    pub fn new() -> StandardMode {
        StandardMode {
            keymap: StandardMode::key_defaults(),
            match_in_progress: false,
        }
    }

    /// Creates a KeyMap with default StandardMode key bindings
    fn key_defaults() -> KeyMap {
        let mut keymap = KeyMap::new();

        // Editor Commands
        keymap.bind_key(Key::Ctrl('q'), "editor::quit".into());
        keymap.bind_key(Key::Ctrl('s'), "editor::save_buffer".into());

        // Cursor movement
        keymap.bind_key(Key::Up, "buffer::move_cursor_backward_line".into());
        keymap.bind_key(Key::Down, "buffer::move_cursor_forward_line".into());
        keymap.bind_key(Key::Left, "buffer::move_cursor_backward_char".into());
        keymap.bind_key(Key::Right, "buffer::move_cursor_forward_char".into());

        keymap.bind_key(Key::CtrlRight, "buffer::move_cursor_forward_word_start".into());
        keymap.bind_key(Key::CtrlLeft, "buffer::move_cursor_backward_word_start".into());
    
        keymap.bind_key(Key::End, "buffer::move_cursor_line_end".into());
        keymap.bind_key(Key::Home, "buffer::move_cursor_line_start".into());

        // Editing
        keymap.bind_key(Key::Tab, "buffer::insert_tab".into());
        keymap.bind_key(Key::Enter, "buffer::insert_newline".into());
        keymap.bind_key(Key::Backspace, "buffer::delete_backward_char".into());
        keymap.bind_key(Key::Delete, "buffer::delete_forward_char".into());

        // History
        keymap.bind_key(Key::Ctrl('z'), "editor::undo".into());
        keymap.bind_key(Key::Ctrl('y'), "editor::redo".into());

        keymap
    }

    /// Checks a Key against the internal keymap
    ///
    /// - If there is a direct match, return the completed BuilderEvent
    /// - If there is a partial match, set match_in_progress to true which
    ///   indicates that the next key should check against the keymap too,
    ///   rather than possibly being inserted into the buffer. This allows
    ///   for non-prefixed keys to be used in keybindings. ie: C-x s rather
    ///   than C-x C-s.
    /// - If there is no match of any kind, return Incomplete
    fn check_key(&mut self, key: Key) -> BuilderEvent {
        match self.keymap.check_key(key) {
            KeyMapState::Match(c) => {
                self.match_in_progress = false;
                BuilderEvent::Complete(c, None)
            },
            KeyMapState::Continue => {
                self.match_in_progress = true;
                BuilderEvent::Incomplete
            }
            KeyMapState::None => {
                self.match_in_progress = false;
                BuilderEvent::Incomplete
            }
        }
    }

}

impl Mode for StandardMode {
    /// Given a key, pass it through the StandardMode KeyMap and return the associated Command, if any.
    /// If no match is found, treat it as an InsertChar command.
    fn handle_key_event(&mut self, key: Key) -> BuilderEvent {
        if self.match_in_progress {
            return self.check_key(key)
        }

        if let Key::Char(c) = key {
            let mut builder_args = BuilderArgs::new().with_char_arg(c);
            BuilderEvent::Complete("buffer::insert_char".into(), Some(builder_args))
        } else {
            self.check_key(key)
        }

    }
}

impl Default for StandardMode {
    fn default() -> Self {
        Self::new()
    }
}
