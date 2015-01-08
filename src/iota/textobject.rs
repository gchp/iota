use buffer::{ Direction, Mark, WordEdgeMatch };

#[derive(Copy, Show)]
#[allow(dead_code)]
pub enum Reference {
    Index(Kind, u32),           // Absolute buffer index, "nth char/word/line/etc."
    Offset(Mark, Kind, i32),    // Relative buffer index, "nth char/word/line/etc. from cursor"
    Mark(Mark, Kind),           // For convenience, the object at mark
}

impl Reference {
    pub fn default() -> Reference {
        Reference::Mark(Mark::Cursor(0), Kind::default())
    }
}

#[derive(Copy, Show)]
#[allow(dead_code)]
pub enum Kind {
    Char,
    Line,

    Word(WordEdgeMatch),
    // Sentence,
    // Paragraph,

    // Expression,
    // Statement,
    // Block,
}

impl Kind {
    pub fn default() -> Kind {
        Kind::Char
    }
}

#[derive(Copy, Show)]
#[allow(dead_code)]
pub enum Anchor {
    Before,
    // Start,
    // Middle,
    // End,
    After
}

impl Anchor {
    pub fn default() -> Anchor {
        Anchor::Before
    }
}

#[derive(Copy, Show)]
pub struct TextObject {
    pub anchor: Anchor,
    pub reference: Reference
}

impl TextObject {
    pub fn default() -> TextObject {
        TextObject {
            anchor: Anchor::default(),
            reference: Reference::default()
        }
    }
}
