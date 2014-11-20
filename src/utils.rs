extern crate rustbox;

pub fn draw(index: uint, data: String) {
    // clear_line(index);
    rustbox::print(0, index, rustbox::Normal, rustbox::White, rustbox::Default, data);
}

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

pub fn get_term_stats() -> String {
    format!("{}x{}", get_term_width(), get_term_height())
}

pub fn clear_line(line: uint) {
    let width = get_term_width();
    for index in range(0, width) {
        rustbox::print(index, line, rustbox::Bold, rustbox::White, rustbox::Black, String::from_str(""));
    }
}
