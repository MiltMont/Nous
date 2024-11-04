use crate::errors::Result;
use crate::parser::Parser;
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
    // Relational operators
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

#[derive(PartialEq, Clone)]
pub enum Statement {
    Return(Expression),
}

impl Debug for Statement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Return(arg0) => write!(f, "Return(\n{:#?} \n\t\t)", arg0),
        }
    }
}

#[derive(PartialEq, Clone, Hash, Eq)]
pub struct Identifier(pub String);

impl Debug for Identifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "'{}'", &self.0)
    }
}

#[derive(PartialEq, Clone)]
pub struct Function {
    pub name: Identifier,
    pub body: Statement,
}

impl Debug for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Function(\n\t\tname: {:?} \n\t\tbody: \n\t\t{:?}\n\t)",
            &self.name.0, &self.body
        )
    }
}

#[derive(PartialEq, Clone)]
pub struct Program(pub Function);

impl Debug for Program {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Program(\n\t{:?}\n)", &self.0)
    }
}

impl From<&mut Parser> for Result<Program> {
    fn from(value: &mut Parser) -> Self {
        value.to_ast_program()
    }
}
