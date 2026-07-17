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

// ANCHOR: opcode
/// A single bytecode instruction for our stack machine.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OpCode {
    Push(i64),
    Negate,
    Add,
    Subtract,
    Multiply,
    Divide,
}
// ANCHOR_END: opcode

// ANCHOR: compile
/// Compile an AST into a flat list of bytecode instructions, once.
pub fn compile(expr: &Expr) -> Vec<OpCode> {
    let mut code = Vec::new();
    emit(expr, &mut code);
    code
}

fn emit(expr: &Expr, code: &mut Vec<OpCode>) {
    match expr {
        Expr::Number(value) => code.push(OpCode::Push(*value)),
        Expr::Unary {
            op: UnOp::Negate,
            operand,
        } => {
            emit(operand, code);
            code.push(OpCode::Negate);
        }
        Expr::Binary { op, left, right } => {
            emit(left, code);
            emit(right, code);
            code.push(match op {
                BinOp::Add => OpCode::Add,
                BinOp::Subtract => OpCode::Subtract,
                BinOp::Multiply => OpCode::Multiply,
                BinOp::Divide => OpCode::Divide,
            });
        }
    }
}
// ANCHOR_END: compile

// ANCHOR: vm
/// Execute bytecode on a stack machine. The compiler guarantees the stack is
/// always well-formed, so the pops here cannot fail for compiled programs.
pub fn execute(code: &[OpCode]) -> Result<i64, RuntimeError> {
    let mut stack: Vec<i64> = Vec::new();

    for instruction in code {
        match instruction {
            OpCode::Push(value) => stack.push(*value),
            OpCode::Negate => {
                let value = stack.pop().expect("stack underflow");
                stack.push(-value);
            }
            OpCode::Add => binary(&mut stack, |a, b| Ok(a + b))?,
            OpCode::Subtract => binary(&mut stack, |a, b| Ok(a - b))?,
            OpCode::Multiply => binary(&mut stack, |a, b| Ok(a * b))?,
            OpCode::Divide => binary(&mut stack, |a, b| {
                if b == 0 {
                    Err(RuntimeError::DivisionByZero)
                } else {
                    Ok(a / b)
                }
            })?,
        }
    }

    Ok(stack.pop().expect("stack underflow"))
}

fn binary(
    stack: &mut Vec<i64>,
    op: impl Fn(i64, i64) -> Result<i64, RuntimeError>,
) -> Result<(), RuntimeError> {
    let right = stack.pop().expect("stack underflow");
    let left = stack.pop().expect("stack underflow");
    stack.push(op(left, right)?);
    Ok(())
}
// ANCHOR_END: vm

// ANCHOR: run
pub fn run(source: &str) -> Result<i64, InterpretError> {
    let tokens = lex(source)?;
    let expr = parse(&tokens)?;
    let code = compile(&expr);
    Ok(execute(&code)?)
}
// ANCHOR_END: run

#[cfg(test)]
mod tests {
    use super::*;

    // ANCHOR: bytecode_test
    #[test]
    fn compiles_to_post_order_stack_bytecode() {
        // 1 + 2 * 3  ->  push 1, push 2, push 3, multiply, add
        let expr = parse(&lex("1 + 2 * 3").unwrap()).unwrap();

        assert_eq!(
            compile(&expr),
            vec![
                OpCode::Push(1),
                OpCode::Push(2),
                OpCode::Push(3),
                OpCode::Multiply,
                OpCode::Add,
            ]
        );
    }
    // ANCHOR_END: bytecode_test

    // ANCHOR: shared_test
    #[test]
    fn the_vm_satisfies_the_same_specification_as_the_interpreter() {
        for &(source, want) in interpreter_v3::PROGRAMS {
            let got = run(source).unwrap_or_else(|error| panic!("{source:?} failed: {error:?}"));
            assert_eq!(got, want, "for {source:?}");
        }
    }

    #[test]
    fn the_vm_reports_division_by_zero_too() {
        assert_eq!(
            run(interpreter_v3::DIVISION_BY_ZERO),
            Err(InterpretError::Runtime(RuntimeError::DivisionByZero))
        );
    }
    // ANCHOR_END: shared_test
}
