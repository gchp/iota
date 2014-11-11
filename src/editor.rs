extern crate rustbox;

use std::char;
use std::comm::{Receiver, Sender};

use rdit::Response;
use buffer::Buffer;
use cursor::Direction;


pub struct Editor {
    pub sender: Sender<rustbox::Event>,
    events: Receiver<rustbox::Event>,
    active_buffer: Buffer,
}

impl Editor {
    pub fn new(filename: String) -> Editor {
        let path = Path::new(filename);
        let buffer = Buffer::new_from_file(&path);

        let (send, recv) = channel();
        Editor {
            sender: send,
            events: recv,
            active_buffer: buffer,
        }
    }

    pub fn handle_key_event(&mut self, ch: u32) -> Response {
        match char::from_u32(ch) {
            Some('q') => Response::Quit,
            Some('d') => {
                let line = self.active_buffer.cursor.line.clone();
                print!("Cursor X {}\n", self.active_buffer.cursor.x);
                print!("Line len {}\n", line.unwrap().len());
                Response::Continue
            },

            // cursor movement
            Some('h') => {
                self.active_buffer.cursor.adjust(Direction::Left);
                Response::Continue
            },
            Some('j') => {
                self.active_buffer.cursor.adjust(Direction::Down);
                Response::Continue
            },
            Some('k') => {
                self.active_buffer.cursor.adjust(Direction::Up);
                Response::Continue
            },
            Some('l') => {
                self.active_buffer.cursor.adjust(Direction::Right);
                Response::Continue
            },

            // default
            _ => Response::Continue,
        }
    }

    pub fn draw(&mut self) {
        self.active_buffer.draw_contents();
        self.active_buffer.draw_status();
        self.active_buffer.cursor.draw();
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

