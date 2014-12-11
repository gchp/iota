# Iota [![Build Status](https://travis-ci.org/gchp/iota.svg?branch=master)](https://travis-ci.org/gchp/iota)

Iota is a terminal-based text-editor written in Rust.

## Screenshot

Here's what it looks like right now, editing itself.

![Screenshot](https://raw.githubusercontent.com/gchp/iota/master/screenshot.png)

## Usage

Clone the project and run `cargo build`.

**NOTE**: You'll need to install the [termbox](https://github.com/nsf/termbox)
library before running the editor. See installation instructions for [rustbox](https://github.com/gchp/rustbox)
which is a wrapper around the termbox library.

Then to start the editor run `./target/iota /path/to/file.txt`. Or simply `./target/iota`
to open an empty buffer.

You can move the cursor around with the arrow keys.

To save, press `Ctrl-s`.
To exit, press `Ctrl-q`, followed by `Ctrl-c`.
