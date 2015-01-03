use super::Mode;
use super::KeyMap;
use super::Key;
use super::Command;
use super::View;
use super::KeyMapState;
use super::EventStatus;
use super::Direction;
use super::Response;
use super::utils;


/// Standard mode is Iota's default mode.
///
/// Standard mode uses non-vi-like keybindings.
/// Unlike Normal, Command and Visual modes which are all used together, Standard
/// mode is used on its own.
///
/// Standard mode allows Iota to be used in a non-modal way, similar to mainstream
/// editors like emacs or sublime.
pub struct StandardMode {
    keymap: KeyMap,
}

impl StandardMode {

    pub fn new() -> StandardMode {
        StandardMode {
            keymap: StandardMode::key_defaults(),
        }
    }

    fn key_defaults() -> KeyMap {
        let mut keymap = KeyMap::new();

        // Editor Commands
        keymap.bind_key(Key::Ctrl('q'), Command::ExitEditor);
        keymap.bind_key(Key::Ctrl('s'), Command::SaveBuffer);
        keymap.bind_keys(vec![Key::Ctrl('x'), Key::Ctrl('c')].as_slice(), Command::ExitEditor);
        keymap.bind_keys(vec![Key::Ctrl('x'), Key::Ctrl('s')].as_slice(), Command::SaveBuffer);

        // Navigation
        keymap.bind_key(Key::Up, Command::MoveCursor(Direction::Up(1)));
        keymap.bind_key(Key::Down, Command::MoveCursor(Direction::Down(1)));
        keymap.bind_key(Key::Left, Command::MoveCursor(Direction::Left(1)));
        keymap.bind_key(Key::Right, Command::MoveCursor(Direction::Right(1)));

        keymap.bind_key(Key::Ctrl('p'), Command::MoveCursor(Direction::Up(1)));
        keymap.bind_key(Key::Ctrl('n'), Command::MoveCursor(Direction::Down(1)));
        keymap.bind_key(Key::Ctrl('b'), Command::MoveCursor(Direction::Left(1)));
        keymap.bind_key(Key::Ctrl('f'), Command::MoveCursor(Direction::Right(1)));
        keymap.bind_key(Key::Ctrl('e'), Command::LineEnd);
        keymap.bind_key(Key::Ctrl('a'), Command::LineStart);

        // Editing
        keymap.bind_key(Key::Tab, Command::InsertTab);
        keymap.bind_key(Key::Enter, Command::InsertChar('\n'));
        keymap.bind_key(Key::Backspace, Command::Delete(Direction::Left(1)));
        keymap.bind_key(Key::Ctrl('h'), Command::Delete(Direction::Left(1)));
        keymap.bind_key(Key::Delete, Command::Delete(Direction::Right(1)));
        keymap.bind_key(Key::Ctrl('d'), Command::Delete(Direction::Right(1)));

        // History
        keymap.bind_key(Key::Ctrl('y'), Command::Redo);
        keymap.bind_key(Key::Ctrl('z'), Command::Undo);

        keymap
    }

    fn handle_command(&mut self, c: Command, view: &mut View) -> Response {
        match c {
            // Editor Commands
            Command::ExitEditor      => return Response::Quit,
            Command::SaveBuffer      => utils::save_buffer(&view.buffer),

            // Navigation
            Command::MoveCursor(dir) => view.move_cursor(dir),
            Command::LineEnd         => view.move_cursor_to_line_end(),
            Command::LineStart       => view.move_cursor_to_line_start(),

            // Editing
            Command::Delete(dir)     => view.delete_chars(dir),
            Command::InsertTab       => view.insert_tab(),
            Command::InsertChar(c)   => view.insert_char(c),
            Command::Redo            => view.redo(),
            Command::Undo            => view.undo(),

            _ => {},
        }
        Response::Continue
    }
}

impl Mode for StandardMode {
    fn handle_key_event(&mut self, key: Option<Key>, view: &mut View) -> EventStatus {
        let key = match key {
            Some(k) => k,
            None => return EventStatus::NotHandled
        };

        // send key to the keymap
        match self.keymap.check_key(key) {
            KeyMapState::Match(command) => {
                return EventStatus::Handled(self.handle_command(command, view));
            },
            KeyMapState::Continue => {
                // keep going and wait for more keypresses
                return EventStatus::Handled(Response::Continue)
            },
            KeyMapState::None => {}  // do nothing and handle the key normally
        }

        // if the key is a character that is not part of a keybinding, insert into the buffer
        // otherwise, ignore it.
        if let Key::Char(c) = key {
            view.insert_char(c);
            EventStatus::Handled(Response::Continue)
        } else {
            EventStatus::NotHandled
        }

    }

    fn interpret_input(&mut self, _input: String, _view: &mut View) -> Response {
        Response::Continue
    }
}
