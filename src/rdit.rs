#![feature(macro_rules)]

extern crate rustbox;

use std::collections::dlist::DList;
use std::comm::{Receiver, Sender};
use std::io::{File, BufferedReader};
use std::str::from_utf8;

pub struct Editor {
    pub sender: Sender<rustbox::Event>,
    pub events: Receiver<rustbox::Event>,
    pub buffers: Vec<Buf>
}

pub struct Buf {
    pub lines: DList<Line>,
    pub active: bool,
    pub num_lines: int,
}

#[deriving(Clone)]
pub struct Line {
    pub data: String,
}

pub enum Response {
    Continue,
    Quit,
}

impl Buf {
    pub fn new() -> Buf {
        Buf {
            lines: DList::new(),
            active: false,
            num_lines: 0,
        }
    }

    pub fn new_from_file(filename: String) -> Buf {
        let path = Path::new(filename.to_string());

        let mut new_buffer = Buf::new();
        let mut file = BufferedReader::new(File::open(&path));
        let lines: Vec<String> = file.lines().map(|x| x.unwrap()).collect();

        for line in lines.iter() {
            new_buffer.lines.push(Line{data: line.clone()})
        }

        new_buffer
    }
}

impl Line {
    pub fn new() -> Line {
        Line {
            data: String::new(),
        }
    }
}

impl Editor {
    pub fn new(filenames: &[String]) -> Editor {
        let mut buffers = Vec::new();

        for filename in filenames.iter() {
            let mut b = Buf::new_from_file(filename.clone());
            b.active = true;
            buffers.push(b);
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
        for buffer in self.buffers.iter() {
            if buffer.active {
                for (index, line) in buffer.lines.iter().enumerate() {
                    rustbox::print(1, index, rustbox::Bold, rustbox::White, rustbox::Black, line.data.clone());
                }
            }
        }
        rustbox::set_cursor(0, 1);
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

