//! Iota
//!
//! A highly customisable text editor built with modern hardware in mind.
//!
//! This module contains all you need to create an `iota` executable.

#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]
#![warn(missing_docs)]

extern crate rustbox;
extern crate gapbuffer;
extern crate tempdir;
extern crate unicode_width;

pub use editor::Editor;
pub use input::Input;
pub use frontends::RustboxFrontend;
pub use modes::{StandardMode, NormalMode, Mode};

mod input;
mod utils;
mod buffer;
mod editor;
mod keyboard;
mod keymap;
mod view;
mod uibuf;
mod log;
mod frontends;
mod modes;
mod overlay;
mod command;
mod textobject;
mod iterators;
