use super::Response;
use input::Input;
use buffer::Direction;
use keyboard::Key;
use view::View;
use frontends::{Frontend, EditorEvent};
use modes::Mode;
use overlay::{Overlay, OverlayType, OverlayEvent, PromptOverlay};


#[derive(Copy, Show)]
pub enum Command {
    SaveBuffer,
    ExitEditor,

    MoveCursor(Direction),
    LineEnd,
    LineStart,

    Delete(Direction),
    InsertTab,
    InsertChar(char),

    SetOverlay(OverlayType),

    Undo,
    Redo,
}

pub enum EventStatus {
    Handled(Response),
    NotHandled,
}


pub struct Editor<'e, T: Frontend> {
    view: View<'e>,

    frontend: T,
    mode: Box<Mode + 'e>,
    overlay: Option<Box<Overlay<T> + 'e>>,
}

impl<'e, T: Frontend> Editor<'e, T> {
    pub fn new(source: Input, mode: Box<Mode + 'e>, frontend: T) -> Editor<'e, T> {
        let height = frontend.get_window_height();
        let width = frontend.get_window_width();
        let view = View::new(source, width, height);

        Editor {
            view: view,
            frontend: frontend,
            mode: mode,
            overlay: None,
        }
    }

    pub fn handle_key_event(&mut self, key: Option<Key>) -> Response {
        let Editor {ref mut view, .. } = *self;

        let response = match self.mode.handle_key_event(key, view) {
            EventStatus::Handled(response) => { response }
            EventStatus::NotHandled        => { Response::Continue }
        };

        let height = self.frontend.get_window_height() - 1;
        if let Response::SetOverlay(OverlayType::Prompt) = response {
            let overlay: Box<Overlay<T>> = box PromptOverlay::new(1, height as int);
            self.overlay = Some(overlay);
        }

        response
    }

    pub fn draw(&mut self) {
        self.view.draw(&mut self.frontend);
        self.view.draw_status(&mut self.frontend);

        if let Some(ref mut overlay) = self.overlay {
            overlay.draw(&mut self.frontend, &mut self.view.uibuf);
            overlay.draw_cursor(&mut self.frontend);
        } else {
            self.view.draw_cursor(&mut self.frontend);
        }
    }

    pub fn overlay_key_event(&mut self, key: Option<Key>) -> Response {
        if let Some(ref mut overlay) = self.overlay {
            if let OverlayEvent::Finished(data) = overlay.handle_key_event(key) {
                if let Some(s) = data {
                    if let Response::Quit = self.mode.interpret_input(s) {
                        return Response::Quit
                    }
                };
                return Response::SetOverlay(OverlayType::None)
            }
        };

        Response::Continue
    }

    pub fn start(&mut self) {
        loop {
            self.view.clear(&mut self.frontend);
            self.draw();
            self.frontend.present();
            let event = self.frontend.poll_event();

            if let EditorEvent::KeyEvent(key) = event {
                match self.overlay_key_event(key) {
                    Response::Quit          => break,
                    Response::SetOverlay(_) => self.overlay = None,
                    _                       => {}
                };

                if let Response::Quit = self.handle_key_event(key) {
                    break;
                }
            }
        }
    }
}
