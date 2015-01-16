#![allow(unstable)]

extern crate "rustc-serialize" as rustc_serialize;
extern crate rustbox;
extern crate docopt;
extern crate iota;

#[cfg(not(test))] use std::io::stdio;
#[cfg(not(test))] use docopt::Docopt;
#[cfg(not(test))] use iota::{
    Editor, Input,
    StandardMode, NormalMode,
    RustboxFrontend, Mode
};
#[cfg(not(test))] use rustbox::{InitOption, RustBox, InputMode};
#[cfg(not(test))] static USAGE: &'static str = "
Usage: iota [<filename>] [options]
       iota --help

Options:
    --vi           Start Iota with vi-like modes
    -h, --help     Show this message.
";


#[derive(RustcDecodable, Show)]
struct Args {
    arg_filename: Option<String>,
    flag_vi: bool,
    flag_help: bool,
}

#[cfg(not(test))]
fn main() {
    let args: Args = Docopt::new(USAGE)
                            .and_then(|d| d.decode())
                            .unwrap_or_else(|e| e.exit());

    // editor source - either a filename or stdin
    let source = if stdio::stdin_raw().isatty() {
        Input::Filename(args.arg_filename)
    } else {
        Input::Stdin(stdio::stdin())
    };

    // RustBox options
    let options = [
        if stdio::stderr_raw().isatty() { Some(InitOption::BufferStderr) } else { None },
    ];

    // initialise the frontend
    let rb = RustBox::init(&options).unwrap();
    // TODO: perhaps move this to options above - will require rustbox change
    rb.set_input_mode(InputMode::Alt);

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
