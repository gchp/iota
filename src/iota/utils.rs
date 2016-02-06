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

/// Determine if a given char is alphanumeric or an underscore
// pub fn is_alpha_or_(c: char) -> bool {
//     c.is_alphanumeric() || c == '_'
// }

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
