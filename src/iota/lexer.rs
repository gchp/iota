use std::str::Chars;
use std::iter::Peekable;
use std::iter::Enumerate;
use std::io::Read;
use std::path::Path;
use std::fs::File;

#[derive(Debug)]
pub enum Token {
    Ident(String),

    OpenBrace,
    CloseBrace,
    OpenParen,
    CloseParen,
    OpenSquare,
    CloseSquare,

    Newline,
    Whitespace,

    Underscore,
    Dash,

    SingleQuote,
    DoubleQuote,
    Comma,
    SemiColon,
    Colon,
    ForwardSlash,
    Pipe,
    Dot,
    Equal,
    Bang,
    Greater,
    Less,
    Hash,
    Dollar,
    Amp,
    Asterisk,
    SingleLineComment(String),
    Attribute(String),
    String(String),
}

impl Token {
    pub fn as_char(&self) -> Option<char> {
        match *self {
            Token::OpenBrace => Some('{'),
            Token::CloseBrace => Some('}'),
            Token::OpenParen => Some('('),
            Token::CloseParen => Some(')'),
            Token::OpenSquare => Some('['),
            Token::CloseSquare => Some(']'),
            Token::Newline => Some('\n'),
            Token::Whitespace => Some(' '),
            Token::Underscore => Some('_'),
            Token::Dash => Some('-'),
            Token::SingleQuote => Some('\''),
            Token::DoubleQuote => Some('"'),
            Token::Comma => Some(','),
            Token::SemiColon => Some(';'),
            Token::Colon => Some(':'),
            Token::ForwardSlash => Some('/'),
            Token::Pipe => Some('|'),
            Token::Dot => Some('.'),
            Token::Equal => Some('='),
            Token::Bang => Some('!'),
            Token::Greater => Some('>'),
            Token::Less => Some('<'),
            Token::Hash => Some('#'),
            Token::Dollar => Some('$'),
            Token::Amp => Some('&'),
            Token::Asterisk => Some('*'),

            _ => None,
        }
    }
}

pub trait Tokenizer {
    fn handle_ident(&self, ch: char, iter: &mut Peekable<Enumerate<Chars>>, y_pos: usize) -> Span;
    fn handle_char(&self, ch: char, mut iter: &mut Peekable<Enumerate<Chars>>, y_pos: usize, idx: usize) -> Option<Span>;

    fn get_stream(&self, input: &str) -> Vec<Span> {
        let mut tokens = Vec::new();
        let mut y_pos = 0;

        let mut chars = input.chars().enumerate().peekable();
        while let Some((idx, c)) = chars.next() {
            match self.handle_char(c, &mut chars, y_pos, idx) {
                Some(span) => tokens.push(span),
                None => {
                    match c {
                        ' ' => tokens.push(Span{y_pos: y_pos, start: idx, end: idx + 1, token: Token::Whitespace}),
                        '\n' => {
                            y_pos += 1;
                            tokens.push(Span{y_pos: y_pos, start: idx, end: idx + 1, token: Token::Newline})
                        }
                        '{' => tokens.push(Span{y_pos: y_pos, start: idx, end: idx + 1, token: Token::OpenBrace}),
                        '}' => tokens.push(Span{y_pos: y_pos, start: idx, end: idx + 1, token: Token::CloseBrace}),
                        '(' => tokens.push(Span{y_pos: y_pos, start: idx, end: idx + 1, token: Token::OpenParen}),
                        ')' => tokens.push(Span{y_pos: y_pos, start: idx, end: idx + 1, token: Token::CloseParen}),
                        '[' => tokens.push(Span{y_pos: y_pos, start: idx, end: idx + 1, token: Token::OpenSquare}),
                        ']' => tokens.push(Span{y_pos: y_pos, start: idx, end: idx + 1, token: Token::CloseSquare}),
                        '\'' => tokens.push(Span{y_pos: y_pos, start: idx, end: idx + 1, token: Token::SingleQuote}),
                        '"' => tokens.push(Span{y_pos: y_pos, start: idx, end: idx + 1, token: Token::DoubleQuote}),
                        ',' => tokens.push(Span{y_pos: y_pos, start: idx, end: idx + 1, token: Token::Comma}),
                        ';' => tokens.push(Span{y_pos: y_pos, start: idx, end: idx + 1, token: Token::SemiColon}),
                        ':' => tokens.push(Span{y_pos: y_pos, start: idx, end: idx + 1, token: Token::Colon}),
                        '/' => tokens.push(Span{y_pos: y_pos, start: idx, end: idx + 1, token: Token::ForwardSlash}),
                        '|' => tokens.push(Span{y_pos: y_pos, start: idx, end: idx + 1, token: Token::Pipe}),
                        '.' => tokens.push(Span{y_pos: y_pos, start: idx, end: idx + 1, token: Token::Dot}),
                        '=' => tokens.push(Span{y_pos: y_pos, start: idx, end: idx + 1, token: Token::Equal}),
                        '!' => tokens.push(Span{y_pos: y_pos, start: idx, end: idx + 1, token: Token::Bang}),
                        '>' => tokens.push(Span{y_pos: y_pos, start: idx, end: idx + 1, token: Token::Greater}),
                        '<' => tokens.push(Span{y_pos: y_pos, start: idx, end: idx + 1, token: Token::Less}),
                        '-' => tokens.push(Span{y_pos: y_pos, start: idx, end: idx + 1, token: Token::Dash}),
                        '#' => tokens.push(Span{y_pos: y_pos, start: idx, end: idx + 1, token: Token::Hash}),
                        '$' => tokens.push(Span{y_pos: y_pos, start: idx, end: idx + 1, token: Token::Dollar}),
                        '&' => tokens.push(Span{y_pos: y_pos, start: idx, end: idx + 1, token: Token::Amp}),
                        '*' => tokens.push(Span{y_pos: y_pos, start: idx, end: idx + 1, token: Token::Asterisk}),
                        _ => {
                            let ident = self.handle_ident(c, &mut chars, y_pos);
                            tokens.push(ident);
                        }
                    }
                }
            }
        }

        tokens
    }
}


#[derive(Debug)]
pub struct Span {
    pub start: usize,
    pub end: usize,
    pub token: Token,
    pub y_pos: usize,
}


pub struct SyntaxInstance {
    pub lexer: Box<Tokenizer>,
    pub file_extensions: Vec<String>,
    pub keywords: Vec<String>,
}

impl SyntaxInstance {
    pub fn rust() -> SyntaxInstance {
        let keywords = vec!(
            "".into()
            );


        SyntaxInstance {
            lexer: Box::new(RustSyntax),
            file_extensions: vec!("rs".into()),
            keywords: keywords,
        }
    }

    pub fn get_stream(&self, text: &str) -> Vec<Span> {
        self.lexer.get_stream(text)
    }
}

fn next_is(iter: &mut Peekable<Enumerate<Chars>>, ch: char) -> bool {
    if let Some(&(_, c)) = iter.peek() {
        if c == ch { true } else { false }
    } else { false }
}

pub struct RustSyntax;

impl Tokenizer for RustSyntax {
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

impl RustSyntax {
    fn is_ident(&self, ch: Option<&(usize, char)>) -> bool {
        if let Some(&(idx, c)) = ch {
            c.is_alphabetic() || c == '_'
        } else {
            false
        }
    }
}


fn main() {
    let rs = RustSyntax;

    let mut data = String::new();
    let mut f = File::open(Path::new("/home/gchp/src/github.com/gchp/iota/input.rs")).unwrap();
    let _ = f.read_to_string(&mut data);

    let tokens = rs.get_stream(&*data);

    for t in &tokens {
        println!("{:?}", t);
    }

}
