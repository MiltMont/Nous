use crate::errors::Result;
use crate::parser::Parser;
use std::fmt::Debug;

#[derive(PartialEq, Eq, Clone)]
pub enum BlockItem {
    S(Statement),
    D(Declaration),
}

impl Debug for BlockItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::S(arg0) => write!(f, "\n{arg0:?}"),
            Self::D(arg0) => write!(f, "\n{arg0:?}"),
        }
    }
}

pub type BlockItems = Vec<BlockItem>;

#[derive(Hash, Debug, PartialEq, Eq, Clone)]
/// A declaration consists of a name
/// and an optional initializer expression.
pub struct Declaration {
    pub name: Identifier,
    pub initializer: Option<Expression>,
}

#[derive(Hash, Debug, PartialEq, Eq, Clone)]
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

#[derive(Hash, Debug, PartialEq, Eq, Clone)]
pub enum UnaryOperator {
    Complement,
    Negate,
    Not,
}

#[derive(Hash, Debug, PartialEq, Eq, Clone)]
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

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum Statement {
    Return(Expression),
    /// Takes an expression node.
    Expression(Expression),
    If {
        condition: Expression,
        then: Box<Statement>,
        else_statement: Option<Box<Statement>>,
    },
    /// Represents null statements, which are expression
    /// statements without the expression.
    Null,
}

#[derive(PartialEq, Clone, Hash, Eq)]
pub struct Identifier(pub String);

impl From<&str> for Identifier {
    fn from(value: &str) -> Self {
        Self(value.into())
    }
}

impl From<String> for Identifier {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<&String> for Identifier {
    fn from(value: &String) -> Self {
        Self(value.into())
    }
}

impl From<Identifier> for String {
    fn from(value: Identifier) -> Self {
        value.0
    }
}

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
            "Function(\n\t\tname: {:?} \n\t\tbody: \n\t\t{:#?}\n\t)",
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
