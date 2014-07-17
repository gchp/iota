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
    pub fn handle_events(&self) -> Response {
        let evt = self.events.recv();
        match evt {
            rustbox::KeyEvent(_, _, ch) => {
                match std::char::from_u32(ch) {
                    Some('q') => Quit,
                    _ => Continue,
                }
            },
            _ => Continue,
        }
    }

    pub fn start(&self) -> Response {
        let resp = self.handle_events();
        resp
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
