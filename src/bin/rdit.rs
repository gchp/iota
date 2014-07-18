extern crate rustbox;
extern crate rdit;

fn main() {
    rustbox::init();

    rustbox::print(1, 1, rustbox::Bold, rustbox::White, rustbox::Black, "Hello, world!");
    rustbox::present();

    let(events, receiver) = channel();
    let editor = rdit::Editor {events: receiver};

    spawn(proc() {
        loop {
            events.send(rustbox::poll_event());
            //break;
        }
    });

    editor.start();

    rustbox::shutdown();
}
