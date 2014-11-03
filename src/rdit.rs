#![feature(macro_rules)]

extern crate rustbox;

use std::collections::dlist::DList;
use std::comm::{Receiver, Sender};
use std::io::{File, BufferedReader};
use std::str::from_utf8;

pub struct Editor {
    pub sender: Sender<rustbox::Event>,
    pub events: Receiver<rustbox::Event>,
    pub buffers: Vec<Buf>,
    pub cursor_x: int,
    pub cursor_y: int,
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
            cursor_x: 0,
            cursor_y: 0,
        }
    }

    pub fn handle_key_event(&mut self, ch: u32) -> Response {
        match std::char::from_u32(ch) {
            Some('q') => Quit,
            Some('h') => { self.cursor_x -= 1; Continue },
            Some('j') => { self.cursor_y += 1; Continue },
            Some('k') => { self.cursor_y -= 1; Continue },
            Some('l') => { self.cursor_x += 1; Continue },
            _ => Continue,
        }
    }

    pub fn draw(&mut self) {
        for buffer in self.buffers.iter() {
            if buffer.active {
                for (index, line) in buffer.lines.iter().enumerate() {
                    rustbox::print(0, index, rustbox::Bold, rustbox::White, rustbox::Black, line.data.clone());
                }
            }
        }
        rustbox::set_cursor(self.cursor_x, self.cursor_y);
    }

    pub fn start(&mut self) -> bool {
        loop {
            self.draw();
            rustbox::present();
            match self.events.try_recv() {
                Ok(rustbox::KeyEvent(_, _, ch)) => {
                    match self.handle_key_event(ch) {
                        Quit => break,
                        _ => {}
                    }
                },
                TryRecvError => {}
            }
        }
        return false
    }

}

