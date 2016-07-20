use std::str::Chars;
use std::iter::Peekable;
use std::iter::Enumerate;

pub mod lexer;
mod rust;


pub mod langs {
    pub use super::rust::RustSyntax;
}

fn next_is(iter: &mut Peekable<Enumerate<Chars>>, ch: char) -> bool {
    if let Some(&(_, c)) = iter.peek() {
        if c == ch { true } else { false }
    } else { false }
}

