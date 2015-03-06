use std::io;

/// A source of Input for the Editor.
///
/// This is used at startup, where the user can either open a file, or
/// start Iota with data from stdin.
pub enum Input {
    /// A Filename
    Filename(Option<String>),

    /// The stdin reader
    Stdin(io::Stdin),
}
