extern crate rustbox;

use std::comm::{Receiver, Sender};
use std::char;
use std::io::{fs, File, FileMode, FileAccess, TempDir};
use std::sync::Arc;
use std::sync::atomic::{Ordering, AtomicBool};

use super::Response;
use input::Input;
use cursor::Direction;
use keyboard::Key;
use keymap::{ KeyMap, KeyMapState };
use view::View;


#[deriving(Copy, Show)]
#[allow(dead_code)]
pub enum Command {
    SaveBuffer,
    ExitEditor,
    ResizeView,

    MoveCursor(Direction),
    LineEnd,
    LineStart,

    Delete(Direction),
    InsertTab,
    InsertLine,
    InsertChar(char)
}

enum EventStatus {
    Handled(Response),
    NotHandled,
}


pub struct Editor<'e> {
    pub running: Arc<AtomicBool>,
    pub sender: Sender<rustbox::Event>,

    keymap: KeyMap,
    events: Receiver<rustbox::Event>,
    view: View<'e>,
}

impl<'e> Editor<'e> {
    pub fn new(source: Input) -> Editor<'e> {
        let view = View::new(source);
        let (send, recv) = channel();
        let keymap = KeyMap::load_defaults();

        Editor {
            sender: send,
            events: recv,
            view: view,
            running: Arc::new(AtomicBool::new(false)),
            keymap: keymap,
        }
    }

    pub fn handle_key_event(&mut self, key: u16, ch: u32) -> Response {
        let key = match key {
            0 => char::from_u32(ch).map(|c| Key::Char(c)),
            a => Key::from_special_code(a),
        };

        match self.handle_system_event(key) {
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
            data.push('\n');
            let result = file.write(data.as_bytes());

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
        // Synchronizes with transfer across thread boundary
        self.running.store(true, Ordering::Relaxed);
        self.event_loop();
        self.main_loop();
    }

    fn main_loop(&mut self) {
        while self.running.load(Ordering::Relaxed) {
            self.view.clear();
            self.draw();
            rustbox::present();
            if let rustbox::Event::KeyEvent(_, key, ch) = self.events.recv() {
                if let Response::Quit = self.handle_key_event(key, ch) {
                    // Okay if it doesn't quit immediately.
                    self.running.store(false, Ordering::Relaxed);
                }
            }
        }
    }

    fn event_loop(&self) {
        // clone the sender so that we can use it in the proc
        let sender = self.sender.clone();
        let running = self.running.clone();

        spawn(move || {
            while running.load(Ordering::Relaxed) {
                if sender.send_opt(rustbox::peek_event(1000)).is_err() {
                    running.store(false, Ordering::Relaxed);
                }
            }
        });
    }

    fn handle_command(&mut self, c: Command) {
        match c {
            // Editor Commands
            Command::ExitEditor      => self.running.store(false, Ordering::Relaxed),
            Command::SaveBuffer      => self.save_active_buffer(),
            Command::ResizeView      => self.view.resize(),

            // Navigation
            Command::MoveCursor(dir) => self.view.move_cursor(dir),
            Command::LineEnd         => self.view.move_cursor_to_line_end(),
            Command::LineStart       => self.view.move_cursor_to_line_start(),

            // Editing
            Command::Delete(dir)     => self.view.delete_char(dir),
            Command::InsertTab       => self.view.insert_tab(),
            Command::InsertLine      => self.view.insert_line(),
            Command::InsertChar(c)   => self.view.insert_char(c)
        }
    }

    fn handle_system_event(&mut self, k: Option<Key>) -> EventStatus {
        let key = match k {
            Some(k) => k,
            None => return EventStatus::NotHandled
        };

        // send key to the keymap
        match self.keymap.check_key(key) {
            KeyMapState::Match(command) => {
                self.handle_command(command);
                return EventStatus::Handled(Response::Continue)
            },
            KeyMapState::Continue => {
                // keep going and wait for more keypresses
                return EventStatus::Handled(Response::Continue) 
            },
            KeyMapState::None => {}  // do nothing and handle the key normally
        }

        // if the key is a character that is not part of a keybinding, insert into the buffer
        // otherwise, ignore it.
        if let Key::Char(c) = key {
            self.view.insert_char(c);
            EventStatus::Handled(Response::Continue)
        } else {
            EventStatus::NotHandled
        }
    }
}
