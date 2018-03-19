use buffer::Mark;
use textobject::{ TextObject, Offset, Kind };
use overlay::OverlayType;
use modes::ModeType;

/// Instructions for the Editor.
/// These do NOT alter the text, but may change editor/view state
#[derive(Copy, Clone, Debug)]
pub enum Instruction {
    SaveBuffer,
    //FindFile,
    ExitEditor,

    SetMark(Mark),
    SetOverlay(OverlayType),
    SetMode(ModeType),
    ShowMessage(&'static str),
    SwitchToLastBuffer,
    None,
}

/// Operations on the Buffer.
/// These DO alter the text, but otherwise may NOT change editor/view state
/// Note that these differ from `log::Change` in that they are higher-level
/// operations dependent on state (cursor/mark locations, etc.), as opposed
/// to concrete operations on absolute indexes (insert 'a' at index 158, etc.)
#[derive(Copy, Clone, Debug)]
pub enum Operation {
    Insert(char), // insert text
    DeleteObject,         // delete some object
    DeleteFromMark(Mark), // delete from some mark to an object

    Undo,         // rewind buffer transaction log
    Redo,         // replay buffer transaction log
}

#[derive(Copy, Clone, Debug)]
pub enum Action {
    Operation(Operation),
    Instruction(Instruction),
}

/// A complete, actionable command
#[derive(Copy, Clone, Debug)]
pub struct Command {
    pub number: i32,        // numeric paramter, line number, repeat count, etc.
    pub action: Action,     // what to do
    pub object: Option<TextObject>, // where to do it
}

impl Command {
    /// Display a message
    pub fn show_message(msg: &'static str) -> Command {
        Command {
            action: Action::Instruction(Instruction::ShowMessage(msg)),
            number: 0,
            object: None,
        }
    }

    /// Shortcut to create an ExitEditor command
    pub fn exit_editor() -> Command {
        Command {
            action: Action::Instruction(Instruction::ExitEditor),
            number: 0,
            object: None,
        }
    }

    /// Shortcut to create a SaveBuffer command
    pub fn save_buffer() -> Command {
        Command {
            action: Action::Instruction(Instruction::SaveBuffer),
            number: 0,
            object: None,
        }
    }

    /// Shortcut to create SetMode command
    pub fn set_mode(mode: ModeType) -> Command {
        Command {
            action: Action::Instruction(Instruction::SetMode(mode)),
            number: 0,
            object: None,
        }
    }

    /// Shortcut to change the active overlay
    pub fn set_overlay(overlay: OverlayType) -> Command {
        Command {
            action: Action::Instruction(Instruction::SetOverlay(overlay)),
            number: 0,
            object: None,
        }
    }

    /// Shortcut to create an Insert command
    pub fn insert_char(c: char) -> Command {
        Command {
            number: 1,
            action: Action::Operation(Operation::Insert(c)),
            object: None,
        }
    }

    /// Shortcut to create an Insert command
    // FIXME: shouldn't need this method
    pub fn insert_tab() -> Command {
        Command {
            number: 4,
            action: Action::Operation(Operation::Insert(' ')),
            object: None,
        }
    }

    /// Shortcut to create Undo command
    pub fn undo() -> Command {
        Command {
            number: 1,
            action: Action::Operation(Operation::Undo),
            object: None
        }
    }

    /// Shortcut to create Redo command
    pub fn redo() -> Command {
        Command {
            number: 1,
            action: Action::Operation(Operation::Redo),
            object: None
        }
    }

    pub fn movement(offset: Offset, kind: Kind) -> Command {
        Command {
            number: 1,
            action: Action::Instruction(Instruction::SetMark(Mark::Cursor(0))),
            object: Some(TextObject {
                kind: kind,
                offset: offset
            })
        }
    }

    pub fn noop() -> Command {
        Command {
            number: 0,
            action: Action::Instruction(Instruction::None),
            object: None,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum BuilderEvent {
    Invalid,            // cannot find a valid interpretation
    Incomplete,         // needs more information
    Complete(Command),  // command is finished
}
