#![feature(macro_rules)]

extern crate rustbox;
use std::comm::{Receiver, Sender};
use std::io::{File, BufferedReader};
use std::str::from_utf8;

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
    pub first_line: Option<Box<Line>>,
    pub last_line: Option<Box<Line>>,
    pub active: bool,
    pub num_lines: int,
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
            first_line: Some(box Line::new()),
            last_line: Some(box Line::new()),
            active: false,
            num_lines: 0,
        }
    }

    pub fn new_from_file(filename: &String) -> Buf {
        let path = Path::new(filename.to_string());
        let file = match File::open(&path) {
            Ok(f) => f,
            Err(_) => fail!("Not implemented!"),
        };

        let mut new_buffer = Buf::new();
        let mut br = BufferedReader::new(file);
        let mut blank_line = Some(box Line::new());

        new_buffer.first_line = blank_line.clone();
        loop {
            match br.read_line() {
                Ok(ln) => {
                    match blank_line {
                        Some(ref mut line) => {
                            line.data = ln.into_bytes();
                            line.next = Some(box Line::new());

                            // rustbox::print(1, 10, rustbox::Bold, rustbox::White, rustbox::Black, line.data.to_string());
                        },
                        None => {}
                    }
                },
                Err(_) => { break; },
            }
            new_buffer.num_lines += 1;
            blank_line = match blank_line {
                Some(line) => line.next,
                None => None
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
            let ref mut line = b.first_line;
            let i = 1;
            loop {
                match line {
                    Some(ref mut l) => {
                        let line_data = from_utf8(l.data.as_slice());
                        match line_data {
                            Some(text) => {
                                rustbox::print(1, i, rustbox::Bold, rustbox::White, rustbox::Black, line_data.to_string());
                            },
                            None => {}
                        }
                        let line = l.next;
                        i += 1;
                    },
                    None => { break; }
                }
            }
            // match b.first_line {
            //     Some(ref l) => {
            //         let line_data = from_utf8(l.data.as_slice());
            //         match line_data {
            //             Some(text) => {
            //                 rustbox::print(1, 8, rustbox::Bold, rustbox::White, rustbox::Black, b.num_lines.to_string());
            //                 rustbox::print(1, 9, rustbox::Bold, rustbox::White, rustbox::Black, line_data.to_string());
            //             },
            //             None => {}
            //         }
            //     },
            //     None => {}
            // }
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

