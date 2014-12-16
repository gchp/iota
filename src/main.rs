extern crate serialize;
extern crate rustrt;
extern crate rustbox;
extern crate docopt;
extern crate iota;

#[cfg(not(test))] use std::any::{Any, AnyRefExt};
#[cfg(not(test))] use std::io::stdio;
#[cfg(not(test))] use std::task;
#[cfg(not(test))] use rustrt::unwind;
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
        fn drop(&mut self) {
            if !task::failing() {
                rustbox::shutdown();
            }
        }
    }

    fn rustbox_panic(msg: &(Any + Send), file: &'static str, line: uint) {
        rustbox::shutdown();
        let msg = match msg.downcast_ref::<String>() {
            Some(m) => m.clone(),
            None => match msg.downcast_ref::<&str>() {
                Some(m) => m.to_string(),
                None => "".to_string()
            }
        };
        let _ = writeln!(&mut stdio::stderr(), "panic at {}, line {}: {}", file, line, msg);
    }

    unsafe { unwind::register(rustbox_panic); }

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
