use rustbox::{Event, RustBox};
use std::borrow::Cow;
use std::char;
use std::io::{fs, File, FileMode, FileAccess, TempDir};

use super::Response;
use input::Input;
use buffer::Direction;
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
    InsertChar(char)
}

enum EventStatus {
    Handled(Response),
    NotHandled,
}


pub struct Editor<'e> {
    keymap: KeyMap,
    view: View<'e>,
    rb: &'e RustBox,
}

impl<'e> Editor<'e> {
    pub fn new(source: Input, rb: &'e RustBox) -> Editor<'e> {
        let view = View::new(source, rb);
        let keymap = KeyMap::load_defaults();

        Editor {
            view: view,
            rb: rb,
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
        let path = match self.view.buffer.file_path {
            Some(ref p) => Cow::Borrowed(p),
            None => {
                // TODO: prompt user for file name here
                Cow::Owned(Path::new("untitled"))
            },
        };

        let tmpdir = match TempDir::new_in(&Path::new("."), "iota") {
            Ok(d) => d,
            Err(e) => panic!("file error: {}", e)
        };

        let tmppath = tmpdir.path().join(Path::new("tmpfile"));

        let mut file = match File::open_mode(&tmppath, FileMode::Open, FileAccess::Write) {
            Ok(f) => f,
            Err(e) => panic!("file error: {}", e)
        };

        //TODO (lee): Is iteration still necessary in this format?
        for line in self.view.buffer.lines() {
            let result = file.write(line);

            if result.is_err() {
                // TODO(greg): figure out what to do here.
                panic!("Something went wrong while writing the file");
            }
        }

        if let Err(e) = fs::rename(&tmppath, &*path) {
            panic!("file error: {}", e);
        }
    }

    pub fn draw(&mut self) {
        self.view.draw(self.rb);
        self.view.draw_status(self.rb);
        self.view.draw_cursor(self.rb);
    }

    pub fn start(&mut self) {
        loop {
            self.view.clear(self.rb);
            self.draw();
            self.rb.present();
            let event = self.rb.poll_event().unwrap();
            if let Event::KeyEvent(_, key, ch) = event {
                if let Response::Quit = self.handle_key_event(key, ch) {
                    break;
                }
            }

        }
    }

    fn handle_command(&mut self, c: Command) -> Response {
        match c {
            // Editor Commands
            Command::ExitEditor      => return Response::Quit,
            Command::SaveBuffer      => self.save_active_buffer(),
            Command::ResizeView      => self.view.resize(self.rb),

            // Navigation
            Command::MoveCursor(dir) => self.view.move_cursor(dir),
            Command::LineEnd         => self.view.move_cursor_to_line_end(),
            Command::LineStart       => self.view.move_cursor_to_line_start(),

            // Editing
            Command::Delete(dir)     => self.view.delete_char(dir),
            Command::InsertTab       => self.view.insert_tab(),
            Command::InsertChar(c)   => self.view.insert_char(c)
        }
        Response::Continue
    }

    fn handle_system_event(&mut self, k: Option<Key>) -> EventStatus {
        let key = match k {
            Some(k) => k,
            None => return EventStatus::NotHandled
        };

        // send key to the keymap
        match self.keymap.check_key(key) {
            KeyMapState::Match(command) => {
                return EventStatus::Handled(self.handle_command(command));
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
