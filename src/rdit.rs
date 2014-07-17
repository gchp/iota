extern crate rustbox;

fn main() {
    rustbox::init();

    loop {
        match rustbox::poll_event() {
            rustbox::KeyEvent(_, _, ch) => {
                match std::char::from_u32(ch) {
                    Some('q') => { break; },
                    _ => {}
                }
            },
            _ => { }
        }
    }

    rustbox::shutdown();
}
