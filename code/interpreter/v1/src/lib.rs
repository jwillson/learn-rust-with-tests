use std::fmt;

// ANCHOR: token
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Number(i64),
    Plus,
    Minus,
    Star,
    Slash,
    LeftParen,
    RightParen,
}
// ANCHOR_END: token

// ANCHOR: error
#[derive(Debug, PartialEq)]
pub struct LexError {
    pub unexpected: char,
}

impl fmt::Display for LexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "unexpected character {:?}", self.unexpected)
    }
}

impl std::error::Error for LexError {}
// ANCHOR_END: error

// ANCHOR: lex
pub fn lex(source: &str) -> Result<Vec<Token>, LexError> {
    let mut tokens = Vec::new();
    let mut chars = source.chars().peekable();

    while let Some(&c) = chars.peek() {
        match c {
            c if c.is_whitespace() => {
                chars.next();
            }
            '+' => push(&mut tokens, &mut chars, Token::Plus),
            '-' => push(&mut tokens, &mut chars, Token::Minus),
            '*' => push(&mut tokens, &mut chars, Token::Star),
            '/' => push(&mut tokens, &mut chars, Token::Slash),
            '(' => push(&mut tokens, &mut chars, Token::LeftParen),
            ')' => push(&mut tokens, &mut chars, Token::RightParen),
            '0'..='9' => {
                let mut value = 0i64;
                while let Some(digit) = chars.peek().and_then(|c| c.to_digit(10)) {
                    value = value * 10 + i64::from(digit);
                    chars.next();
                }
                tokens.push(Token::Number(value));
            }
            _ => return Err(LexError { unexpected: c }),
        }
    }

    Ok(tokens)
}

fn push(tokens: &mut Vec<Token>, chars: &mut std::iter::Peekable<std::str::Chars>, token: Token) {
    chars.next();
    tokens.push(token);
}
// ANCHOR_END: lex

#[cfg(test)]
mod tests {
    use super::*;

    // ANCHOR: test
    #[test]
    fn lexes_numbers_and_operators() {
        let got = lex("1 + 2 * 3").unwrap();

        assert_eq!(
            got,
            vec![
                Token::Number(1),
                Token::Plus,
                Token::Number(2),
                Token::Star,
                Token::Number(3),
            ]
        );
    }

    #[test]
    fn lexes_multi_digit_numbers_and_parentheses() {
        let got = lex("(42 - 7)").unwrap();

        assert_eq!(
            got,
            vec![
                Token::LeftParen,
                Token::Number(42),
                Token::Minus,
                Token::Number(7),
                Token::RightParen,
            ]
        );
    }

    #[test]
    fn ignores_whitespace_including_none() {
        assert_eq!(lex("1+2").unwrap(), lex("  1  +  2  ").unwrap());
    }

    #[test]
    fn rejects_an_unexpected_character() {
        let got = lex("1 $ 2");

        assert_eq!(got, Err(LexError { unexpected: '$' }));
    }
    // ANCHOR_END: test
}
