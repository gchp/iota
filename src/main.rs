#![feature(libc)]
#![cfg(not(test))]

extern crate libc;
extern crate "rustc-serialize" as rustc_serialize;
extern crate rustbox;
extern crate docopt;
extern crate iota;

use std::io::stdin;
use docopt::Docopt;
use iota::{
    Editor, Input,
    StandardMode, NormalMode,
    RustboxFrontend, Mode
};
use rustbox::{InitOptions, RustBox, InputMode};
static USAGE: &'static str = "
Usage: iota [<filename>] [options]
       iota --help

Options:
    --vi           Start Iota with vi-like modes
    -h, --help     Show this message.
";


#[derive(RustcDecodable, Debug)]
struct Args {
    arg_filename: Option<String>,
    flag_vi: bool,
    flag_help: bool,
}

fn is_atty(fileno: libc::c_int) -> bool {
    // FIXME: find a way to do this without unsafe
    //        std::io doesn't allow for this, currently
    unsafe { libc::isatty(fileno) != 0 }
}

fn main() {
    let args: Args = Docopt::new(USAGE)
                            .and_then(|d| d.decode())
                            .unwrap_or_else(|e| e.exit());

    let stdin_is_atty = is_atty(libc::STDIN_FILENO);
    let stderr_is_atty = is_atty(libc::STDERR_FILENO);

    // editor source - either a filename or stdin
    let source = if stdin_is_atty {
        Input::Filename(args.arg_filename)
    } else {
        Input::Stdin(stdin())
    };


    // initialise rustbox
    let rb = match RustBox::init(InitOptions{
        buffer_stderr: stderr_is_atty,
        input_mode: InputMode::Esc,
    }) {
        Result::Ok(v) => v,
        Result::Err(e) => panic!("{}", e),
    };

    // initialise the frontend
    let frontend = RustboxFrontend::new(&rb);

    // initialise the editor mode
    let mode: Box<Mode> = if args.flag_vi {
        Box::new(NormalMode::new())
    } else {
         Box::new(StandardMode::new())
    };

    // start the editor
    let mut editor = Editor::new(source, mode, frontend);
    editor.start();
}
