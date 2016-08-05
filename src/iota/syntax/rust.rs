use std::str::Chars;
use std::iter::Peekable;

use ::syntax::lexer::{Token, Lexer};
use ::syntax::next_is;


pub struct RustSyntax;

impl Lexer for RustSyntax {
    fn handle_char(&self, ch: char, mut iter: &mut Peekable<Chars>) -> Option<Token> {
        match ch {
            '#' => {
                if next_is(&mut iter, '!') || next_is(&mut iter, '[') {
                    let mut s = String::from("#");
                    while let Some(&c) = iter.peek() {
                        if c == '\n' { break }
                        s.push(iter.next().unwrap())
                    }
                    return Some(Token::Attribute(s));
                }
                return Some(Token::Hash)
            }

            '/' => {
                if next_is(&mut iter, '/') {
                    let mut s = String::from("/");
                    s.push(iter.next().unwrap());

                    let mut doc_comment = false;
                    if next_is(&mut iter, '/') || next_is(&mut iter, '!') {
                        doc_comment = true;
                    }

                    while let Some(&c) = iter.peek() {
                        if c == '\n' { break }
                        s.push(iter.next().unwrap())
                    }

                    return Some(if doc_comment{ Token::DocComment(s) } else { Token::SingleLineComment(s) });
                }
                return Some(Token::ForwardSlash);
            }

            '"' => {
                let mut s = String::from("\"");
                while let Some(&c) = iter.peek() {
                    s.push(iter.next().unwrap());
                    if c == '\\' {
                        if let Some(&c_) = iter.peek() {
                            // this is handling escaped single quotes...
                            if c_ == '"' {
                                s.push(iter.next().unwrap());
                                continue;
                            }
                        }
                    }
                    if c == '"' {
                        break;
                    }
                }
                return Some(Token::String(s));
            }

            '\'' => {
                let mut s = String::from("'");
                while let Some(&c) = iter.peek() {
                    s.push(iter.next().unwrap());
                    if c == '\\' {
                        if let Some(&c_) = iter.peek() {
                            // this is handling escaped single quotes...
                            if c_ == '\'' {
                                s.push(iter.next().unwrap());
                                continue;
                            }
                        }
                    }
                    if !c.is_alphanumeric() {
                        break;
                    }
                }
                return Some(Token::Special(s));
            }

            _ => None,
        }

    }

    fn handle_ident(&self, ch: char, mut iter: &mut Peekable<Chars>) -> Token {
        let mut ident = String::new();
        ident.push(ch);

        while self.is_ident(iter.peek()) {
            ident.push(iter.next().unwrap())
        }

        let token;

        if next_is(&mut iter, '(') {
            // function calls or definitions
            token = Token::FunctionCallDef(ident);
        } else if next_is(&mut iter, '!') {
            // macro calls
            ident.push(iter.next().unwrap());
            token = Token::Special(ident);
        } else {
            // regular idents
            token = Token::Ident(ident);
        }

        token
    }
}

