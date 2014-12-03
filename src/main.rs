extern crate serialize;
extern crate rustbox;
extern crate docopt;
extern crate iota;

use docopt::Docopt;

#[cfg(not(test))] use iota::Editor;


static USAGE: &'static str = "
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
    let args: Args = Docopt::new(USAGE)
                            .and_then(|d| d.decode())
                            .unwrap_or_else(|e| e.exit());

    rustbox::init();
    let mut editor = Editor::new(args.arg_filename);
    editor.start();
    rustbox::shutdown();
}
