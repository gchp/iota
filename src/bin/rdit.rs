extern crate rustbox;
extern crate rdit;

fn main() {
    rustbox::init();

    rustbox::print(1, 1, rustbox::Bold, rustbox::White, rustbox::Black, "Hello, world!".to_string());
    rustbox::present();

    let editor = rdit::Editor::new();
    let sender = editor.sender.clone();

    spawn(proc() {
        loop {
            sender.send(rustbox::poll_event());
        }
    });

    editor.start();

    rustbox::shutdown();
}
