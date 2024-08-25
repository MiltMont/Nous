use std::fmt::Debug;

#[derive(Debug, PartialEq, Clone)]
pub enum UnaryOperator {
    Complement,
    Negate,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Expression {
    Constant(i64),
    Unary(UnaryOperator, Box<Expression>),
}

#[derive(Debug, PartialEq, Clone)]
pub enum Statement {
    Return(Expression),
}

#[derive(PartialEq, Clone)]
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
