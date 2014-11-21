extern crate rustbox;

use std::char;
use std::comm::{Receiver, Sender};
use std::num;
use std::io::{File, FileMode, FileAccess};

use rdit::Response;
use buffer::Buffer;
use cursor::Direction;
use keyboard::Key;


pub struct Editor<'e> {
    pub sender: Sender<rustbox::Event>,
    events: Receiver<rustbox::Event>,
    active_buffer: Buffer<'e>,
}

impl<'e> Editor<'e> {
    pub fn new(filename: String) -> Editor<'e> {
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
        let input_key: Option<Key> = num::from_u16(key);

        match input_key {
            Some(Key::Enter) => {
                self.active_buffer.insert_line();
                return Response::Continue
            }
            Some(Key::Up) => {
                self.active_buffer.adjust_cursor(Direction::Up);
                return Response::Continue
            }
            Some(Key::Down) => {
                self.active_buffer.adjust_cursor(Direction::Down);
                return Response::Continue
            }
            Some(Key::Left) => {
                self.active_buffer.adjust_cursor(Direction::Left);
                return Response::Continue
            }
            Some(Key::Right) => {
                self.active_buffer.adjust_cursor(Direction::Right);
                return Response::Continue
            }
            Some(Key::Space) => {
                self.active_buffer.insert_char(' ');
                return Response::Continue
            }
            Some(Key::Backspace) => {
                self.active_buffer.delete_char();
                return Response::Continue
            }
            Some(Key::CtrlS) => {
                self.save_active_buffer();
                return Response::Continue
            }
            Some(Key::CtrlQ) => {
                return Response::Quit
            }
            _ => {}
        }

        print!("k: {} ", key);
        print!("c: {} **", ch);

        match char::from_u32(ch) {
            Some(c) => {
                self.active_buffer.insert_char(c);
                return Response::Continue
            }
            _ => {}
        }

        Response::Continue
    }

    pub fn save_active_buffer(&mut self) {
        let lines = &self.active_buffer.lines;
        let path = Path::new(&self.active_buffer.file_path);

        let mut file = match File::open_mode(&path, FileMode::Open, FileAccess::Write) {
            Ok(f) => f,
            Err(e) => panic!("file error: {}", e),
        };

        for line in lines.iter() {
            let data = format!("{}\n", line.borrow().data);
            let result = file.write(data.as_bytes());

            if result.is_err() {
                // TODO: figure out what to do here.
            }
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

