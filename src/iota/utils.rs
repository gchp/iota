pub fn char_width(c: char, is_cjk: bool, tab_width: usize, position: usize) -> Option<usize> {
    use unicode_width::UnicodeWidthChar;

    if c == '\t' {
        Some(tab_width - position%tab_width)
    } else if c == '\n' {
        Some(1)
    } else if is_cjk {
        UnicodeWidthChar::width_cjk(c)
    } else {
        UnicodeWidthChar::width(c)
    }
}
