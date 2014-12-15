extern crate serialize;
extern crate rustbox;
extern crate docopt;
extern crate iota;

#[cfg(not(test))] use std::io::stdio;
#[cfg(not(test))] use docopt::Docopt;
#[cfg(not(test))] use iota::{Editor, Input};
#[cfg(not(test))] static USAGE: &'static str = "
Usage: iota [<filename>]
       iota --help

Options:
    -h, --help     Show this message.
";


#[deriving(Decodable, Show)]
struct Args {
    arg_filename: Option<String>,
    flag_help: bool,
}

#[cfg(not(test))]
fn main() {
    struct RustBoxGuard;

    impl Drop for RustBoxGuard {
        fn drop(&mut self) { rustbox::shutdown() }
    }

    let args: Args = Docopt::new(USAGE)
                            .and_then(|d| d.decode())
                            .unwrap_or_else(|e| e.exit());
    let source = if stdio::stdin_raw().isatty() {
        Input::Filename(args.arg_filename)
    } else {
        Input::Stdin(stdio::stdin())
    };

    rustbox::init();
    let _guard = RustBoxGuard; // Ensure that RustBox gets shut down on abnormal termination.
    let mut editor = Editor::new(source);
    editor.start();
}
