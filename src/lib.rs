pub use editor::Editor;

pub enum Response {
    Continue,
    Quit,
}

mod utils;
mod buffer;
mod editor;
mod cursor;
mod keyboard;
mod view;
