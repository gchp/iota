extern crate rustbox;

pub fn draw(index: uint, data: String) {
    rustbox::print(0, index, rustbox::Bold, rustbox::White, rustbox::Black, data);
}
