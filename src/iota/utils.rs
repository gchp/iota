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

pub fn str_width(s: &str, is_cjk: bool, tab_width: usize) -> usize {
    s.chars().fold(0, |acc, c|
        acc + char_width(c, is_cjk, tab_width, acc).unwrap_or(0)
    )
}

/// Determine if a given char is alphanumeric or an underscore
pub fn is_alpha_or_(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

/// Determine the length (in bytes) of a UTF-8 character given the first byte.
pub fn char_length(b: u8) -> usize {
    if b & 0b1000_0000 == 0b0000_0000 {
        1
    } else if b & 0b1110_0000 == 0b1100_0000 {
        2
    } else if b & 0b1111_0000 == 0b1110_0000 {
        3
    } else if b & 0b1111_1000 == 0b1111_0000 {
        4
    } else {
        panic!("invalid UTF-8 character")
    }
}

/// Determine if the given u8 is the start of a UTF-8 character.
pub fn is_start_of_char(b: u8) -> bool {
    b & 0b1100_0000 != 0b1000_0000
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_alpha_or_() {
        assert!(is_alpha_or_('a'));
        assert!(is_alpha_or_('5'));
        assert!(is_alpha_or_('_'));
    }
}
