pub use editor::Editor;

mod utils;
mod buffer;
mod editor;
mod cursor;
mod keyboard;
mod view;
mod uibuf;

pub enum Response {
    Continue,
    Quit,
}
