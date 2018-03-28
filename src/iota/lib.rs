//! Iota
//!
//! A highly customisable text editor built with modern hardware in mind.
//!
//! This module contains all you need to create an `iota` executable.

#![cfg_attr(feature = "clippy", feature(plugin))]
#![cfg_attr(feature = "clippy", plugin(clippy))]
#![feature(fn_traits)]
#![warn(missing_docs)]

extern crate gapbuffer;
extern crate regex;
extern crate rustbox;
extern crate tempdir;
extern crate unicode_width;
#[macro_use]
extern crate lazy_static;

pub use editor::Editor;
pub use input::Input;
pub use modes::{EmacsMode, Mode, NormalMode, StandardMode};

mod buffer;
mod command;
mod editor;
mod input;
mod iterators;
mod keyboard;
mod keymap;
mod log;
mod modes;
mod overlay;
mod textobject;
mod utils;
mod view;
