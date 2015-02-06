use std::default::Default;

use keyboard::Key;
use keymap::{ KeyMap, KeyMapState };
use buffer::{ Direction, Mark, WordEdgeMatch };
use textobject::{ TextObject, Offset, Kind, Anchor };
use overlay::OverlayType;

/// Instructions for the Editor.
/// These do NOT alter the text, but may change editor/view state
#[derive(Copy, Debug)]
#[allow(dead_code)]
pub enum Instruction {
    SaveBuffer,
    //FindFile,
    ExitEditor,

    SetMark(Mark),
    SetOverlay(OverlayType),
    // SetMode(...)
}

/// Operations on the Buffer.
/// These DO alter the text, but otherwise may NOT change editor/view state
/// Note that these differ from log::Change in that they are higher-level
/// operations dependent on state (cursor/mark locations, etc.), as opposed
/// to concrete operations on absolute indexes (insert 'a' at index 158, etc.)
#[derive(Copy, Debug)]
#[allow(dead_code)]
pub enum Operation {
    Insert(char), // insert text
    Delete,       // delete some object

    Undo,         // rewind buffer transaction log
    Redo,         // replay buffer transaction log
}

/// Fragments that can be combined to specify a command
#[derive(Copy, Debug)]
#[allow(dead_code)]
pub enum Partial {
    Kind(Kind),
    Anchor(Anchor),
    Offset(Offset),
    Object(TextObject),
    Action(Action),
}

#[derive(Copy, Debug)]
#[allow(dead_code)]
pub enum Action {
    Operation(Operation),
    Instruction(Instruction),
}

/// A complete, actionable command
#[derive(Copy, Debug)]
pub struct Command {
    pub number: i32,        // numeric paramter, line number, repeat count, etc.
    pub action: Action,     // what to do
    pub object: TextObject, // where to do it
}

impl Command {
    /// Shortcut to create an ExitEditor command
    pub fn exit_editor() -> Command {
        Command {
            action: Action::Instruction(Instruction::ExitEditor),
            number: 0,
            object: TextObject {
                kind: Kind::Char,
                offset: Offset::Absolute(0),
            },
        }
    }

    /// Shortcut to create a SaveBuffer command
    pub fn save_buffer() -> Command {
        Command {
            action: Action::Instruction(Instruction::SaveBuffer),
            number: 0,
            object: TextObject {
                kind: Kind::Char,
                offset: Offset::Absolute(0),
            },
        }
    }
}

pub struct Builder {
    number: Option<i32>,
    repeat: Option<usize>,

    action: Option<Action>,
    mark: Option<Mark>,
    kind: Option<Kind>,
    anchor: Option<Anchor>,
    offset: Option<Offset>,
    object: Option<TextObject>,

    reading_number: bool,
    keymap: KeyMap<Partial>,
}

#[derive(Copy, Debug)]
pub enum BuilderEvent {
    Invalid,            // cannot find a valid interpretation
    Incomplete,         // needs more information
    Complete(Command),  // command is finished
}

impl Builder {
    pub fn new() -> Builder {
        Builder {
            number: None,
            repeat: None,
            action: None,
            mark: None,
            kind: None,
            anchor: None,
            offset: None,
            object: None,
            reading_number: false,
            keymap: default_keymap()
        }
    }

    pub fn reset(&mut self) {
        self.number = None;
        self.repeat = None;
        self.action = None;
        self.mark = None;
        self.kind = None;
        self.anchor = None;
        self.object = None;
        self.offset = None;
        self.reading_number = false;
    }

    pub fn check_key(&mut self, key: Key) -> BuilderEvent {
        if let Key::Char(c) = key {
            // '0' might be bound (start of line), and cannot be the start of a number sequence
            if c.is_digit(10) && (self.reading_number || c != '0') {
                let n = c.to_digit(10).unwrap();
                self.reading_number = true;
                self.append_digit(n);
                return BuilderEvent::Incomplete;
            } else if self.reading_number {
                
                self.reading_number = false;
            }
        }

        match self.keymap.check_key(key) {
            KeyMapState::Match(partial) => self.apply_partial(partial),
            KeyMapState::None           => { self.reset(); return BuilderEvent::Invalid; }
            _ => {},
        }
        
        if let Some(c) = self.complete_command() {
            self.reset();
            return BuilderEvent::Complete(c)
        } else {
            return BuilderEvent::Incomplete
        }
    }

    fn complete_object(&mut self) -> Option<TextObject> {
        let mut result: Option<TextObject> = if let Some(object) = self.object {
            // we have a complete object ready to go
            Some(object)
        } else if let Some(kind) = self.kind {
            // we have at least a kind
            Some(TextObject {
                kind: kind,
                offset: self.offset.unwrap_or(Offset::Absolute(0)),
            })
        } else {
            None
        };

        if let Some(ref mut object) = result {
            if let Some(number) = self.number {
                object.offset = object.offset.with_num(number as usize);
            }
        }

        result
    }

    fn complete_command(&mut self) -> Option<Command> {
        /* rules for completing commands:
              | action            | number | object | reference    | kind |   | result                                                            |
              | -                 | -      | -      | -            | -    | - | -                                                                 |
              | no                | no     | no     | no           | yes  |   | move cursor to next (default) text object with kind               |
              | no                | no     | no     | yes          | -    |   | move cursor to text object with reference + default anchor        |
              | no                | no     | yes    | -            | -    |   | move cursor to text object                                        |
              | no                | yes    | no     | no           | no   |   | incomplete                                                        |
              | no                | yes    | no     | no           | yes  |   | move cursor to nth instance of kind (from start of buffer)        |
              | no                | yes    | no     | yes (index)  | -    |   | set index to number, cursor to ref + default anchor               |
              | no                | yes    | no     | yes (offset) | -    |   | multiply offset by number, cursor to ref + default anchor         |
              | no                | yes    | no     | yes (mark)   | -    |   | incomplete                                                        |
              | yes (instruction) | -      | -      | -            | -    |   | send instruction to editor with whatever parameters are available |
              | yes (operation)   | no     | no     | no           | no   |   | incomplete                                                        |
              | yes (operation)   | no     | no     | no           | yes  |   | apply operation to kind at cursor (default anchor)                |
              | yes (operation)   | no     | no     | yes          | -    |   | apply operation to reference with default anchor                  |
              | yes (operation)   | no     | yes    | -            | -    |   | apply operation to object                                         |
              | yes (operation)   | yes    | no     | no           | no   |   | incomplete                                                        |
              | yes (operation)   | yes    | no     | no           | yes  |   | apply operation to nth instance of kind                           |
        */
        
        // editor instructions may not need a text object, go ahead and return immediately
        if let Some(Action::Instruction(i)) = self.action {
            return Some(Command {
                number: self.repeat.unwrap_or(0) as i32,
                action: Action::Instruction(i),
                object: self.complete_object().unwrap_or_default()
            });
        }

        if let Some(to) = self.complete_object() {
            if let Some(Action::Operation(o)) = self.action {
                // we have an object, and an operation
                return Some(Command {
                    number: self.repeat.unwrap_or(0) as i32,
                    action: Action::Operation(o),
                    object: to
                });
            } else {
                // we have just an object, assume move cursor instruction
                return Some(Command {
                    number: self.repeat.unwrap_or(0) as i32,
                    action: Action::Instruction(Instruction::SetMark(Mark::Cursor(0))),
                    object: to
                });
            }
        }
        None
    }

    fn append_digit(&mut self, n: usize) {
        if let Some(current) = self.number {
            self.number = Some((current*10) + n as i32);
        } else {
            self.number = Some(n as i32);
        }
    }

    fn apply_partial(&mut self, partial: Partial) {
        match partial {
            Partial::Kind(k)      => self.kind = Some(k),
            Partial::Anchor(a)    => self.anchor = Some(a),
            Partial::Offset(o)    => self.offset = Some(o),
            Partial::Object(o)    => self.object = Some(o),
            Partial::Action(a)    => { 
                self.action = Some(a);
                if !self.reading_number && self.number.is_some() && self.repeat.is_none() {
                    // the first number followed by an action is treated as a repetition
                    self.repeat = Some(self.number.unwrap() as usize);
                    self.number = None;
                }
            }
        }
        
        // propagate upwards from anchor to object
        // if both an object(a) and an anchor(b) have been applied, the resulting
        // object should be exactly the same as (a), only using (b) as the anchor
        if let Some(anchor) = self.anchor {
            if let Some(kind) = self.kind {
                self.kind = Some(kind.with_anchor(anchor));
            }
            if let Some(object) = self.object {
                self.object = Some(TextObject {
                    kind: object.kind.with_anchor(anchor),
                    offset: object.offset,
                });
            }
        }        
        if let Some(kind) = self.kind {
            if let Some(ref mut object) = self.object {
                object.kind = kind;
            }
        }
        if let Some(offset) = self.offset {
            if let Some(ref mut object) = self.object {
                object.offset = offset;
            }
        }

        // propagate downwards from object to unset partials
        if let Some(object) = self.object {
            if self.offset.is_none() { self.offset = Some(object.offset); }
            if self.kind.is_none() { self.kind = Some(object.kind); }
            if self.anchor.is_none() { self.anchor = Some(object.kind.get_anchor()); }
        }

        if self.offset.is_some() && self.kind.is_some() && self.object.is_none() {
            self.object = Some(TextObject {
                kind: self.kind.unwrap(),
                offset: self.offset.unwrap(),
            });
        }
    }
}

fn default_keymap() -> KeyMap<Partial> {
    let mut keymap = KeyMap::new();

    // next/previous char
    keymap.bind_key(Key::Char('l'), Partial::Object(TextObject {
        kind: Kind::Char,
        offset: Offset::Forward(1, Mark::Cursor(0))
    }));
    keymap.bind_key(Key::Char('h'), Partial::Object(TextObject {
        kind: Kind::Char,
        offset: Offset::Backward(1, Mark::Cursor(0))
    }));

    // next/previous line
    keymap.bind_key(Key::Char('j'), Partial::Object(TextObject {
        kind: Kind::Line(Anchor::Same),
        offset: Offset::Forward(1, Mark::Cursor(0))
    }));
    keymap.bind_key(Key::Char('k'), Partial::Object(TextObject {
        kind: Kind::Line(Anchor::Same),
        offset: Offset::Backward(1, Mark::Cursor(0))
    }));

    // next/previous word
    keymap.bind_key(Key::Char('w'), Partial::Object(TextObject {
        kind: Kind::Word(Anchor::Before),
        offset: Offset::Forward(1, Mark::Cursor(0))
    }));
    keymap.bind_key(Key::Char('b'), Partial::Object(TextObject {
        kind: Kind::Word(Anchor::Before),
        offset: Offset::Backward(1, Mark::Cursor(0))
    }));

    // start/end line
    keymap.bind_key(Key::Char('$'), Partial::Object(TextObject {
        kind: Kind::Line(Anchor::End),
        offset: Offset::Forward(1, Mark::Cursor(0)),
    }));
    keymap.bind_key(Key::Char('^'), Partial::Object(TextObject {
        kind: Kind::Line(Anchor::Start),
        offset: Offset::Forward(1, Mark::Cursor(0)),
    }));

    // kinds
    keymap.bind_keys(&[Key::Char('`'), Key::Char('c')], Partial::Kind(Kind::Char));
    keymap.bind_keys(&[Key::Char('`'), Key::Char('w')], Partial::Kind(Kind::Word(Anchor::Before)));
    keymap.bind_keys(&[Key::Char('`'), Key::Char('l')], Partial::Kind(Kind::Line(Anchor::Before)));

    // anchors
    keymap.bind_key(Key::Char(','), Partial::Anchor(Anchor::Before));
    keymap.bind_key(Key::Char('.'), Partial::Anchor(Anchor::After));

    // actions
    keymap.bind_key(Key::Char('d'), Partial::Action(Action::Operation(Operation::Delete)));
    keymap.bind_key(Key::Char(':'), Partial::Action(Action::Instruction(Instruction::SetOverlay(OverlayType::Prompt))));

    keymap
}
