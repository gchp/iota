extern crate rustbox;
extern crate rdit;

use std::os;
use rdit::Editor;

fn main() {
    rustbox::init();

    let filename = match os::args().pop() {
        Some(f) => f,
        None => String::new(),
    };
    let mut editor = Editor::new(filename);
    editor.start();

    rustbox::shutdown();
}
