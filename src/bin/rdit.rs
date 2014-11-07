extern crate rustbox;
extern crate rdit;

use std::os;

fn main() {
    rustbox::init();

    let filename = match os::args().pop() {
        Some(f) => f,
        None => String::new(),
    };
    let mut editor = rdit::Editor::new(filename);
    
    // clone the sender so that we can use it in the proc
    let sender = editor.sender.clone();

    spawn(proc() {
        loop {
            sender.send(rustbox::poll_event());
        }
    });

    editor.start();

    rustbox::shutdown();
}
