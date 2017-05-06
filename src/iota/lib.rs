//! Iota
//!
//! A highly customisable text editor built with modern hardware in mind.
//!
//! This module contains all you need to create an `iota` executable.

#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]
#![warn(missing_docs)]

extern crate syntect;
extern crate rustbox;
extern crate gapbuffer;
extern crate tempdir;
extern crate regex;
extern crate unicode_width;
#[macro_use] extern crate lazy_static;

pub use editor::{Editor, Options};
pub use input::Input;
pub use modes::{StandardMode, NormalMode, Mode};

mod input;
mod utils;
mod buffer;
mod editor;
mod keyboard;
mod keymap;
mod view;
mod log;
mod modes;
mod overlay;
mod command;
mod textobject;
mod iterators;
