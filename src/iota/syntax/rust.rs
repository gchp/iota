use std::str::Chars;
use std::iter::Peekable;
use std::iter::Enumerate;

use ::syntax::lexer::{Token, Lexer, Span};
use ::syntax::next_is;


pub struct RustSyntax;

impl Lexer for RustSyntax {
    fn handle_char(&self, ch: char, mut iter: &mut Peekable<Enumerate<Chars>>, y_pos: usize, idx: usize) -> Option<Span> {
        match ch {
            '#' => {
                let st = idx;
                let mut end = idx;
                if next_is(&mut iter, '!') || next_is(&mut iter, '[') {
                    let mut s = String::new();
                    while let Some(&(e, c)) = iter.peek() {
                        if c == '\n' { break }
                        end = e;
                        s.push(iter.next().unwrap().1)
                    }
                    return Some(Span {
                        y_pos: y_pos,
                        start: st,
                        end: end,
                        token: Token::Attribute(s),
                    });
                }
                return Some(Span{y_pos: y_pos, start: idx, end: idx + 1, token: Token::Hash})
            }

            '/' => {
                let st = idx;
                let mut end = idx;
                if next_is(&mut iter, '/') {
                    let mut s = String::from("/");
                    while let Some(&(e, c)) = iter.peek() {
                        if c == '\n' { break }
                        end = e;
                        s.push(iter.next().unwrap().1)
                    }
                    //tokens.push(Token::SingleLineComment(s));
                    return Some(Span {
                        y_pos: y_pos,
                        start: st,
                        end: end,
                        token: Token::SingleLineComment(s),
                    });
                }
                return Some(Span {
                    y_pos: y_pos,
                    start: st,
                    end: end,
                    token: Token::ForwardSlash,
                });
            }

            '"' => {
                let st = idx;
                let mut end = idx;
                let mut s = String::from("\"");
                while let Some(&(e, c)) = iter.peek() {
                    end = e;
                    s.push(iter.next().unwrap().1);
                    if c == '"' {
                        break;
                    }
                }
                return Some(Span {
                    y_pos: y_pos,
                    start: st,
                    end: end,
                    token: Token::String(s),
                });
            }

            _ => None,
        }

    }

    fn handle_ident(&self, ch: char, iter: &mut Peekable<Enumerate<Chars>>, y_pos: usize) -> Span {
        let mut ident = String::new();
        ident.push(ch);
        let start = iter.peek().unwrap().0 - 1;

        while self.is_ident(iter.peek()) {
            ident.push(iter.next().unwrap().1)
        }

        Span {
            y_pos: y_pos,
            start: start,
            end: start + ident.len() - 1,
            token: Token::Ident(ident),
        }
    }
}

