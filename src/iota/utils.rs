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

pub fn str_width(s: &str, is_cjk: bool, tab_width: uint) -> uint {
    s.chars().fold(0, |acc, c|
        acc + if c == '\t' {
            tab_width - acc%tab_width
        } else {
            c.width(is_cjk).unwrap_or(0)
        }
    )
}
