use std::str;
use textobject::{Offset, Kind, Anchor};
use command::Command;

// Current chainable commands
pub enum ChainableCmds {
    Quit,
    Write,
}
// Holds our chained command structure
pub struct ChainedCmdBuilder {
    quit: bool,
    save: bool,
    line_j: Option<usize>,
}
// Parses our input
pub struct ChainedCmdParser<'a> {
    buffer: &'a [u8],
    len: usize,
    pos: usize,
    is_arg: bool,
}

// A safe version of memcmp. May not be necessary, but there
// was some performance issues with built in solution in the
// past
fn memeq<'a, T: PartialEq>(a: &'a [T], b: &'a [T]) -> bool {
    if a.len() != b.len() {
        return false;
    }

    for i in 0..a.len() {
        if a[i] != b[i] {
            return false;
        }
    }

    true
}

impl ChainedCmdBuilder {
    pub fn new() -> ChainedCmdBuilder {
        ChainedCmdBuilder {
            quit: false,
            save: false,
            line_j: None,
        }
    }
    // Parses a chainable command
    pub fn parse(&mut self, cmd: &str) -> Vec<Command> {
        let mut parser = ChainedCmdParser::new(cmd);

        while parser.pos <= parser.len {
            match parser.buffer[parser.pos] {
                b'q' => {
                    // our command is quit
                    self.quit = true;
                    if parser.pos < parser.len {
                        parser.cmd(ChainableCmds::Quit);
                    }
                }
                b'w' => {
                    // Our command is write
                    self.save = true;
                    if parser.pos < parser.len {
                        parser.cmd(ChainableCmds::Write);
                    }
                }
                b'1' | b'2' | b'3' | b'4' | b'5' | b'6' | b'7' | b'8' | b'9' | b'0' => {
                    let line_j: usize = parser.arg().parse().ok().expect("Need usize");
                    self.line_j = Some(line_j); // This should always be set
                }
                b' ' => {
                    // Arg is coming, this will be useful in the future
                    parser.is_arg = true;
                }
                _ => {
                    continue;
                }
            }

            parser.is_arg = false;
            parser.pos += 1;
        }

        self.build_commands()
    }

    // Builds our final vec of commands
    fn build_commands(&mut self) -> Vec<Command> {
        let mut res = vec![];

        if self.line_j.is_some() {
            res.push(Command::movement(Offset::Absolute(self.line_j.unwrap()),
                                       Kind::Line(Anchor::Start)));
        }
        if self.save {
            res.push(Command::save_buffer());
        }
        if self.quit {
            res.push(Command::exit_editor());
        }
        if res.is_empty() {
            res.push(Command::noop());
        }

        return res;
    }
}

impl<'a> ChainedCmdParser<'a> {
    pub fn new(cmd: &'a str) -> ChainedCmdParser<'a> {
        ChainedCmdParser {
            buffer: cmd.as_bytes(),
            len: cmd.len() - 1,
            pos: 0,
            is_arg: false,
        }
    }

    // This is a little hokey, but basically checks if
    // our command is long and moves our position
    fn cmd(&mut self, cmd: ChainableCmds) {
        match cmd {
            ChainableCmds::Quit => {
                let l_cmd = "quit".as_bytes();
                match self.len >= l_cmd.len() {
                    true => {
                        match memeq(&self.buffer[self.pos..l_cmd.len()], &l_cmd) {
                            true => self.pos += l_cmd.len() - 1,
                            false => return,
                        }
                    }
                    false => return,
                }
            }
            ChainableCmds::Write => {
                let l_cmd = "write".as_bytes();
                match self.len >= l_cmd.len() {
                    true => {
                        match memeq(&self.buffer[self.pos..l_cmd.len()], &l_cmd) {
                            true => self.pos += l_cmd.len() - 1,
                            false => return,
                        }
                    }
                    false => return,
                }
            }
        }
    }

    // Parses an arg from a chainable command
    fn arg(&mut self) -> &str {
        let start = self.pos;
        while self.buffer[self.pos] != b' ' {
            if self.pos + 1 > self.len {
                break;
            }
            self.pos += 1;
        }
        str::from_utf8(&self.buffer[start..self.pos + 1]).unwrap_or("")
    }
}
