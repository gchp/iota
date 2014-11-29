extern crate rustbox;

use std::char;
use std::comm::{Receiver, Sender};
use std::num;
use std::io::{File, FileMode, FileAccess};

use super::Response;
use cursor::Direction;
use keyboard::Key;
use view::View;


enum EventStatus {
    Handled(Response),
    NotHandled,
}


pub struct Editor<'e> {
    pub sender: Sender<rustbox::Event>,
    events: Receiver<rustbox::Event>,
    view: View<'e>,

    pub running: bool,
}

impl<'e> Editor<'e> {
    pub fn new(filename: String) -> Editor<'e> {
        let path = Path::new(filename);
        let view = View::new(&path);

        let (send, recv) = channel();
        Editor {
            sender: send,
            events: recv,
            view: view,
            running: false,
        }
    }

    pub fn handle_key_event(&mut self, key: u16, ch: u32) -> Response {
        let input_key: Option<Key> = num::from_u16(key);

        let event_status = self.handle_system_event(input_key.unwrap());
        match event_status {
            EventStatus::Handled(r) => { return r }
            EventStatus::NotHandled => { /* keep going */ }
        }

        print!("k: {} ", key);
        print!("c: {} **", ch);

        match char::from_u32(ch) {
            Some(c) => {
                self.view.insert_char(c);
                return Response::Continue
            }
            _ => {}
        }

        Response::Continue
    }

    pub fn save_active_buffer(&mut self) {
        let lines = &self.view.buffer.lines;
        let path = Path::new(&self.view.buffer.file_path);

        let mut file = match File::open_mode(&path, FileMode::Open, FileAccess::Write) {
            Ok(f) => f,
            Err(e) => panic!("file error: {}", e),
        };

        for line in lines.iter() {
            let data = format!("{}\n", line.borrow().data);
            let result = file.write(data.as_bytes());

            if result.is_err() {
                // TODO(greg): figure out what to do here.
            }
        }
    }

    pub fn draw(&mut self) {
        self.view.draw();
        self.view.draw_status();
        self.view.draw_cursor();
    }

    pub fn start(&mut self) {
        self.running = true;
        self.event_loop();
        self.main_loop();
    }

    fn main_loop(&mut self) {
        while self.running {
            self.view.clear();
            self.draw();
            rustbox::present();
            match self.events.recv() {
                rustbox::KeyEvent(_, key, ch) => {
                    match self.handle_key_event(key, ch) {
                        // TODO(greg): refactor event handling responses
                        Response::Continue => { /* keep going*/ }
                        Response::Quit     => self.running = false,
                    }
                },
                _ => {}
            }
        }
    }

    fn event_loop(&self) {
        // clone the sender so that we can use it in the proc
        let sender = self.sender.clone();

        spawn(proc() {
            loop {
                sender.send(rustbox::poll_event());
            }
        });
    }

    fn handle_system_event(&mut self, key: Key) -> EventStatus {
        match key {
            Key::Up        => { self.view.move_cursor(Direction::Up); }
            Key::Down      => { self.view.move_cursor(Direction::Down); }
            Key::Left      => { self.view.move_cursor(Direction::Left); }
            Key::Right     => { self.view.move_cursor(Direction::Right); }
            Key::Enter     => { self.view.insert_line(); }
            Key::Space     => { self.view.insert_char(' '); }
            Key::Backspace => { self.view.delete_char(Direction::Left); }
            Key::Delete    => { self.view.delete_char(Direction::Right); }
            Key::CtrlS     => { self.save_active_buffer(); }
            Key::CtrlQ     => { return EventStatus::Handled(Response::Quit) }
            _              => { return EventStatus::NotHandled }
        }
        // event is handled and we want to keep the editor running
        EventStatus::Handled(Response::Continue)
    }

}

