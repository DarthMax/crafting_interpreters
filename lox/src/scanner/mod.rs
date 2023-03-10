use std::string::String;

use crate::scanner::source_iterator::{Entry, SourceIterator};
use crate::token::TokenType::*;
use crate::token::{Token, TokenType};

pub(crate) mod source_iterator;

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

        while let Some(e) = source_iter.next() {
            match e.value {
                '(' => tokens.push(Token::new(LeftParent, e, 1)),
                ')' => tokens.push(Token::new(RightParent, e, 1)),
                '{' => tokens.push(Token::new(LeftBrace, e, 1)),
                '}' => tokens.push(Token::new(RightBrace, e, 1)),
                ',' => tokens.push(Token::new(Comma, e, 1)),
                '.' => tokens.push(Token::new(Dot, e, 1)),
                '-' => tokens.push(Token::new(Minus, e, 1)),
                '+' => tokens.push(Token::new(Plus, e, 1)),
                ';' => tokens.push(Token::new(Semicolon, e, 1)),
                '*' => tokens.push(Token::new(Star, e, 1)),
                '!' => scan_with_equal(&mut tokens, &mut source_iter, BangEqual, Bang, e),
                '=' => scan_with_equal(&mut tokens, &mut source_iter, EqualEqual, Equal, e),
                '<' => scan_with_equal(&mut tokens, &mut source_iter, LessEqual, Less, e),
                '>' => scan_with_equal(&mut tokens, &mut source_iter, GreaterEqual, Greater, e),
                '/' => {
                    if source_iter.next_match('/') {
                        source_iter.scan_until('\n');
                    } else {
                        tokens.push(Token::new(Slash, e, 1))
                    }
                }
                ' ' | '\r' | '\t' | '\n' => (),
                '"' => match scan_string(&mut source_iter, e) {
                    Ok(token) => tokens.push(token),
                    Err(e) => {
                        println!("Error!: {e}");
                        break;
                    }
                },
                value if value.is_numeric() => tokens.push(scan_number(&mut source_iter, e)),
                value if value.is_alphanumeric() => {
                    tokens.push(scan_identifier(&mut source_iter, e))
                }
                value => {
                    println!("Error!: Unrecognized Character '{value}'");
                    break;
                }
            }
        }

        return tokens;

        fn scan_with_equal(
            tokens: &mut Vec<Token>,
            source_iter: &mut SourceIterator,
            a: TokenType,
            b: TokenType,
            entry: Entry,
        ) {
            if source_iter.next_match('=') {
                tokens.push(Token::new(a, entry, 2))
            } else {
                tokens.push(Token::new(b, entry, 1))
            };
        }

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
            Ok(Token::new(
                token,
                first_entry,
                entry.position - first_entry.position + 1,
            ))
        }

        fn scan_number(source_iter: &mut SourceIterator, first_entry: Entry) -> Token {
            let mut found_dot = false;

            let mut last_entry = first_entry;
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
            Token::new(
                token_type,
                first_entry,
                last_entry.position - first_entry.position + 1,
            )
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

            Token::new(
                token_type,
                first_entry,
                last_entry.position - first_entry.position + 1,
            )
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
        println!("{tokens:?}")
    }
}
