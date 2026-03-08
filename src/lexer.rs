use std::fmt;
use std::iter::Peekable;
use std::str::Chars;

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    // Keywords
    If,
    Else,
    While,
    Int,
    Bool,
    Return,
    PrintInt,
    True,
    False,

    // Identifiers and Literals
    Identifier(String),
    IntLiteral(i32),

    // Operators
    Plus,   // +
    Minus,  // -
    Star,   // *
    Slash,  // /
    Assign, // =
    Eq,     // ==
    Ne,     // !=
    Lt,     // <
    Le,     // <=
    Gt,     // >
    Ge,     // >=

    // Delimiters
    LParen, // (
    RParen, // )
    LBrace, // {
    RBrace, // }
    Semi,   // ;
    Comma,  // ,

    EOF,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::Identifier(s) => write!(f, "{}", s),
            Token::IntLiteral(i) => write!(f, "{}", i),
            _ => write!(f, "{:?}", self),
        }
    }
}

#[derive(Debug, thiserror::Error, PartialEq, Clone)]
pub enum LexerError {
    #[error("Unexpected character: '{0}' at line {1}, col {2}")]
    UnexpectedCharacter(char, usize, usize),
    #[error("Invalid integer literal: '{0}' at line {1}, col {2}")]
    InvalidIntegerLiteral(String, usize, usize),
}

#[derive(Clone)]
pub struct Lexer<'a> {
    input: Peekable<Chars<'a>>,
    line: usize,
    column: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Lexer {
            input: input.chars().peekable(),
            line: 1,
            column: 1,
        }
    }

    fn advance(&mut self) -> Option<char> {
        let c = self.input.next();
        if let Some(ch) = c {
            if ch == '\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
        }
        c
    }

    fn peek(&mut self) -> Option<&char> {
        self.input.peek()
    }

    fn skip_whitespace_and_comments(&mut self) {
        loop {
            match self.peek() {
                Some(&c) if c.is_whitespace() => {
                    self.advance();
                }
                Some('/') => {
                    let mut iter_clone = self.input.clone();
                    iter_clone.next(); // skip first /
                    if let Some('/') = iter_clone.next() {
                        // It is a comment, consume until newline or EOF
                        while let Some(&ch) = self.peek() {
                            if ch == '\n' {
                                break;
                            }
                            self.advance();
                        }
                    } else {
                        break;
                    }
                }
                _ => break,
            }
        }
    }

    pub fn next_token(&mut self) -> Result<Token, LexerError> {
        self.skip_whitespace_and_comments();

        let start_line = self.line;
        let start_column = self.column;

        let c = match self.advance() {
            Some(c) => c,
            None => return Ok(Token::EOF),
        };

        match c {
            '(' => Ok(Token::LParen),
            ')' => Ok(Token::RParen),
            '{' => Ok(Token::LBrace),
            '}' => Ok(Token::RBrace),
            ';' => Ok(Token::Semi),
            ',' => Ok(Token::Comma),
            '+' => Ok(Token::Plus),
            '-' => Ok(Token::Minus),
            '*' => Ok(Token::Star),
            '/' => Ok(Token::Slash),
            '=' => {
                if let Some(&'=') = self.peek() {
                    self.advance();
                    Ok(Token::Eq)
                } else {
                    Ok(Token::Assign)
                }
            }
            '!' => {
                if let Some(&'=') = self.peek() {
                    self.advance();
                    Ok(Token::Ne)
                } else {
                    Err(LexerError::UnexpectedCharacter('!', start_line, start_column))
                }
            }
            '<' => {
                if let Some(&'=') = self.peek() {
                    self.advance();
                    Ok(Token::Le)
                } else {
                    Ok(Token::Lt)
                }
            }
            '>' => {
                if let Some(&'=') = self.peek() {
                    self.advance();
                    Ok(Token::Ge)
                } else {
                    Ok(Token::Gt)
                }
            }
            _ if c.is_alphabetic() || c == '_' => {
                let mut ident = String::new();
                ident.push(c);
                while let Some(&ch) = self.peek() {
                    if ch.is_alphanumeric() || ch == '_' {
                        ident.push(self.advance().unwrap());
                    } else {
                        break;
                    }
                }

                match ident.as_str() {
                    "if" => Ok(Token::If),
                    "else" => Ok(Token::Else),
                    "while" => Ok(Token::While),
                    "int" => Ok(Token::Int),
                    "bool" => Ok(Token::Bool),
                    "return" => Ok(Token::Return),
                    "printInt" => Ok(Token::PrintInt),
                    "true" => Ok(Token::True),
                    "false" => Ok(Token::False),
                    _ => Ok(Token::Identifier(ident)),
                }
            }
            _ if c.is_ascii_digit() => {
                let mut num_str = String::new();
                num_str.push(c);
                while let Some(&ch) = self.peek() {
                    if ch.is_ascii_digit() {
                        num_str.push(self.advance().unwrap());
                    } else {
                        break;
                    }
                }
                match num_str.parse::<i32>() {
                    Ok(val) => Ok(Token::IntLiteral(val)),
                    Err(_) => Err(LexerError::InvalidIntegerLiteral(
                        num_str,
                        start_line,
                        start_column,
                    )),
                }
            }
            _ => Err(LexerError::UnexpectedCharacter(c, start_line, start_column)),
        }
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Result<Token, LexerError>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.next_token() {
            Ok(Token::EOF) => None,
            other => Some(other),
        }
    }
}
