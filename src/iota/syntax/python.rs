use std::str::Chars;
use std::iter::Peekable;

use ::syntax::lexer::{Token, Lexer};
use ::syntax::next_is;

pub struct PythonSyntax;

impl Lexer for PythonSyntax {
    fn handle_char(&self, ch: char, mut iter: &mut Peekable<Chars>) -> Option<Token> {
        match ch {
            '#' => {
                let mut s = String::from("#");
                while let Some(&c) = iter.peek() {
                    if c == '\n' { break }
                    s.push(iter.next().unwrap())
                }
                Some(Token::SingleLineComment(s))
            }

            '@' => {
                let mut s = String::from("@");
                while let Some(&c) = iter.peek() {
                    if c == '\n' { break }
                    s.push(iter.next().unwrap())
                }
                Some(Token::Attribute(s))
            }


            ch if ch == '"' || ch == '\'' => {
                let mut s= format!("{}", ch);
                while let Some(&c) = iter.peek() {
                    s.push(iter.next().unwrap());
                    if c == ch {
                        break;
                    }
                }
                Some(Token::String(s))
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


        if next_is(&mut iter, '(') {
            // function calls or definitions
            Token::FunctionCallDef(ident)
        } else {
            // regular idents
            Token::Ident(ident)
        }
    }
}

