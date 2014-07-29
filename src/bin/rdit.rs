extern crate rustbox;
extern crate rdit;

use std::os;

fn main() {
    rustbox::init();

    rustbox::print(1, 1, rustbox::Bold, rustbox::White, rustbox::Black, "Hello, world!".to_string());
    rustbox::present();

    // TODO: perhaps only pass in a slice?
    let editor = rdit::Editor::new(os::args());
    
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
