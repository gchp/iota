extern crate rustbox;

use std::char;
use std::comm::{Receiver, Sender};
use std::num::from_u64;

use rdit::Response;
use buffer::Buffer;
use cursor::Direction;
use keyboard::Key;


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

    pub fn handle_key_event(&mut self, key: u16, ch: u32) -> Response {
        let input_key: Option<Key> = from_u64(key as u64);

        match input_key {
            Some(Key::Enter) => {
                self.active_buffer.insert_new_line();
                return Response::Continue
            }
            _ => {}
        }

        match char::from_u32(ch) {
            Some('q') => Response::Quit,
            Some('c') => {
                rustbox::present();
                Response::Continue
            }

            // cursor movement
            Some('h') => {
                self.active_buffer.adjust_cursor(Direction::Left);
                Response::Continue
            },
            Some('j') => {
                self.active_buffer.adjust_cursor(Direction::Down);
                Response::Continue
            },
            Some('k') => {
                self.active_buffer.adjust_cursor(Direction::Up);
                Response::Continue
            },
            Some('l') => {
                self.active_buffer.adjust_cursor(Direction::Right);
                Response::Continue
            },
            Some(c) => {
                self.active_buffer.insert_char(c);
                Response::Continue
            }

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
            rustbox::clear();
            self.draw();
            rustbox::present();
            match self.events.try_recv() {
                Ok(rustbox::KeyEvent(_, key, ch)) => {
                    match self.handle_key_event(key, ch) {
                        Response::Quit => break,
                        Response::Continue => {
                            rustbox::clear();
                            rustbox::present();
                        }
                    }
                },
                _ => {}
            }
        }
        return false
    }

}

