extern crate rustbox;

use std::char;
use std::comm::{Receiver, Sender};

use rdit::Response;
use rdit::Buffer;


pub struct Editor {
    pub sender: Sender<rustbox::Event>,
    pub events: Receiver<rustbox::Event>,
    pub buffers: Vec<Buffer>,
    pub cursor_x: int,
    pub cursor_y: int,
}

impl Editor {
    pub fn new(filenames: &[String]) -> Editor {
        let mut buffers = Vec::new();

        for filename in filenames.iter() {
            let mut b = Buffer::new_from_file(filename.clone());
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
        match char::from_u32(ch) {
            Some('q') => Response::Quit,

            // cursor movement
            Some('h') => { self.cursor_x -= 1; Response::Continue },
            Some('j') => { self.cursor_y += 1; Response::Continue },
            Some('k') => { self.cursor_y -= 1; Response::Continue },
            Some('l') => { self.cursor_x += 1; Response::Continue },

            // default
            _ => Response::Continue,
        }
    }

    pub fn draw(&mut self) {
        // TODO: change this to only draw the active buffer
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
                        Response::Quit => break,
                        _ => {}
                    }
                },
                _ => {}
            }
        }
        return false
    }

}

