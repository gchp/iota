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
    pub prev: Option<Line>,
    pub next: Option<Line>,
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
    
    pub fn new_from_file(filename: &String) -> Buf {
        let path = Path::new(filename.to_string());
        let file = match File::open(&path) {
            Ok(f) => f,
            Err(_) => fail!("Not implemented!"),
        };

        let mut br = BufferedReader::new(file);
        
        loop {
            match br.read_line() {
                Ok(l) => {
                    // create a Line instance
                    let line = Line { data: l };
                },
                Err(_) => {
                    break;
                },
            }
        }
       
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
    pub fn new(filenames: Vec<String>) -> Editor {
        let mut buffers = Vec::new();

        if filenames.len() > 1 {
            for filename in filenames.iter() {
                buffers.push(Buf::new_from_file(filename));
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

