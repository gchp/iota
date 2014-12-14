extern crate rustbox;

use std::comm::{Receiver, Sender};
use std::num;
use std::io::{fs, File, FileMode, FileAccess, TempDir};
use std::sync::{Arc, RWLock};

use super::Response;
use input::Input;
use cursor::Direction;
use keyboard::Key;
use view::View;


enum EventStatus {
    Handled(Response),
    NotHandled,
}


pub struct Editor<'e> {
    pub running: Arc<RWLock<bool>>,
    pub sender: Sender<rustbox::Event>,

    events: Receiver<rustbox::Event>,
    view: View<'e>,
}

impl<'e> Editor<'e> {
    pub fn new(source: Input) -> Editor<'e> {
        let view = View::new(source);

        let (send, recv) = channel();
        Editor {
            sender: send,
            events: recv,
            view: view,
            running: Arc::new(RWLock::new(false)),
        }
    }

    pub fn handle_key_event(&mut self, key: u16, ch: u32) -> Response {
        let key_code = key as u32 + ch;
        let input_key: Option<Key> = num::from_u32(key_code);

        match self.handle_system_event(input_key) {
            EventStatus::Handled(response) => { response }
            EventStatus::NotHandled        => { Response::Continue }
        }
    }

    pub fn save_active_buffer(&mut self) {
        let lines = &self.view.buffer.lines;
        let path = self.view.buffer.file_path.as_ref().unwrap();

        let tmpdir = match TempDir::new_in(&Path::new("."), "iota") {
            Ok(d) => d,
            Err(e) => panic!("file error: {}", e)
        };

        let tmppath = tmpdir.path().join(Path::new("tmpfile"));

        let mut file = match File::open_mode(&tmppath, FileMode::Open, FileAccess::Write) {
            Ok(f) => f,
            Err(e) => panic!("file error: {}", e)
        };

        for line in lines.iter() {
            let mut data = line.data.clone();
            data.push('\n' as u8);
            let result = file.write(data.as_slice());

            if result.is_err() {
                // TODO(greg): figure out what to do here.
                panic!("Something went wrong while writing the file");
            }
        }

        if let Err(e) = fs::rename(&tmppath, path) {
            panic!("file error: {}", e);
        }
    }

    pub fn draw(&mut self) {
        self.view.draw();
        self.view.draw_status();
        self.view.draw_cursor();
    }

    pub fn start(&mut self) {
        *self.running.write() = true;
        self.event_loop();
        self.main_loop();
    }

    fn main_loop(&mut self) {
        while *self.running.read() {
            self.view.clear();
            self.draw();
            rustbox::present();
            if let rustbox::Event::KeyEvent(_, key, ch) = self.events.recv() {
                if let Response::Quit = self.handle_key_event(key, ch) {
                    *self.running.write() = false;
                }
            }
        }
    }

    fn event_loop(&self) {
        // clone the sender so that we can use it in the proc
        let sender = self.sender.clone();
        let lock = self.running.clone();

        spawn(proc() {
            while *lock.read() {
                let _ = sender.send_opt(rustbox::peek_event(1000));
            }
        });
    }

    fn handle_system_event(&mut self, k: Option<Key>) -> EventStatus {
        let key = match k {
            Some(k) => k,
            None => return EventStatus::NotHandled
        };

        match key {
            Key::Up        => { self.view.move_cursor(Direction::Up); }
            Key::Down      => { self.view.move_cursor(Direction::Down); }
            Key::Left      => { self.view.move_cursor(Direction::Left); }
            Key::Right     => { self.view.move_cursor(Direction::Right); }
            Key::Enter     => { self.view.insert_line(); }

            // Tab inserts 4 spaces, rather than a \t
            Key::Tab       => { self.view.insert_tab(); }

            Key::Backspace => { self.view.delete_char(Direction::Left); }
            Key::Delete    => { self.view.delete_char(Direction::Right); }
            Key::CtrlS     => { self.save_active_buffer(); }
            Key::CtrlQ     => { return EventStatus::Handled(Response::Quit) }
            Key::CtrlR     => { self.view.resize(); }

            // TODO(greg): move these keys to event handlers of each mode
            // This block is for matching keys which will insert a char to the buffer
            Key::Exclaim   |  Key::Hash |
            Key::Dollar    | Key::Percent |
            Key::Ampersand | Key::Quote |
            Key::LeftParen | Key::RightParen |
            Key::Asterisk  | Key::Plus |
            Key::Comma     | Key::Minus |
            Key::Period    | Key::Slash |
            Key::D0 | Key::D1 | Key::D2 |
            Key::D3 | Key::D4 | Key::D5 |
            Key::D6 | Key::D7 | Key::D8 |
            Key::D9 | Key::Colon |
            Key::Semicolon | Key::Less |
            Key::Equals    | Key::Greater |
            Key::Question  | Key::At |
            Key::LeftBracket  | Key::Backslash |
            Key::RightBracket | Key::Caret |
            Key::Underscore   | Key::Backquote |

            Key::ShiftA | Key::ShiftB | Key::ShiftC |
            Key::ShiftD | Key::ShiftE | Key::ShiftF |
            Key::ShiftG | Key::ShiftH | Key::ShiftI |
            Key::ShiftJ | Key::ShiftK | Key::ShiftL |
            Key::ShiftM | Key::ShiftN | Key::ShiftO |
            Key::ShiftP | Key::ShiftQ | Key::ShiftR |
            Key::ShiftS | Key::ShiftT | Key::ShiftU |
            Key::ShiftV | Key::ShiftW | Key::ShiftX |
            Key::ShiftY | Key::ShiftZ |

            Key::A | Key::B | Key::C | Key::D |
            Key::E | Key::F | Key::G | Key::H |
            Key::I | Key::J | Key::K | Key::L |
            Key::M | Key::N | Key::O | Key::P |
            Key::Q | Key::R | Key::S | Key::T |
            Key::U | Key::V | Key::W | Key::X |
            Key::Y | Key::Z | Key::LeftBrace |
            Key::Pipe       | Key::RightBrace |
            Key::Tilde      | Key::Space => { self.view.insert_char(key.get_char().unwrap()) }

            // default
            _              => { return EventStatus::NotHandled }
        }
        // event is handled and we want to keep the editor running
        EventStatus::Handled(Response::Continue)
    }

}
