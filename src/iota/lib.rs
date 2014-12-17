#![feature(unboxed_closures)]
#![feature(unsafe_destructor)]

extern crate rustbox;

pub use editor::Editor;
pub use input::Input;

mod input;
mod utils;
mod buffer;
mod editor;
mod cursor;
mod keyboard;
mod keymap;
mod view;
mod uibuf;

#[deriving(Copy)]
pub enum Response {
    Continue,
    Quit,
}
