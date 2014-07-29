extern crate rustbox;
use std::comm::{Receiver, Sender};
use std::io::{File, BufferedReader};


pub struct Editor {
    pub sender: Sender<rustbox::Event>,
    pub events: Receiver<rustbox::Event>,
    pub buffers: Vec<Buf>
}

pub struct Buf {
    pub first_line: Line,
    pub last_line: Line,
}

pub struct Line {
    pub data: Vec<u8>,
    pub prev: Option<Box<Line>>,
    pub next: Option<Box<Line>>,
}

pub enum Response {
    Continue,
    Quit,
}

impl Buf {
    pub fn new() -> Buf {
        Buf {
            first_line: Line::new(),
            last_line: Line::new(),
        }
    }
}

impl Line {
    pub fn new() -> Line {
        Line {
            data: Vec::new(),
            prev: None,
            next: None,
        }
    }
}

impl Editor {
    pub fn new() -> Editor {
        let (send, recv) = channel();
        Editor {
            sender: send,
            events: recv,
            buffers: Vec::new(),
        }
    }

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

