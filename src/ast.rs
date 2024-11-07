use crate::errors::Result;
use crate::parser::Parser;
use std::fmt::Debug;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum BlockItem {
    S(Statement),
    D(Declaration),
}

pub type BlockItems = Vec<BlockItem>;

#[derive(Debug, PartialEq, Eq, Clone)]
/// A declaration consists of a name
/// and an optional initializer expression.
pub struct Declaration {
    pub name: Identifier,
    pub initializer: Option<Expression>,
}

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

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Expression {
    Constant(i64),
    /// This holds a variable name
    Var(Identifier),
    Unary(UnaryOperator, Box<Expression>),
    Binary(BinaryOperator, Box<Expression>, Box<Expression>),
    /// Consists of the lvalue beign updated and the expression
    /// we're assigning to that lvalue.
    Assignment(Box<Expression>, Box<Expression>),
}

#[derive(Eq, PartialEq, Clone)]
pub enum Statement {
    Return(Expression),
    /// Takes an expression node.
    Expression(Expression),
    /// Represents null statements, which are expression
    /// statements without the expression.
    Null,
}

impl Debug for Statement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Return(arg0) => write!(f, "Return(\n{:#?} \n\t\t)", arg0),
            Self::Null => write!(f, "Null"),
            Self::Expression(e) => write!(f, "Expression(\n{:#?}\n\t)", e),
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
    pub body: BlockItems,
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
