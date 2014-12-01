# Rdit

[![Build Status](https://travis-ci.org/gchp/rdit.svg?branch=master)](https://travis-ci.org/gchp/rdit)

Rdit is a terminal-based text-editor written in Rust.

This is really an experimental project that I'm using as I learn Rust.
Pretty much everything at this stage is subject to change.
I'm also interested in a better name for this, definitely open to suggestions. 

I've never built an editor before, so this is new territory for me. Please
don't look at the early history of the project, it's full of my silly
mistakes and awful workarounds as I try figure out how this all works.

## Screenshot

Here's what it looks like right now, editing itself.

![Screenshot](https://raw.githubusercontent.com/gchp/rdit/master/screenshot.png)

## Usage

Clone the project and run `cargo build`.

Then to start the editor run `./target/rdit /path/to/file.txt`. Or simply `./target/rdit`
to open an empty buffer.

You can move the cursor around with the arrow keys.

To save, press `Ctrl-s`.
To exit, press `Ctrl-q`, followed by `Ctrl-c`.
