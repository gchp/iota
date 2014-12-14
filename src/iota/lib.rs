#![feature(macro_rules)]

pub use editor::Editor;
pub use input::Input;

macro_rules! set_cursor_line (
    ($cursor:expr, $line:expr) => ({
        let line: *mut Line = $line;
        let line: &mut Line = unsafe { &mut *line };
        $cursor.set_line(Some(line));
    })
)

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
