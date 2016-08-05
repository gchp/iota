use std::str::Chars;
use std::iter::Peekable;

pub mod lexer;
mod rust;
mod python;


pub mod langs {
    pub use super::rust::RustSyntax;
    pub use super::python::PythonSyntax;
}

fn next_is(iter: &mut Peekable<Chars>, ch: char) -> bool {
    if let Some(&c) = iter.peek() {
        if c == ch { true } else { false }
    } else { false }
}

