#![feature(phase)]

#[phase(plugin)] extern crate lazy_static;
extern crate serialize;
extern crate rustrt;
extern crate rustbox;
extern crate docopt;
extern crate iota;

#[cfg(not(test))] use std::any::{Any, AnyRefExt};
#[cfg(not(test))] use std::io::stdio;
#[cfg(not(test))] use std::rt::backtrace;
#[cfg(not(test))] use std::sync::Mutex;
#[cfg(not(test))] use rustrt::unwind;
#[cfg(not(test))] use docopt::Docopt;
#[cfg(not(test))] use iota::{Editor, Input};
#[cfg(not(test))] use rustbox::RustBox;
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
lazy_static! {
    // Place to hold backtraces until RustBox terminates.
    static ref BACKTRACE: Mutex<Vec<u8>> = Mutex::new(Vec::new());
}

#[cfg(not(test))]
fn main() {
    struct RustBoxGuard;

    impl Drop for RustBoxGuard {
        fn drop(&mut self) {
            let mut guard = BACKTRACE.lock();
            if !guard.is_empty() {
                drop(stdio::stderr().write(&**guard));
                guard.truncate(0);
            }
        }
    }

    fn rustbox_panic(msg: &(Any + Send), file: &'static str, line: uint) {
        if !rustbox::running() { return }
        let msg = match msg.downcast_ref::<String>() {
            Some(m) => m.clone(),
            None => match msg.downcast_ref::<&str>() {
                Some(m) => m.to_string(),
                None => "".to_string()
            }
        };
        let mut guard = BACKTRACE.lock();
        drop(writeln!(guard, "panic at {}, line {}: {}", file, line, msg));
        if backtrace::log_enabled() {
            drop(backtrace::write(&mut *guard));
        }
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

    let _guard = RustBoxGuard; // This lets us capture errors on panic!
    {
        let rb = RustBox::init().unwrap();
        let mut editor = Editor::new(source, &rb);
        editor.start();
    }
}
