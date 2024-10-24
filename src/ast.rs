use std::fmt::Debug;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum BinaryOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Remainder,
    And,
    Or,
    Equal,
    NotEqual,
    LessThan,
    LessOrEqual,
    GreaterThan,
    GreaterOrEqual,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum UnaryOperator {
    Complement,
    Negate,
    Not,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Expression {
    Constant(i64),
    Unary(UnaryOperator, Box<Expression>),
    Binary(BinaryOperator, Box<Expression>, Box<Expression>),
}

#[derive(Debug, PartialEq, Clone)]
pub enum Statement {
    Return(Expression),
}

#[derive(PartialEq, Clone, Hash, Eq)]
pub struct Identifier(pub String);

impl Debug for Identifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "'{}'", &self.0)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Function {
    pub name: Identifier,
    pub body: Statement,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Program(pub Function);
