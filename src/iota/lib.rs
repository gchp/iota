pub use editor::Editor;
pub use input::Input;

mod input;
mod utils;
mod buffer;
mod editor;
mod cursor;
mod keyboard;
mod view;
mod uibuf;

#[deriving(Copy)]
pub enum Response {
    Continue,
    Quit,
}
