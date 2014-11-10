extern crate rustbox;

pub fn draw(index: uint, data: String) {
    rustbox::print(0, index, rustbox::Bold, rustbox::White, rustbox::Black, data);
}

pub fn draw_cursor(x: uint, y: uint) {
    let x = x.to_int().unwrap();
    let y = y.to_int().unwrap();
    rustbox::set_cursor(x, y);
}
