extern crate rustbox;
extern crate rdit;

#[cfg(not(test))] use std::os;
#[cfg(not(test))] use rdit::Editor;

#[cfg(not(test))]
fn main() {
    rustbox::init();

    let args = os::args();
    let mut editor: Editor;

    if args.len() == 1 {
        editor = Editor::new(None);
    } else {
        let filename = os::args().pop();
        editor = Editor::new(filename);
    }
    editor.start();

    rustbox::shutdown();
}
