use uibuf::UIBuffer;
use keyboard::Key;
use frontends::Frontend;


/// State for the overlay
pub enum OverlayEvent {
    Finished(Option<String>),
    Ok,
}

#[derive(Copy, Show)]
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
        cursor_y: uint,
        data: String,
        prefix: &'static str,
    },

    None,
}

impl Overlay {
    pub fn is_none(&self) -> bool {
        match self {
            &Overlay::None => true,
            _ => false
        }
    }

    pub fn draw<F: Frontend>(&self, frontend: &mut F, uibuf: &mut UIBuffer) {
        match self {
            &Overlay::Prompt {prefix, ref data, ..} => {
                let height = frontend.get_window_height() - 1;
                let offset = prefix.len();


                for (index, ch) in prefix.chars().enumerate() {
                    uibuf.update_cell_content(index, height, ch);
                }
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
            &Overlay::Prompt {cursor_x, cursor_y, ..} => {
                frontend.draw_cursor(cursor_x as int, cursor_y as int)
            },

            _ => {}
        }
    }

    pub fn handle_key_event(&mut self, key: Option<Key>) -> OverlayEvent {
        match self {
            &Overlay::Prompt {ref mut cursor_x, ref mut data, ..} => {
                if let Some(k) = key {
                    match k {
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
                            data.push(c);
                            if let Some(width) = c.width(false) {
                                *cursor_x += width;
                            }
                        }
                        _ => {}
                    }
                }
            }

            _ => {}
        }
        OverlayEvent::Ok
    }
}
