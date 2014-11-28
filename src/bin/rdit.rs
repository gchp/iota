extern crate rustbox;
extern crate rdit;

#[cfg(not(test))] use std::os;
#[cfg(not(test))] use rdit::Editor;

#[cfg(not(test))]
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
