use uibuf::UIBuffer;
use keyboard::Key;
use frontends::Frontend;


/// State for the overlay
pub enum OverlayEvent {
    Finished(Option<String>),
    Ok,
}

#[derive(Copy, Show, PartialEq, Eq)]
pub enum OverlayType {
    Prompt,
}


/// An interface for user interaction
///
/// This can be a prompt, autocompletion list, anything thatn requires input
/// from the user.
pub enum Overlay {
    Prompt {
        cursor_x: uint,
        data: String,
        prefix: &'static str,
    },

    None,
}

impl Overlay {
    pub fn draw<F: Frontend>(&self, frontend: &mut F, uibuf: &mut UIBuffer) {
        match self {
            &Overlay::Prompt {prefix, ref data, ..} => {
                let height = frontend.get_window_height() - 1;
                let offset = prefix.len();

                // draw the given prefix
                for (index, ch) in prefix.chars().enumerate() {
                    uibuf.update_cell_content(index, height, ch);
                }

                // draw the overlay data
                for (index, ch) in data.chars().enumerate() {
                    uibuf.update_cell_content(index + offset, height, ch);
                }

                uibuf.draw_range(frontend, height, height+1);
            }

            _ => {}
        }
    }

    pub fn draw_cursor<F: Frontend>(&mut self, frontend: &mut F) {
        match self {
            &Overlay::Prompt {cursor_x, ..} => {
                // Prompt is always on the bottom, so we can use the
                // height given by the frontend here
                let height = frontend.get_window_height() - 1;
                frontend.draw_cursor(cursor_x as int, height as int)
            },

            _ => {}
        }
    }

    pub fn handle_key_event(&mut self, key: Key) -> OverlayEvent {
        match self {
            &Overlay::Prompt {ref mut cursor_x, ref mut data, ..} => {
                match key {
                    Key::Esc => return OverlayEvent::Finished(None),
                    Key::Backspace => {
                        if let Some(c) = data.pop() {
                            if let Some(width) = c.width(false) {
                                *cursor_x -= width;
                            }
                        }
                    }
                    Key::Enter => {
                        // FIXME: dont clone
                        let data = data.clone();
                        return OverlayEvent::Finished(Some(data))
                    }
                    Key::Char(c) => {
                        if let Some(width) = c.width(false) {
                            data.push(c);
                            *cursor_x += width;
                        }
                    }
                    _ => {}
                }
            }

            _ => {}
        }
        OverlayEvent::Ok
    }
}
