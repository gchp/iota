use keyboard::Key;
use keymap::{KeyMap, KeyMapState};
use command::{BuilderEvent, BuilderArgs };

use super::Mode;


/// Emacs mode uses Emacs-like keybindings.
///
pub struct EmacsMode {
    keymap: KeyMap,
    match_in_progress: bool,
}

impl EmacsMode {

    /// Create a new instance of EmacsMode
    pub fn new() -> EmacsMode {
        EmacsMode {
            keymap: EmacsMode::key_defaults(),
            match_in_progress: false,
        }
    }

    /// Creates a KeyMap with default EmacsMode key bindings
    fn key_defaults() -> KeyMap {
        let mut keymap = KeyMap::new();

        // Editor Commands
        keymap.bind_keys(&[Key::Ctrl('x'), Key::Ctrl('c')], "editor::exit".into());
        keymap.bind_keys(&[Key::Ctrl('x'), Key::Ctrl('s')], "editor::save_buffer".into());

        // Cursor movement
        keymap.bind_key(Key::Up, "buffer::move_cursor_backward_line".into());
        keymap.bind_key(Key::Down, "buffer::move_cursor_forward_line".into());
        keymap.bind_key(Key::Left, "buffer::move_cursor_forward_char".into());
        keymap.bind_key(Key::Right, "buffer::move_cursor_backward_char".into());
        keymap.bind_key(Key::Ctrl('p'), "buffer::move_cursor_backward_line".into());
        keymap.bind_key(Key::Ctrl('n'), "buffer::move_cursor_forward_line".into());
        keymap.bind_key(Key::Ctrl('f'), "buffer::move_cursor_forward_char".into());
        keymap.bind_key(Key::Ctrl('b'), "buffer::move_cursor_backward_char".into());

        keymap.bind_key(Key::Ctrl('e'), "buffer::move_cursor_line_end".into());
        keymap.bind_key(Key::Ctrl('a'), "buffer::move_cursor_line_start".into());

        // Editing
        keymap.bind_key(Key::Tab, "buffer::insert_tab".into());
        keymap.bind_key(Key::Enter, "buffer::insert_newline".into());
        keymap.bind_key(Key::Backspace, "buffer::delete_backward_char".into());
        keymap.bind_key(Key::Delete, "buffer::delete_forward_char".into());
        keymap.bind_key(Key::Ctrl('h'), "buffer::delete_backward_char".into());
        keymap.bind_key(Key::Ctrl('d'), "buffer::delete_forward_char".into());
        // keymap.bind_keys(&[Key::Ctrl('x'), Key::Ctrl('f')], Command {
        //     number: 1,
        //     action: Action::Instruction(Instruction::SetOverlay(OverlayType::SelectFile)),
        //     object: None
        // });
        keymap.bind_keys(&[Key::Ctrl('x'), Key::Ctrl('b')], "editor::switch_to_last_buffer".into());

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

impl Mode for EmacsMode {
    /// Given a key, pass it through the EmacsMode KeyMap and return the associated Command, if any.
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

impl Default for EmacsMode {
    fn default() -> Self {
        Self::new()
    }
}
