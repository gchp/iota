extern crate rustbox;

use std::char;
use std::comm::{Receiver, Sender};

use rdit::Response;
use rdit::Buffer;

use utils;

pub struct Editor {
    pub sender: Sender<rustbox::Event>,
    events: Receiver<rustbox::Event>,
    active_buffer: Buffer,
    cursor_x: int,
    cursor_y: int,
}

impl Editor {
    pub fn new(filename: String) -> Editor {
        let mut buffer = Buffer::new_from_file(filename);
        buffer.active = true;

        let (send, recv) = channel();
        Editor {
            sender: send,
            events: recv,
            active_buffer: buffer,
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
        for (index, line) in self.active_buffer.lines.iter().enumerate() {
            utils::draw(index, line.data.clone());
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

