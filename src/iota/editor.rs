use super::Response;
use input::Input;
use cursor::Direction;
use keyboard::Key;
use log::LogEntries;
use view::View;
use frontends::{Frontend, EditorEvent};
use modes::Mode;


#[deriving(Copy, Show)]
#[allow(dead_code)]
pub enum Command {
    SaveBuffer,
    ExitEditor,

    MoveCursor(Direction),
    LineEnd,
    LineStart,

    Delete(Direction),
    InsertTab,
    InsertLine,
    InsertChar(char),

    Undo,
    Redo,
}

pub enum EventStatus {
    Handled(Response),
    NotHandled,
}


pub struct Editor<'e, T: Frontend, M: Mode> {
    view: View<'e>,

    frontend: T,
    mode: M,

    /// Undo / redo log
    log: LogEntries,
}

impl<'e, T: Frontend, M: Mode> Editor<'e, T, M> {
    pub fn new(source: Input, mode: M, frontend: T) -> Editor<'e, T, M> {
        let height = frontend.get_window_height();
        let width = frontend.get_window_width();
        let view = View::new(source, width, height);

        Editor {
            view: view,
            frontend: frontend,
            mode: mode,
            log: LogEntries::new(),
        }
    }

    pub fn handle_key_event(&mut self, key: Option<Key>) -> Response {
        let Editor {ref mut log, ref mut view, .. } = *self;

        match self.mode.handle_key_event(key, view, log) {
            EventStatus::Handled(response) => { response }
            EventStatus::NotHandled        => { Response::Continue }
        }
    }

    pub fn draw(&mut self) {
        self.view.draw(&mut self.frontend);
        self.view.draw_status(&mut self.frontend);
        self.view.draw_cursor(&mut self.frontend);
    }

    pub fn start(&mut self) {
        loop {
            self.view.clear(&mut self.frontend);
            self.draw();
            self.frontend.present();
            let event = self.frontend.poll_event();
            if let EditorEvent::KeyEvent(key) = event {
                if let Response::Quit = self.handle_key_event(key) {
                    break;
                }
            }

        }
    }

}
