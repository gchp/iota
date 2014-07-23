extern crate rustbox;
use std::comm::{Receiver};

pub struct Editor {
    pub events: Receiver<rustbox::Event>,
    pub buffers: Vec<Buf>
}

pub struct Buf {
    pub first_line: Vec<Line>,
    pub last_line: Vec<Line>,
}

pub struct Line {
    pub data: Vec<u8>,
}

enum Response {
    Continue,
    Quit,
}

impl Editor {
    pub fn handle_key_event(&self, ch: u32) -> Response {
        match std::char::from_u32(ch) {
            Some('q') => Quit,
            _ => Continue,
        }
    }

    pub fn start(&self) -> bool {
        loop {
            let status = match self.events.recv() {
                rustbox::KeyEvent(_, _, ch) => {
                    let status = self.handle_key_event(ch);
                    match status {
                        Quit => { false },
                        _ => { true },
                    }
                },
                _ => { true }
            };
            if status == false {
                return false;
            }
        }
    }
}

