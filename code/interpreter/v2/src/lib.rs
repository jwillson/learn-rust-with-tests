use std::fmt;

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

// ANCHOR: ast
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Number(i64),
    Unary {
        op: UnOp,
        operand: Box<Expr>,
    },
    Binary {
        op: BinOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UnOp {
    Negate,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BinOp {
    Add,
    Subtract,
    Multiply,
    Divide,
}
// ANCHOR_END: ast

#[derive(Debug, PartialEq)]
pub enum ParseError {
    UnexpectedEnd,
    UnexpectedToken(Token),
    ExpectedRightParen,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::UnexpectedEnd => write!(f, "unexpected end of input"),
            ParseError::UnexpectedToken(token) => write!(f, "unexpected token {token:?}"),
            ParseError::ExpectedRightParen => write!(f, "expected a closing parenthesis"),
        }
    }
}

impl std::error::Error for ParseError {}

// ANCHOR: parse
pub fn parse(tokens: &[Token]) -> Result<Expr, ParseError> {
    let mut parser = Parser {
        tokens,
        position: 0,
    };
    let expr = parser.expression()?;

    match parser.peek() {
        None => Ok(expr),
        Some(token) => Err(ParseError::UnexpectedToken(token.clone())),
    }
}

struct Parser<'a> {
    tokens: &'a [Token],
    position: usize,
}

impl Parser<'_> {
    // expression = term (("+" | "-") term)*
    fn expression(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.term()?;

        while let Some(op) = self.match_additive() {
            let right = self.term()?;
            left = Expr::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    // term = factor (("*" | "/") factor)*
    fn term(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.factor()?;

        while let Some(op) = self.match_multiplicative() {
            let right = self.factor()?;
            left = Expr::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    // factor = NUMBER | "-" factor | "(" expression ")"
    fn factor(&mut self) -> Result<Expr, ParseError> {
        match self.advance() {
            Some(Token::Number(value)) => Ok(Expr::Number(*value)),
            Some(Token::Minus) => Ok(Expr::Unary {
                op: UnOp::Negate,
                operand: Box::new(self.factor()?),
            }),
            Some(Token::LeftParen) => {
                let inner = self.expression()?;
                match self.advance() {
                    Some(Token::RightParen) => Ok(inner),
                    _ => Err(ParseError::ExpectedRightParen),
                }
            }
            Some(token) => Err(ParseError::UnexpectedToken(token.clone())),
            None => Err(ParseError::UnexpectedEnd),
        }
    }

    fn match_additive(&mut self) -> Option<BinOp> {
        let op = match self.peek()? {
            Token::Plus => BinOp::Add,
            Token::Minus => BinOp::Subtract,
            _ => return None,
        };
        self.advance();
        Some(op)
    }

    fn match_multiplicative(&mut self) -> Option<BinOp> {
        let op = match self.peek()? {
            Token::Star => BinOp::Multiply,
            Token::Slash => BinOp::Divide,
            _ => return None,
        };
        self.advance();
        Some(op)
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.position)
    }

    fn advance(&mut self) -> Option<&Token> {
        let token = self.tokens.get(self.position);
        if token.is_some() {
            self.position += 1;
        }
        token
    }
}
// ANCHOR_END: parse

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_str(source: &str) -> Result<Expr, ParseError> {
        parse(&lex(source).unwrap())
    }

    // ANCHOR: test
    #[test]
    fn parses_a_single_number() {
        assert_eq!(parse_str("42").unwrap(), Expr::Number(42));
    }

    #[test]
    fn multiplication_binds_tighter_than_addition() {
        // 1 + 2 * 3  parses as  1 + (2 * 3)
        let got = parse_str("1 + 2 * 3").unwrap();

        assert_eq!(
            got,
            Expr::Binary {
                op: BinOp::Add,
                left: Box::new(Expr::Number(1)),
                right: Box::new(Expr::Binary {
                    op: BinOp::Multiply,
                    left: Box::new(Expr::Number(2)),
                    right: Box::new(Expr::Number(3)),
                }),
            }
        );
    }

    #[test]
    fn parentheses_override_precedence() {
        // (1 + 2) * 3
        let got = parse_str("(1 + 2) * 3").unwrap();

        assert_eq!(
            got,
            Expr::Binary {
                op: BinOp::Multiply,
                left: Box::new(Expr::Binary {
                    op: BinOp::Add,
                    left: Box::new(Expr::Number(1)),
                    right: Box::new(Expr::Number(2)),
                }),
                right: Box::new(Expr::Number(3)),
            }
        );
    }

    #[test]
    fn parses_unary_minus() {
        assert_eq!(
            parse_str("-5").unwrap(),
            Expr::Unary {
                op: UnOp::Negate,
                operand: Box::new(Expr::Number(5)),
            }
        );
    }

    #[test]
    fn reports_a_missing_closing_parenthesis() {
        assert_eq!(parse_str("(1 + 2"), Err(ParseError::ExpectedRightParen));
    }

    #[test]
    fn reports_trailing_tokens() {
        assert_eq!(
            parse_str("1 2"),
            Err(ParseError::UnexpectedToken(Token::Number(2)))
        );
    }
    // ANCHOR_END: test
}
