use miette::Diagnostic;
use thiserror::Error;

use crate::evaluation::ValueNode;
use crate::position::Position;
use crate::token::{Token, TokenType};

#[derive(Diagnostic, Error, Debug)]
pub enum LoxError {
    #[error(transparent)]
    #[diagnostic(transparent)]
    RuntimeError(RuntimeError),
    #[error(transparent)]
    #[diagnostic(transparent)]
    ParseError(ParseError),
}

#[derive(Diagnostic, Error, Debug)]
pub enum ParseError {
    #[error("Illegal Token")]
    IllegalToken {
        found: String,
        #[label("found `{found:}`")]
        position: Position,
    },
    #[error("Unexpected Token")]
    UnexpectedToken {
        found: String,
        expected: String,
        #[label("found `{found:}` expected `{expected:}`")]
        position: Position,
    },
    #[error("unclosed delimiter")]
    UnclosedDelimiter {
        #[label("unclosed delimiter")]
        start_position: Position,
        #[label = "expected closing delimiter"]
        end_position: Position,
    },
    #[error("unexpected end of token stream")]
    UnexpectedEndOfTokenStream,
}

impl ParseError {
    pub fn illegal_token(found: Token) -> LoxError {
        LoxError::ParseError(ParseError::IllegalToken {
            found: found.token_type.to_string(),
            position: found.position,
        })
    }

    pub fn unexpected_token(found: Token, expected: TokenType) -> LoxError {
        LoxError::ParseError(ParseError::UnexpectedToken {
            found: found.token_type.to_string(),
            expected: expected.to_string(),
            position: found.position,
        })
    }

    pub fn unexpected_token_raw(
        found: TokenType,
        expected: TokenType,
        position: Position,
    ) -> LoxError {
        LoxError::ParseError(ParseError::UnexpectedToken {
            found: found.to_string(),
            expected: expected.to_string(),
            position,
        })
    }

    pub fn unexpected_end_of_stream() -> LoxError {
        LoxError::ParseError(ParseError::UnexpectedEndOfTokenStream)
    }

    pub fn unclosed_delimiter(start_position: &Position, end_position: &Position) -> LoxError {
        LoxError::ParseError(ParseError::UnclosedDelimiter {
            start_position: start_position.clone(),
            end_position: end_position.clone(),
        })
    }
}

#[derive(Diagnostic, Error, Debug)]
#[error("RuntimeError")]
pub enum RuntimeError {
    #[error("TypeError")]
    TypeError {
        found: String,
        expected: String,
        #[label("no implicit conversion of type {found:} into {expected:}")]
        position: Position,
    },
}

impl RuntimeError {
    pub(crate) fn type_error(found: &ValueNode, expected: String) -> LoxError {
        LoxError::RuntimeError(RuntimeError::TypeError {
            found: format!("{:?}", found.value),
            expected,
            position: found.position.clone(),
        })
    }
}
