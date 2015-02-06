use super::Mode;
use super::KeyMap;
use super::Key;
use super::KeyMapState;
use buffer::Mark;
use command::{BuilderEvent, Operation, Instruction, Command, Action};
use textobject::{Anchor, Kind, TextObject, Offset};


// TODO: move this somewhere else - probably command module
fn movement(offset: Offset, kind: Kind) -> Command {
    Command {
        number: 1,
        action: Action::Instruction(Instruction::SetMark(Mark::Cursor(0))),
        object: Some(TextObject {
            kind: kind,
            offset: offset
        })
    }

}

/// Standard mode is Iota's default mode.
///
/// Standard mode uses non-vi-like keybindings.
/// Unlike Normal, Command and Visual modes which are all used together, Standard
/// mode is used on its own.
///
/// Standard mode allows Iota to be used in a non-modal way, similar to mainstream
/// editors like emacs or sublime.
pub struct StandardMode {
    keymap: KeyMap<Command>,
}

impl StandardMode {

    /// Create a new instance of StandardMode
    pub fn new() -> StandardMode {
        StandardMode {
            keymap: StandardMode::key_defaults(),
        }
    }

    /// Creates a KeyMap with default StandardMode key bindings
    fn key_defaults() -> KeyMap<Command> {
        let mut keymap = KeyMap::new();

        // Editor Commands
        keymap.bind_key(Key::Ctrl('q'), Command::exit_editor());
        keymap.bind_key(Key::Ctrl('s'), Command::save_buffer());
        keymap.bind_keys(vec![Key::Ctrl('x'), Key::Ctrl('c')].as_slice(), Command::exit_editor());
        keymap.bind_keys(vec![Key::Ctrl('x'), Key::Ctrl('s')].as_slice(), Command::save_buffer());

        // Cursor movement
        keymap.bind_key(Key::Up, movement(Offset::Backward(1, Mark::Cursor(0)), Kind::Line(Anchor::Same)));
        keymap.bind_key(Key::Down, movement(Offset::Forward(1, Mark::Cursor(0)), Kind::Line(Anchor::Same)));
        keymap.bind_key(Key::Left, movement(Offset::Backward(1, Mark::Cursor(0)), Kind::Char));
        keymap.bind_key(Key::Right, movement(Offset::Forward(1, Mark::Cursor(0)), Kind::Char));
        keymap.bind_key(Key::Ctrl('p'), movement(Offset::Backward(1, Mark::Cursor(0)), Kind::Line(Anchor::Same)));
        keymap.bind_key(Key::Ctrl('n'), movement(Offset::Forward(1, Mark::Cursor(0)), Kind::Line(Anchor::Same)));
        keymap.bind_key(Key::Ctrl('b'), movement(Offset::Backward(1, Mark::Cursor(0)), Kind::Char));
        keymap.bind_key(Key::Ctrl('f'), movement(Offset::Forward(1, Mark::Cursor(0)), Kind::Char));
        keymap.bind_key(Key::Ctrl('e'), movement(Offset::Forward(1, Mark::Cursor(0)), Kind::Line(Anchor::End)));
        keymap.bind_key(Key::Ctrl('a'), movement(Offset::Forward(1, Mark::Cursor(0)), Kind::Line(Anchor::Start)));

        // Editing
        keymap.bind_key(Key::Tab, Command::insert_tab());
        keymap.bind_key(Key::Enter, Command::insert_char('\n'));
        keymap.bind_key(Key::Backspace, Command {
            number: 1,
            action: Action::Operation(Operation::Delete),
            object: Some(TextObject {
                kind: Kind::Char,
                offset: Offset::Backward(1, Mark::Cursor(0))
            })
        });
        keymap.bind_key(Key::Delete, Command {
            number: 1,
            action: Action::Operation(Operation::Delete),
            object: Some(TextObject {
                kind: Kind::Char,
                offset: Offset::Forward(1, Mark::Cursor(0))
            })
        });
        keymap.bind_key(Key::Ctrl('h'), Command {
            number: 1,
            action: Action::Operation(Operation::Delete),
            object: Some(TextObject {
                kind: Kind::Char,
                offset: Offset::Backward(1, Mark::Cursor(0))
            })
        });
        keymap.bind_key(Key::Ctrl('d'), Command {
            number: 1,
            action: Action::Operation(Operation::Delete),
            object: Some(TextObject {
                kind: Kind::Char,
                offset: Offset::Forward(1, Mark::Cursor(0))
            })
        });

        // History
        keymap.bind_key(Key::Ctrl('z'), Command::undo());
        keymap.bind_key(Key::Ctrl('y'), Command::redo());

        keymap
    }

}

impl Mode for StandardMode {
    /// Given a key, pass it through the StandardMode KeyMap and return the associated Command, if any.
    /// If no match is found, treat it as an InsertChar command.
    fn handle_key_event(&mut self, key: Key) -> BuilderEvent {

        if let Key::Char(c) = key {
            BuilderEvent::Complete(Command::insert_char(c))
        } else {
            if let KeyMapState::Match(c) = self.keymap.check_key(key) {
                BuilderEvent::Complete(c)
            } else {
                BuilderEvent::Incomplete
            }
        }
    }
}
