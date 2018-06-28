use buffer::Mark;
use textobject::{ TextObject, Offset, Kind };
use overlay::OverlayType;
use modes::ModeType;
use keymap::CommandInfo;

/// Instructions for the Editor.
/// These do NOT alter the text, but may change editor/view state
#[derive(Clone)]
pub enum Instruction {
    SaveBuffer,
    //FindFile,
    ExitEditor,

    SetMark(Mark),
    SetOverlay(OverlayType),
    SetMode(ModeType),
    ShowMessage(String),
    SwitchToLastBuffer,
    None,
}

/// Operations on the Buffer.
/// These DO alter the text, but otherwise may NOT change editor/view state
/// Note that these differ from `log::Change` in that they are higher-level
/// operations dependent on state (cursor/mark locations, etc.), as opposed
/// to concrete operations on absolute indexes (insert 'a' at index 158, etc.)
#[derive(Clone)]
pub enum Operation {
    Insert(char), // insert text
    DeleteObject,         // delete some object
    DeleteFromMark(Mark), // delete from some mark to an object

    Undo,         // rewind buffer transaction log
    Redo,         // replay buffer transaction log
}

#[derive(Clone)]
pub enum Action {
    Operation(Operation),
    Instruction(Instruction),
}

/// A complete, actionable command
#[derive(Clone)]
pub struct Command {
    pub number: i32,        // numeric paramter, line number, repeat count, etc.
    pub action: Action,     // what to do
    pub object: Option<TextObject>, // where to do it
}

impl Command {
    /// Display a message
    pub fn show_message(args: Option<BuilderArgs>) -> Command {
        let args = args.expect("no arguments given to show_message");
        let message = args.str_args.expect("no message provided");
        Command {
            action: Action::Instruction(Instruction::ShowMessage(message)),
            number: 0,
            object: None,
        }
    }

    /// Shortcut to create an ExitEditor command
    pub fn exit_editor(_args: Option<BuilderArgs>) -> Command {
        Command {
            action: Action::Instruction(Instruction::ExitEditor),
            number: 0,
            object: None,
        }
    }

    /// Shortcut to create a SaveBuffer command
    pub fn save_buffer(_args: Option<BuilderArgs>) -> Command {
        Command {
            action: Action::Instruction(Instruction::SaveBuffer),
            number: 0,
            object: None,
        }
    }

    /// Shortcut to create SetMode command
    pub fn set_mode(args: Option<BuilderArgs>) -> Command {
        let args = args.expect("no arguments given to set_mode");
        let mode_type = args.mode_args.expect("no mode type given");
        Command {
            action: Action::Instruction(Instruction::SetMode(mode_type)),
            number: 0,
            object: None,
        }
    }

    /// Shortcut to change the active overlay
    pub fn set_overlay(args: Option<BuilderArgs>) -> Command {
        let args = args.expect("no arguments given to set_overlay");
        let overlay = args.overlay_args.expect("no overlay type given");
        Command {
            action: Action::Instruction(Instruction::SetOverlay(overlay)),
            number: 0,
            object: None,
        }
    }

    /// Shortcut to create a Delete command
    pub fn delete_char(args: Option<BuilderArgs>) -> Command {
        let args = args.expect("no arguments given to delete_char");
        let kind = args.kind.expect("no kind provided");
        let offset = args.offset.expect("no offset provided");
        Command {
            number: 1,
            action: Action::Operation(Operation::DeleteFromMark(Mark::Cursor(0))),
            object: Some(TextObject {
                kind: kind,
                offset: offset
            })
        }
    }

    /// Shortcut to create an Insert command
    pub fn insert_char(args: Option<BuilderArgs>) -> Command {
        let args = args.expect("no arguments given to insert_char");
        let c = args.char_args.expect("no char given");
        Command {
            number: 1,
            action: Action::Operation(Operation::Insert(c)),
            object: None,
        }
    }

    /// Shortcut to create a Delete command
    pub fn delete(args: Option<BuilderArgs>) -> Command {
        let args = args.expect("no arguments given to insert_char");
        let kind = args.kind.expect("no kind provided");
        let offset = args.offset.expect("no offset provided");
        let repeat = args.number.unwrap_or(1);
        Command {
            number: repeat,
            action: Action::Operation(Operation::DeleteFromMark(Mark::Cursor(0))),
            object: Some(TextObject {
                kind: kind,
                offset: offset,
            }),
        }
    }

    /// Shortcut to create an Insert command
    // FIXME: shouldn't need this method
    pub fn insert_tab(_args: Option<BuilderArgs>) -> Command {
        Command {
            number: 4,
            action: Action::Operation(Operation::Insert(' ')),
            object: None,
        }
    }

    /// Shortcut to create Undo command
    pub fn undo(_args: Option<BuilderArgs>) -> Command {
        Command {
            number: 1,
            action: Action::Operation(Operation::Undo),
            object: None
        }
    }

    /// Shortcut to create Redo command
    pub fn redo(_args: Option<BuilderArgs>) -> Command {
        Command {
            number: 1,
            action: Action::Operation(Operation::Redo),
            object: None
        }
    }

    pub fn move_cursor(args: Option<BuilderArgs>) -> Command {
        let args = args.expect("no args given to movement");
        let kind = args.kind.expect("no kind provided");
        let offset = args.offset.expect("no offset provided");
        let repeat = args.number.unwrap_or(1);
        Command {
            number: repeat,
            action: Action::Instruction(Instruction::SetMark(Mark::Cursor(0))),
            object: Some(TextObject {
                kind: kind,
                offset: offset
            })
        }
    }

    pub fn noop(_args: Option<BuilderArgs>) -> Command {
        Command {
            number: 0,
            action: Action::Instruction(Instruction::None),
            object: None,
        }
    }
}

#[derive(Clone)]
pub struct BuilderArgs {
    pub char_args: Option<char>,
    pub number: Option<i32>,
    pub str_args: Option<String>,
    pub mode_args: Option<ModeType>,
    pub overlay_args: Option<OverlayType>,
    pub kind: Option<Kind>,
    pub offset: Option<Offset>,
}

impl BuilderArgs {
    pub fn new() -> BuilderArgs {
        BuilderArgs {
            char_args: None,
            number: None,
            str_args: None,
            mode_args: None,
            overlay_args: None,
            kind: None,
            offset: None,
        }
    }

    pub fn with_char_arg(mut self, c: char) -> BuilderArgs {
        self.char_args = Some(c);

        self
    }

    pub fn with_str(mut self, s: String) -> BuilderArgs {
        self.str_args = Some(s);

        self
    }

    pub fn with_number(mut self, n: i32) -> BuilderArgs {
        self.number = Some(n);

        self
    }

    pub fn with_kind(mut self, kind: Kind) -> BuilderArgs {
        self.kind = Some(kind);

        self
    }

    pub fn with_offset(mut self, offset: Offset) -> BuilderArgs {
        self.offset = Some(offset);

        self
    }

    pub fn with_mode(mut self, mode: ModeType) -> BuilderArgs {
        self.mode_args = Some(mode);

        self
    }

    pub fn with_overlay(mut self, overlay: OverlayType) -> BuilderArgs {
        self.overlay_args = Some(overlay);

        self
    }
}


pub enum BuilderEvent {
    Invalid,            // cannot find a valid interpretation
    Incomplete,         // needs more information
    Complete(CommandInfo),  // command is finished
}
