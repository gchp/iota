#![feature(macro_rules)]

extern crate rustbox;
use std::comm::{Receiver, Sender};
use std::io::{File, BufferedReader};

macro_rules! some {
    ($e:expr) => (
        match $e {
            Some(e) => e,
            None => return None
        }
    )
}

pub struct Editor {
    pub sender: Sender<rustbox::Event>,
    pub events: Receiver<rustbox::Event>,
    pub buffers: Vec<Buf>
}

pub struct Buf {
    pub first_line: Line,
    pub last_line: Line,
    pub active: bool,
}

#[deriving(Clone)]
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
            active: false,
        }
    }

    pub fn new_from_file(filename: &String) -> Buf {
        let path = Path::new(filename.to_string());
        let file = match File::open(&path) {
            Ok(f) => f,
            Err(_) => fail!("Not implemented!"),
        };

        let mut new_buffer = Buf {
            first_line: Line::new(),
            last_line: Line::new(),
            active: false,
        };

        let mut br = BufferedReader::new(file);
        let mut blank_line = Line::new();

        new_buffer.first_line = blank_line.clone();
        loop {
            match br.read_line() {
                Ok(l) => {
                    blank_line.data = l.into_bytes();

                    if new_buffer.first_line.data.len() == 0 {
                        new_buffer.first_line.data = blank_line.data
                    }

                    blank_line.next = Some(box Line::new());
                },
                Err(_) => {
                    break;
                },
            }
            blank_line = *match blank_line.next {
                Some(l) => l.clone(),
                None => box Line::new(),
            };
        }
        new_buffer.last_line = blank_line;
        new_buffer
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
    pub fn new(filenames: Vec<String>) -> Editor {
        let mut buffers = Vec::new();

        if filenames.len() > 1 {
            for filename in filenames.iter() {
                let mut b = Buf::new_from_file(filename);
                b.active = true;
                buffers.push(b);
            }
        }

        let (send, recv) = channel();
        Editor {
            sender: send,
            events: recv,
            buffers: buffers,
        }
    }

    pub fn handle_key_event(&self, ch: u32) -> Response {
        match std::char::from_u32(ch) {
            Some('q') => Quit,
            _ => Continue,
        }
    }

    pub fn draw(&mut self) {
        for mut b in self.buffers.iter() {
            if b.active {
                let ref line = b.first_line;
                let line_data = std::str::from_utf8(line.data.as_slice());
                match line_data {
                    Some(text) => {
                        rustbox::print(1, 1, rustbox::Bold, rustbox::White, rustbox::Black, text.to_string());
                    },
                    None => {}
                }
            }
       }
    }

    pub fn start(&mut self) -> bool {
        loop {
            self.draw();
            rustbox::present();
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

