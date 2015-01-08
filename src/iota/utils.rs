use buffer::Buffer;

#[cfg(test)]
pub fn data_from_str(s: &'static str) -> String {
    String::from_str(s)
}

pub fn char_width(c: char, is_cjk: bool, tab_width: uint, position: uint) -> Option<uint> {
    if c == '\t' {
        Some(tab_width - position%tab_width)
    } else if c == '\n' {
        Some(1)
    } else {
        c.width(is_cjk)
    }
}

pub fn str_width(s: &str, is_cjk: bool, tab_width: uint) -> uint {
    s.chars().fold(0, |acc, c|
        acc + char_width(c, is_cjk, tab_width, acc).unwrap_or(0)
    )
}

/// Determine if a given char is alphanumeric or an underscore
pub fn is_alpha_or_(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
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
