use std::str::Chars;
use std::iter::Peekable;
use std::iter::Enumerate;

use ::syntax::lexer::{Token, Lexer};
use ::syntax::next_is;

pub struct PythonSyntax;

impl Lexer for PythonSyntax {
    fn handle_char(&self, ch: char, mut iter: &mut Peekable<Enumerate<Chars>>, y_pos: usize, idx: usize) -> Option<Token> {
        match ch {
            '#' => {
                let st = idx;
                let mut end = idx;
                let mut s = String::from("#");
                while let Some(&(e, c)) = iter.peek() {
                    if c == '\n' { break }
                    end = e;
                    s.push(iter.next().unwrap().1)
                }
                return Some(Token::SingleLineComment(s));
            }

            '@' => {
                let st = idx;
                let mut end = idx;
                let mut s = String::from("@");
                while let Some(&(e, c)) = iter.peek() {
                    if c == '\n' { break }
                    end = e;
                    s.push(iter.next().unwrap().1)
                }
                return Some(Token::Attribute(s));
            }


            ch if ch == '"' || ch == '\'' => {
                let st = idx;
                let mut end = idx;
                let mut s= format!("{}", ch);
                while let Some(&(e, c)) = iter.peek() {
                    end = e;
                    s.push(iter.next().unwrap().1);
                    if c == ch {
                        break;
                    }
                }
                return Some(Token::String(s));
            }

            _ => None,
        }

    }

    fn handle_ident(&self, ch: char, mut iter: &mut Peekable<Enumerate<Chars>>, y_pos: usize) -> Token {
        let mut ident = String::new();
        ident.push(ch);
        let start = iter.peek().unwrap().0 - 1;

        while self.is_ident(iter.peek()) {
            ident.push(iter.next().unwrap().1)
        }


        let token;
        if next_is(&mut iter, '(') {
            // function calls or definitions
            token = Token::FunctionCallDef(ident);
        } else {
            // regular idents
            token = Token::Ident(ident);
        }

        token
    }
}

