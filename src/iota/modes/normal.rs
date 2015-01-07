use super::Mode;
use super::KeyMap;
use super::Key;
use super::Command;
use super::View;
use super::KeyMapState;
use super::EventStatus;
use super::Direction;
use super::WordEdgeMatch;
use super::Response;
use super::utils;
use super::{Overlay, OverlayType, OverlayEvent};


pub struct NormalMode {
    keymap: KeyMap,
}

impl NormalMode {

    pub fn new() -> NormalMode {
        NormalMode {
            keymap: NormalMode::key_defaults(),
        }
    }

    fn key_defaults() -> KeyMap {
        let mut keymap = KeyMap::new();

        // movement
        keymap.bind_key(Key::Up, Command::MoveCursor(Direction::Up(1)));
        keymap.bind_key(Key::Down, Command::MoveCursor(Direction::Down(1)));
        keymap.bind_key(Key::Left, Command::MoveCursor(Direction::Left(1)));
        keymap.bind_key(Key::Right, Command::MoveCursor(Direction::Right(1)));
        keymap.bind_key(Key::Char('h'), Command::MoveCursor(Direction::Left(1)));
        keymap.bind_key(Key::Char('j'), Command::MoveCursor(Direction::Down(1)));
        keymap.bind_key(Key::Char('k'), Command::MoveCursor(Direction::Up(1)));
        keymap.bind_key(Key::Char('l'), Command::MoveCursor(Direction::Right(1)));
        keymap.bind_key(Key::Char('W'), Command::MoveCursor(Direction::RightWord(1, WordEdgeMatch::Whitespace)));
        keymap.bind_key(Key::Char('B'), Command::MoveCursor(Direction::LeftWord(1, WordEdgeMatch::Whitespace)));
        keymap.bind_key(Key::Char('w'), Command::MoveCursor(Direction::RightWord(1, WordEdgeMatch::Alphabet)));
        keymap.bind_key(Key::Char('b'), Command::MoveCursor(Direction::LeftWord(1, WordEdgeMatch::Alphabet)));
        keymap.bind_key(Key::Char('0'), Command::LineStart);
        keymap.bind_key(Key::Char('$'), Command::LineEnd);

        // editing
        keymap.bind_keys(&[Key::Char('d'), Key::Char('W')], Command::Delete(Direction::RightWord(1, WordEdgeMatch::Whitespace)));
        keymap.bind_keys(&[Key::Char('d'), Key::Char('B')], Command::Delete(Direction::LeftWord(1, WordEdgeMatch::Whitespace)));
        keymap.bind_keys(&[Key::Char('d'), Key::Char('w')], Command::Delete(Direction::RightWord(1, WordEdgeMatch::Alphabet)));
        keymap.bind_keys(&[Key::Char('d'), Key::Char('b')], Command::Delete(Direction::LeftWord(1, WordEdgeMatch::Alphabet)));
        keymap.bind_key(Key::Char('x'), Command::Delete(Direction::Right(1)));
        keymap.bind_key(Key::Char('X'), Command::Delete(Direction::Left(1)));
        keymap.bind_key(Key::Char('u'), Command::Undo);
        keymap.bind_key(Key::Ctrl('r'), Command::Redo);

        // open a prompt to the user
        keymap.bind_key(Key::Char(':'), Command::SetOverlay(OverlayType::Prompt));

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
            Command::Redo            => view.redo(),
            Command::Undo            => view.undo(),

            // Prompt
            Command::SetOverlay(o)   => view.set_overlay(o),
            _                        => {}
        }
        Response::Continue
    }
}

impl Mode for NormalMode {
    fn handle_key_event(&mut self, key: Option<Key>, view: &mut View) -> EventStatus {
        let key = match key {
            Some(k) => k,
            None => return EventStatus::NotHandled
        };

        // if there is an overlay on the view, send the key there
        // and don't allow the mode to handle it.
        match view.overlay {
            Overlay::None => {},
            _             => {
                let event = view.overlay.handle_key_event(key);
                if let OverlayEvent::Finished(response) = event {
                    view.overlay = Overlay::None;
                    if let Some(data) = response {
                        return EventStatus::Handled(self.interpret_command_input(data, view))
                    }
                }
                return EventStatus::NotHandled
            }
        }

        // send key to the keymap
        match self.keymap.check_key(key) {
            KeyMapState::Match(command) => {
                EventStatus::Handled(self.handle_command(command, view))
            },
            KeyMapState::Continue => {
                // keep going and wait for more keypresses
                EventStatus::Handled(Response::Continue)
            },
            KeyMapState::None => { EventStatus::NotHandled }
        }

    }

    fn interpret_command_input(&mut self, input: String, view: &mut View) -> Response {
        let command = Command::from_str(&*input);
        self.handle_command(command, view)
    }
}
