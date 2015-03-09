#![feature(libc)]
#![feature(io)]
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

fn main() {
    let args: Args = Docopt::new(USAGE)
                            .and_then(|d| d.decode())
                            .unwrap_or_else(|e| e.exit());

    // editor source - either a filename or stdin
    // FIXME: find a way to do this without unsafe
    //        std::io doesn't allow for this, currently
    let source = if unsafe { libc::isatty(libc::STDIN_FILENO) == 1 } {
        Input::Filename(args.arg_filename)
    } else {
        Input::Stdin(stdin())
    };

    // initialise rustbox
    let rb = match RustBox::init(InitOptions{
        // FIXME: find a way to do this without unsafe
        //        std::io doesn't allow for this, currently
        buffer_stderr: unsafe { libc::isatty(libc::STDERR_FILENO) == 1 },
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
