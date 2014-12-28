use super::Response;
use input::Input;
use buffer::Direction;
use keyboard::Key;
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
        }
    }

    pub fn handle_key_event(&mut self, key: Option<Key>) -> Response {
        let Editor {ref mut view, .. } = *self;

        match self.mode.handle_key_event(key, view) {
            EventStatus::Handled(response) => { response }
            EventStatus::NotHandled        => { Response::Continue }
        }
    }

    pub fn save_active_buffer(&mut self) {
        let path = match self.view.buffer.file_path {
            Some(ref p) => Cow::Borrowed(p),
            None => {
                // TODO: prompt user for file name here
                Cow::Owned(Path::new("untitled"))
            },
        };

        let tmpdir = match TempDir::new_in(&Path::new("."), "iota") {
            Ok(d) => d,
            Err(e) => panic!("file error: {}", e)
        };

        let tmppath = tmpdir.path().join(Path::new("tmpfile"));

        let mut file = match File::open_mode(&tmppath, FileMode::Open, FileAccess::Write) {
            Ok(f) => f,
            Err(e) => panic!("file error: {}", e)
        };

        //TODO (lee): Is iteration still necessary in this format?
        for line in self.view.buffer.lines() {
            let result = file.write(line);

            if result.is_err() {
                // TODO(greg): figure out what to do here.
                panic!("Something went wrong while writing the file");
            }
        }

        if let Err(e) = fs::rename(&tmppath, &*path) {
            panic!("file error: {}", e);
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
