use uibuf::UIBuffer;
use keyboard::Key;
use frontends::Frontend;


/// State for the overlay
pub enum OverlayEvent {
    Finished(Option<String>),
    Ok,
}

#[deriving(Copy, Show)]
pub enum OverlayType {
    Prompt,
    None,
}


/// An interface for user interaction
///
/// This can be a prompt, autocompletion list, anything thatn requires input
/// from the user.
pub trait Overlay<T: Frontend> {
    fn draw(&mut self, frontend: &mut T, uibuf: &mut UIBuffer);
    fn draw_cursor(&mut self, frontend: &mut T);
    fn handle_key_event(&mut self, key: Option<Key>) -> OverlayEvent;
}


/// An overlay for getting input from the user
pub struct PromptOverlay {
    pub cursor_x: int,
    pub cursor_y: int,
    pub prefix: char,

    data: String,
}


impl PromptOverlay {
    pub fn new(cursor_x: int, cursor_y: int) -> PromptOverlay {
        PromptOverlay {
            cursor_x: cursor_x,
            cursor_y: cursor_y,
            data: String::new(),
            prefix: ':',
        }
    }
}


impl<T: Frontend> Overlay<T> for PromptOverlay {
    fn draw(&mut self, frontend: &mut T, uibuf: &mut UIBuffer) {
        let height = frontend.get_window_height() - 1;

        uibuf.update_cell_content(0, height, self.prefix);
        for (index, ch) in self.data.chars().enumerate() {
            uibuf.update_cell_content(index + 1, height, ch);
        }

        uibuf.draw_range(frontend, height, height+1);
    }

    fn draw_cursor(&mut self, frontend: &mut T) {
        frontend.draw_cursor(self.cursor_x, self.cursor_y);
    }

    fn handle_key_event(&mut self, key: Option<Key>) -> OverlayEvent {
        if let Some(k) = key {
            match k {
                Key::Esc => return OverlayEvent::Finished(None),
                Key::Backspace => {
                    if let Some(c) = self.data.pop() {
                        if let Some(width) = c.width(false) {
                            self.cursor_x -= width as int;
                        }
                    }
                }
                Key::Enter => {
                    // FIXME: dont clone
                    let data = self.data.clone();
                    return OverlayEvent::Finished(Some(data))
                }
                Key::Char(c) => {
                    self.data.push(c);
                    if let Some(width) = c.width(false) {
                        self.cursor_x += width as int;
                    }
                }
                _ => {}
            }
        }

        OverlayEvent::Ok
    }
}
