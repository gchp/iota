//! Iota
//!
//! A highly customisable text editor built with modern hardware in mind.
//!
//! This module contains all you need to create an `iota` executable.

#![warn(missing_docs)]

extern crate rustbox;
extern crate gapbuffer;
extern crate tempdir;
extern crate unicode_width;

pub use editor::Editor;
pub use input::Input;
pub use frontends::RustboxFrontend;
pub use frontends::EditorEvent;
pub use modes::{StandardMode, NormalMode, Mode};
pub use keyboard::Key;

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
