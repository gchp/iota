use keyboard::Key;
use keymap::{ KeyMap, KeyMapState };
use buffer::{ Direction, Mark, WordEdgeMatch };
use textobject::{ TextObject, Reference, Kind, Anchor };
use overlay::OverlayType;

/// Instructions for the Editor.
/// These do NOT alter the text, but may change editor/view state
#[derive(Copy, Show)]
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
#[derive(Copy, Show)]
#[allow(dead_code)]
pub enum Operation {
    Insert,     // insert text
    Delete,     // delete some object

    Undo,       // rewind buffer transaction log
    Redo,       // replay buffer transaction log
}

/// Fragments that can be combined to specify a command
#[derive(Copy, Show)]
#[allow(dead_code)]
pub enum Partial {
    Kind(Kind),
    Anchor(Anchor),
    Reference(Reference),
    Object(TextObject),
    Action(Action),
}

#[derive(Copy, Show)]
#[allow(dead_code)]
pub enum Action {
    Operation(Operation),
    Instruction(Instruction),
}

/// A complete, actionable command
#[derive(Copy, Show)]
pub struct Command {
    pub number: i32,        // numeric paramter, line number, repeat count, etc.
    pub action: Action,     // what to do
    pub object: TextObject, // where to do it
}

pub struct Builder {
    number: Option<i32>,
    repeat: Option<usize>,

    action: Option<Action>,
    mark: Option<Mark>,
    kind: Option<Kind>,
    anchor: Option<Anchor>,
    reference: Option<Reference>,
    object: Option<TextObject>,

    reading_number: bool,
    keymap: KeyMap<Partial>,
}

#[derive(Copy, Show)]
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
            reference: None,
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
        self.reference = None;
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
        if let Some(to) = self.object {
            // we have a whole object ready
            return Some(to)
        } else if let Some(reference) = self.reference {
            // we have a particular object in mind, use number as index/offset
            let reference = if let Some(n) = self.number {
                // set the reference index to n, or multiply offset by n
                // also, reset the 'outer' number, since index/offset is separate from repeat counter
                self.number = None;
                match reference {
                    Reference::Index(k, _) => Reference::Index(k, n as u32),
                    Reference::Offset(m, k, i) => Reference::Offset(m, k, i * n),
                    Reference::Mark(m, k) => Reference::Mark(m, k)
                }
            } else { reference };
            self.reference = Some(reference);

            return Some(TextObject { anchor: self.anchor.unwrap_or(Anchor::default()),
                                     reference: reference })
        } else if let Some(kind) = self.kind {
            // we have a particular kind...
            if let Some(mark) = self.mark {
                // and a specific mark, return the object of that kind, at that mark
                return Some(TextObject { anchor: self.anchor.unwrap_or(Anchor::default()),
                                         reference: Reference::Mark(mark, kind) })
            } else if let Some(n) = self.number {
                // and a particular number, return the nth of that kind
                return Some(TextObject { anchor: self.anchor.unwrap_or(Anchor::default()),
                                         reference: Reference::Index(kind, n as u32) })
            } else {
                // just the kind, nothing else, find next instance from cursor
                return Some(TextObject { anchor: self.anchor.unwrap_or(Anchor::default()),
                                         reference: Reference::Offset(Mark::Cursor(0), kind, 1) })
            }
        }
        None
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
                object: self.complete_object().unwrap_or(TextObject {
                    anchor: Anchor::default(),
                    reference: Reference::default()
                })
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
            Partial::Reference(r) => self.reference = Some(r),
            Partial::Object(o)    => self.object = Some(o),
            Partial::Action(a)    => { 
                self.action = Some(a);
                if !self.reading_number && self.number.is_some() && self.repeat.is_none() {
                    self.repeat = Some(self.number.unwrap() as usize);
                    self.number = None;
                }
            }
        }
    }
}

fn default_keymap() -> KeyMap<Partial> {
    let mut keymap = KeyMap::new();

    // next/previous char
    keymap.bind_key(Key::Char('l'), Partial::Reference(Reference::Offset(Mark::Cursor(0), Kind::Char,  1)));
    keymap.bind_key(Key::Char('h'), Partial::Reference(Reference::Offset(Mark::Cursor(0), Kind::Char, -1)));

    // next/previous word
    keymap.bind_key(Key::Char('w'), Partial::Reference(Reference::Offset(Mark::Cursor(0), Kind::Word(WordEdgeMatch::Alphabet),  1)));
    keymap.bind_key(Key::Char('b'), Partial::Reference(Reference::Offset(Mark::Cursor(0), Kind::Word(WordEdgeMatch::Alphabet), -1)));
    keymap.bind_key(Key::Char('W'), Partial::Reference(Reference::Offset(Mark::Cursor(0), Kind::Word(WordEdgeMatch::Whitespace),  1)));
    keymap.bind_key(Key::Char('B'), Partial::Reference(Reference::Offset(Mark::Cursor(0), Kind::Word(WordEdgeMatch::Whitespace), -1)));

    // next/previous line
    keymap.bind_key(Key::Char('j'), Partial::Reference(Reference::Offset(Mark::Cursor(0), Kind::Line,  1)));
    keymap.bind_key(Key::Char('k'), Partial::Reference(Reference::Offset(Mark::Cursor(0), Kind::Line, -1)));

    // start/end of line
    keymap.bind_key(Key::Char('0'), Partial::Object(TextObject { anchor: Anchor::Before, 
                                                                 reference: Reference::Mark(Mark::Cursor(0), Kind::Line) }));
    keymap.bind_key(Key::Char('$'), Partial::Object(TextObject { anchor: Anchor::After, 
                                                                 reference: Reference::Mark(Mark::Cursor(0), Kind::Line) }));

    // kinds
    keymap.bind_keys(&[Key::Char('`'), Key::Char('c')], Partial::Kind(Kind::Char));
    keymap.bind_keys(&[Key::Char('`'), Key::Char('w')], Partial::Kind(Kind::Word(WordEdgeMatch::Alphabet)));
    keymap.bind_keys(&[Key::Char('`'), Key::Char('W')], Partial::Kind(Kind::Word(WordEdgeMatch::Whitespace)));
    keymap.bind_keys(&[Key::Char('`'), Key::Char('l')], Partial::Kind(Kind::Line));

    // anchors
    keymap.bind_key(Key::Char(','), Partial::Anchor(Anchor::Before));
    keymap.bind_key(Key::Char('.'), Partial::Anchor(Anchor::After));

    // actions
    keymap.bind_key(Key::Char('d'), Partial::Action(Action::Operation(Operation::Delete)));
    keymap.bind_key(Key::Char('g'), Partial::Action(Action::Instruction(Instruction::SetMark(Mark::Cursor(0)))));

    keymap.bind_key(Key::Char(':'), Partial::Action(Action::Instruction(Instruction::SetOverlay(OverlayType::Prompt))));

    keymap
}
