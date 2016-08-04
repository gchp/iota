use std::str::Chars;
use std::iter::Peekable;
use std::iter::Enumerate;

use super::next_is;

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
    At,
    DoubleColon,
    Number(char),
    SingleLineComment(String),
    DocComment(String),
    Attribute(String),
    String(String),
    Special(String),
    FunctionCallDef(String),
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
            Token::At => Some('@'),

            _ => None,
        }
    }
}

pub trait Lexer {
    fn handle_ident(&self, ch: char, iter: &mut Peekable<Enumerate<Chars>>, y_pos: usize) -> Token;
    fn handle_char(&self, ch: char, mut iter: &mut Peekable<Enumerate<Chars>>, y_pos: usize, idx: usize) -> Option<Token>;

    fn is_ident(&self, ch: Option<&(usize, char)>) -> bool {
        if let Some(&(idx, c)) = ch {
            c.is_alphabetic() || c == '_'
        } else {
            false
        }
    }

    fn get_stream(&self, input: &str) -> Vec<Token> {
        let mut tokens = Vec::new();
        let mut y_pos = 0;

        let mut chars = input.chars().enumerate().peekable();
        while let Some((idx, c)) = chars.next() {
            match self.handle_char(c, &mut chars, y_pos, idx) {
                Some(span) => tokens.push(span),
                None => {
                    match c {
                        ' ' => tokens.push(Token::Whitespace),
                        '\n' => {
                            y_pos += 1;
                            tokens.push(Token::Newline)
                        }
                        '{' => tokens.push(Token::OpenBrace),
                        '}' => tokens.push(Token::CloseBrace),
                        '(' => tokens.push(Token::OpenParen),
                        ')' => tokens.push(Token::CloseParen),
                        '[' => tokens.push(Token::OpenSquare),
                        ']' => tokens.push(Token::CloseSquare),
                        '\'' => tokens.push(Token::SingleQuote),
                        '"' => tokens.push(Token::DoubleQuote),
                        ',' => tokens.push(Token::Comma),
                        ';' => tokens.push(Token::SemiColon),
                        '/' => tokens.push(Token::ForwardSlash),
                        '|' => tokens.push(Token::Pipe),
                        '.' => tokens.push(Token::Dot),
                        '=' => tokens.push(Token::Equal),
                        '!' => tokens.push(Token::Bang),
                        '>' => tokens.push(Token::Greater),
                        '<' => tokens.push(Token::Less),
                        '-' => tokens.push(Token::Dash),
                        '#' => tokens.push(Token::Hash),
                        '$' => tokens.push(Token::Dollar),
                        '&' => tokens.push(Token::Amp),
                        '*' => tokens.push(Token::Asterisk),
                        '@' => tokens.push(Token::At),
                        '_' => tokens.push(Token::Underscore),
                        ':' => {
                            if next_is(&mut chars, ':') {
                                let _ = chars.next();
                                tokens.push(Token::DoubleColon)
                            } else {
                                tokens.push(Token::Colon)
                            }
                        }
                        ch if ch.is_numeric() => {
                            tokens.push(Token::Number(ch));
                        }
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


pub struct SyntaxInstance {
    pub lexer: Box<Lexer>,
    pub keywords: Vec<String>,
    pub types: Vec<String>,
}

impl SyntaxInstance {
    pub fn is_keyword(&self, s: &str) -> bool {
        self.keywords.contains(&s.into())
    }

    pub fn is_type(&self, s: &str) -> bool {
        self.types.contains(&s.into())
    }

    pub fn get_stream(&self, text: &str) -> Vec<Token> {
        self.lexer.get_stream(text)
    }
}

macro_rules! define_lang {
    (
        $name:ident,
        $lexer_ty:ident,
        $lexer:expr,
        keywords=[$($keys:expr),*],
        types=[$($types:expr),*]
    ) => {
        use ::syntax::langs::$lexer_ty;

        impl SyntaxInstance {
            pub fn $name() -> SyntaxInstance {
                let mut keywords = Vec::new();
                $(
                    keywords.push($keys.into());
                )*
                let mut types = Vec::new();
                $(
                    types.push($types.into());
                )*

                SyntaxInstance {
                    lexer: Box::new($lexer),
                    keywords: keywords,
                    types: types,
                }
            }
        }
    };
}


define_lang!(rust, RustSyntax, RustSyntax,
             keywords=["fn", "let", "struct", "pub", "use", "impl", "while", "for", "match", "return", "if", "else", "break", "mod", "extern", "crate"],
             types=["usize", "u32", "i32", "String", "mut", "Buffer", "Option", "str", "char"]
);

define_lang!(python, PythonSyntax, PythonSyntax,
             keywords=["def", "for", "while", "if", "class", "import", "return", "from", "not", "in", "and", "else", "try", "except"],
             types=["int", "dict", "list", "typle", "type", "Exception"]
);

