use std::fmt::{Display, Formatter};
use std::string::String;

use crate::scanner::source_iterator::{Entry, SourceIterator};
use crate::scanner::TokenType::*;

mod source_iterator;

pub struct Scanner {
    code: String,
}

impl Scanner {
    pub fn new(code: String) -> Self {
        Scanner { code }
    }

    pub fn scan(&self) -> Vec<Token> {
        let mut tokens: Vec<Token> = Vec::new();
        let mut source_iter = SourceIterator::new(self.code.clone());

        loop {
            match source_iter.next() {
                Some(
                    e @ Entry {
                        value,
                        line,
                        column,
                        ..
                    },
                ) => match value {
                    '(' => tokens.push(Token::new(LeftParent, line, column)),
                    ')' => tokens.push(Token::new(RightParent, line, column)),
                    '{' => tokens.push(Token::new(LeftBrace, line, column)),
                    '}' => tokens.push(Token::new(RightBrace, line, column)),
                    ',' => tokens.push(Token::new(Comma, line, column)),
                    '.' => tokens.push(Token::new(Dot, line, column)),
                    '-' => tokens.push(Token::new(Minus, line, column)),
                    '+' => tokens.push(Token::new(Plus, line, column)),
                    ';' => tokens.push(Token::new(Semicolon, line, column)),
                    '*' => tokens.push(Token::new(Star, line, column)),
                    '!' => {
                        let token = if source_iter.next_match('=') {
                            BangEqual
                        } else {
                            Bang
                        };
                        tokens.push(Token::new(token, line, column))
                    }
                    '=' => {
                        let token = if source_iter.next_match('=') {
                            EqualEqual
                        } else {
                            Equal
                        };
                        tokens.push(Token::new(token, line, column))
                    }
                    '<' => {
                        let token = if source_iter.next_match('=') {
                            LessEqual
                        } else {
                            Less
                        };
                        tokens.push(Token::new(token, line, column))
                    }
                    '>' => {
                        let token = if source_iter.next_match('=') {
                            GreaterEqual
                        } else {
                            Greater
                        };
                        tokens.push(Token::new(token, line, column))
                    }
                    '/' => {
                        if source_iter.next_match('/') {
                            source_iter.scan_until('\n');
                        } else {
                            tokens.push(Token::new(Slash, line, column))
                        }
                    }
                    ' ' | '\r' | '\t' | '\n' => (),
                    '"' => match scan_string(&mut source_iter, e) {
                        Ok(token) => tokens.push(token),
                        Err(e) => {
                            println!("Error!: {}", e);
                            break;
                        }
                    },
                    value if value.is_numeric() => tokens.push(scan_number(&mut source_iter, e)),
                    value if value.is_alphanumeric() => {
                        tokens.push(scan_identifier(&mut source_iter, e))
                    }
                    value => {
                        println!("Error!: Unrecognized Character '{}'", value);
                        break;
                    }
                },
                None => break,
            }
        }

        return tokens;

        fn scan_string(
            source_iter: &mut SourceIterator,
            first_entry: Entry,
        ) -> Result<Token, String> {
            let entry = source_iter.scan_until('"');

            if entry.is_none() {
                return Err("Unterminated String".to_string());
            }

            let entry = entry.unwrap();

            let value = source_iter.substring(first_entry.position + 1, entry.position - 1);
            let token = StringToken { value };
            Ok(Token::new(token, first_entry.line, first_entry.column))
        }

        fn scan_number(source_iter: &mut SourceIterator, first_entry: Entry) -> Token {
            let mut found_dot = false;

            let mut last_entry = first_entry.clone();
            loop {
                match (source_iter.peek(), source_iter.peek_next()) {
                    (Some(c), _) if c.is_numeric() => {
                        last_entry = source_iter.next().unwrap();
                    }
                    (Some(c), Some(d)) if c == '.' && !found_dot && d.is_numeric() => {
                        found_dot = true;
                        last_entry = source_iter.next().unwrap();
                    }
                    _ => break,
                }
            }

            let value = source_iter
                .substring(first_entry.position, last_entry.position)
                .parse::<f64>()
                .unwrap();
            let token_type = Number { value };
            Token::new(token_type, first_entry.line, first_entry.column)
        }

        fn scan_identifier(source_iter: &mut SourceIterator, first_entry: Entry) -> Token {
            let mut last_entry = first_entry;
            loop {
                match source_iter.peek() {
                    Some(e) if !e.is_alphanumeric() => break,
                    None => break,
                    _ => last_entry = source_iter.next().unwrap(),
                }
            }

            let value = source_iter.substring(first_entry.position, last_entry.position);

            let token_type = match value.as_ref() {
                "and" => And,
                "class" => Class,
                "else" => Else,
                "false" => False,
                "for" => For,
                "fun" => Fun,
                "if" => If,
                "nil" => Nil,
                "or" => Or,
                "print" => Print,
                "return" => Return,
                "super" => Super,
                "this" => This,
                "true" => True,
                "var" => Var,
                "while" => While,
                _ => Identifier { value },
            };

            Token::new(token_type, first_entry.line, first_entry.column)
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub line: u32,
    pub column: u32,
}

impl Token {
    pub fn new(token_type: TokenType, line: u32, column: u32) -> Token {
        Token {
            token_type,
            line,
            column,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum TokenType {
    // Single-character tokens.
    LeftParent,
    RightParent,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,

    // One or two character tokens.
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    // Literals.
    Identifier { value: String },
    StringToken { value: String },
    Number { value: f64 },

    // Keywords.
    And,
    Class,
    Else,
    False,
    Fun,
    For,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,

    Eof,
}

impl Display for TokenType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Plus => write!(f, "+"),
            Star => write!(f, "*"),
            LeftParent => write!(f, "("),
            RightParent => write!(f, ")"),
            _ => write!(f, ""),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn foo() {
        let scanner = Scanner::new("2.hallowelt".to_string());
        let tokens = scanner.scan();
        println!("{:?}", tokens)
    }
}
