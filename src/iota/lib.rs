#![feature(unboxed_closures)]
#![feature(unsafe_destructor)]
#![feature(slicing_syntax)]
#![feature(associated_types)]

extern crate rustbox;
extern crate gapbuffer;

use overlay::OverlayType;

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

#[derive(Copy)]
pub enum Response {
    SetOverlay(OverlayType),
    Continue,
    Quit,
}
