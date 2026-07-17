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

// ANCHOR: eval
#[derive(Debug, PartialEq)]
pub enum RuntimeError {
    DivisionByZero,
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RuntimeError::DivisionByZero => write!(f, "division by zero"),
        }
    }
}

impl std::error::Error for RuntimeError {}

pub fn eval(expr: &Expr) -> Result<i64, RuntimeError> {
    match expr {
        Expr::Number(value) => Ok(*value),
        Expr::Unary {
            op: UnOp::Negate,
            operand,
        } => Ok(-eval(operand)?),
        Expr::Binary { op, left, right } => {
            let left = eval(left)?;
            let right = eval(right)?;
            match op {
                BinOp::Add => Ok(left + right),
                BinOp::Subtract => Ok(left - right),
                BinOp::Multiply => Ok(left * right),
                BinOp::Divide if right == 0 => Err(RuntimeError::DivisionByZero),
                BinOp::Divide => Ok(left / right),
            }
        }
    }
}
// ANCHOR_END: eval

// ANCHOR: run
#[derive(Debug, PartialEq)]
pub enum InterpretError {
    Lex(LexError),
    Parse(ParseError),
    Runtime(RuntimeError),
}

impl From<LexError> for InterpretError {
    fn from(error: LexError) -> InterpretError {
        InterpretError::Lex(error)
    }
}

impl From<ParseError> for InterpretError {
    fn from(error: ParseError) -> InterpretError {
        InterpretError::Parse(error)
    }
}

impl From<RuntimeError> for InterpretError {
    fn from(error: RuntimeError) -> InterpretError {
        InterpretError::Runtime(error)
    }
}

pub fn run(source: &str) -> Result<i64, InterpretError> {
    let tokens = lex(source)?;
    let expr = parse(&tokens)?;
    Ok(eval(&expr)?)
}
// ANCHOR_END: run

// ANCHOR: cases
/// The shared behaviour specification: programs and the value each must
/// produce. The bytecode VM in the next chapter runs against this exact list.
pub const PROGRAMS: &[(&str, i64)] = &[
    ("1", 1),
    ("1 + 2", 3),
    ("1 + 2 * 3", 7),
    ("(1 + 2) * 3", 9),
    ("-5 + 3", -2),
    ("10 / 2 - 1", 4),
    ("2 * (3 + 4) - 5", 9),
    ("-(2 + 3) * -4", 20),
    ("100 - 4 * 25", 0),
];

pub const DIVISION_BY_ZERO: &str = "1 / (3 - 3)";
// ANCHOR_END: cases

#[cfg(test)]
mod tests {
    use super::*;

    // ANCHOR: test
    #[test]
    fn evaluates_every_program_in_the_specification() {
        for &(source, want) in PROGRAMS {
            let got = run(source).unwrap_or_else(|error| panic!("{source:?} failed: {error:?}"));
            assert_eq!(got, want, "for {source:?}");
        }
    }

    #[test]
    fn division_by_zero_is_a_runtime_error() {
        assert_eq!(
            run(DIVISION_BY_ZERO),
            Err(InterpretError::Runtime(RuntimeError::DivisionByZero))
        );
    }

    #[test]
    fn a_syntax_error_surfaces_as_a_parse_error() {
        assert!(matches!(run("(1 + 2"), Err(InterpretError::Parse(_))));
    }
    // ANCHOR_END: test
}
