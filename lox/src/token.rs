use std::fmt::{Display, Formatter};

use crate::position::Position;
use crate::scanner::source_iterator::Entry;

#[derive(Debug, PartialEq, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub position: Position,
}

impl Token {
    pub(crate) fn new(token_type: TokenType, entry: Entry, length: usize) -> Token {
        Token {
            token_type,
            position: Position {
                absolute: entry.position,
                length,
            },
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
            TokenType::Plus => write!(f, "+"),
            TokenType::Star => write!(f, "*"),
            TokenType::LeftParent => write!(f, "("),
            TokenType::RightParent => write!(f, ")"),
            TokenType::Semicolon => write!(f, ";"),
            TokenType::Eof => write!(f, "EOF"),
            _ => write!(f, ""),
        }
    }
}
