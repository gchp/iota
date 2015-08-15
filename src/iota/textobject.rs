use std::default::Default;

#[derive(Copy, Clone, Debug)]
pub enum Kind {
    Char,
    Line(Anchor),

    Word(Anchor),
    // Sentence(Anchor),
    // Paragraph(Anchor),

    // Expression(Anchor),
    // Statement(Anchor),
    // Block(Anchor),
}

impl Kind {
    pub fn with_anchor(&self, anchor: Anchor) -> Kind {
        match *self {
            Kind::Char => Kind::Char,
            Kind::Line(_) => Kind::Line(anchor),
            Kind::Word(_) => Kind::Word(anchor),
        }
    }
    pub fn get_anchor(&self) -> Anchor {
        match *self {
            Kind::Char => Default::default(),
            Kind::Line(a) | Kind::Word(a) => a,
        }
    }
}

impl Default for Kind {
    fn default() -> Kind {
        Kind::Char
    }
}

#[derive(Copy, Clone, Debug)]
pub enum Anchor {
    Before,     // Index just prior to TextObject
    Start,      // First index within TextObject
    // Middle,  // Middle index of TextObject
    End,        // Last index within TextObject
    After,      // First index after TextObject
    Same,       // Same as index within current TextObject of the same Kind
}

impl Default for Anchor {
    fn default() -> Anchor {
        Anchor::Same
    }
}

#[derive(Copy, Clone, Debug)]
pub enum Offset {
    Backward(usize),
    Forward(usize),
}

impl Offset {
    pub fn with_num(&self, n: usize) -> Offset{
        match *self {
            Offset::Backward(_) => Offset::Backward(n),
            Offset::Forward(_)  => Offset::Forward(n),
        }
    }
}

impl Default for Offset {
    fn default() -> Offset {
        Offset::Forward(0)
    }
}

#[derive(Copy, Clone, Debug, Default)]
pub struct TextObject {
    pub kind: Kind,
    pub offset: Offset
}