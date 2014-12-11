use std::io::stdio;

pub enum Input {
    Filename(Option<String>),
    Stdin(stdio::StdinReader),
}
