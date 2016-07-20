use std::str::Chars;
use std::iter::Peekable;
use std::iter::Enumerate;

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
            Token::At => Some('@'),

            _ => None,
        }
    }
}

pub trait Lexer {
    fn handle_ident(&self, ch: char, iter: &mut Peekable<Enumerate<Chars>>, y_pos: usize) -> Span;
    fn handle_char(&self, ch: char, mut iter: &mut Peekable<Enumerate<Chars>>, y_pos: usize, idx: usize) -> Option<Span>;

    fn is_ident(&self, ch: Option<&(usize, char)>) -> bool {
        if let Some(&(idx, c)) = ch {
            c.is_alphabetic() || c == '_'
        } else {
            false
        }
    }

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
                        '@' => tokens.push(Span{y_pos: y_pos, start: idx, end: idx + 1, token: Token::At}),
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
    pub lexer: Box<Lexer>,
    pub file_extensions: Vec<String>,
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

    pub fn get_stream(&self, text: &str) -> Vec<Span> {
        self.lexer.get_stream(text)
    }
}

macro_rules! define_lang {
    (
        $name:ident,
        $lexer_ty:ident,
        $lexer:expr,
        extensions=[$($exts:expr),+],
        keywords=[$($keys:expr),*],
        types=[$($types:expr),*]
    ) => {
        use ::syntax::langs::$lexer_ty;

        impl SyntaxInstance {
            pub fn $name() -> SyntaxInstance {
                let mut extensions = Vec::new();
                $(
                    extensions.push($exts.into());
                )*
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
                    file_extensions: extensions,
                    keywords: keywords,
                    types: types,
                }
            }
        }
    };
}


define_lang!(rust, RustSyntax, RustSyntax,
             extensions=["rs"],
             keywords=["fn", "let", "struct", "pub", "use", "impl"],
             types=["usize", "u32", "i32", "String", "mut", "Buffer", "Option"]
);

define_lang!(python, PythonSyntax, PythonSyntax,
             extensions=["py"],
             keywords=["def", "for", "while", "if", "class", "import", "return", "from", "not", "in", "and", "else", "try", "except"],
             types=["int", "dict", "list", "typle", "type", "Exception"]
);

