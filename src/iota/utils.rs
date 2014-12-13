extern crate rustbox;

pub fn draw_cursor(x: uint, y: uint) {
    let x = x.to_int().unwrap();
    let y = y.to_int().unwrap();
    rustbox::set_cursor(x, y);
}

pub fn get_term_height() -> uint {
    rustbox::height()
}

pub fn get_term_width() -> uint {
    rustbox::width()
}

#[cfg(test)]
pub fn data_from_str(s: &'static str) -> String {
    s.into_string()
}
