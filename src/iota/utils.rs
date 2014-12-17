use rustbox::RustBox;

pub fn draw_cursor(rb: &RustBox, x: uint, y: uint) {
    let x = x.to_int().unwrap();
    let y = y.to_int().unwrap();
    rb.set_cursor(x, y);
}

pub fn get_term_height(rb: &RustBox) -> uint {
    rb.height()
}

pub fn get_term_width(rb: &RustBox) -> uint {
    rb.width()
}

#[cfg(test)]
pub fn data_from_str(s: &'static str) -> String {
    s.into_string()
}
