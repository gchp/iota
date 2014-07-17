extern crate rustbox;
use std::comm::{Receiver};

struct Editor {
    events: Receiver<rustbox::Event>,
}

enum Response {
    Continue,
    Quit,
}

impl Editor {
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

fn main() {
    rustbox::init();

    let(events, receiver) = channel();
    let editor = Editor {events: receiver};

    spawn(proc() {
        loop {
            events.send(rustbox::poll_event());
            //break;
        }
    });

    editor.start();

    rustbox::shutdown();
}
