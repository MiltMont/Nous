use crate::errors::Result;
use crate::parser::Parser;
use std::fmt::Debug;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct FunctionDeclaration {
    pub name: Identifier,
    pub parameters: Vec<Identifier>,
    pub body: Option<Block>,
}

#[derive(Hash, Debug, PartialEq, Eq, Clone)]
/// A variable declaration consists of a name
/// and an optional initializer expression.
pub struct VariableDeclaration {
    pub name: Identifier,
    pub initializer: Option<Expression>,
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Block(pub BlockItems);

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

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Declaration {
    FuncDecl(FunctionDeclaration),
    VarDecl(VariableDeclaration),
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
    Conditional {
        condition: Box<Expression>,
        exp1: Box<Expression>,
        exp2: Box<Expression>,
    },
    FunctionCall {
        name: Identifier,
        arguments: Vec<Expression>,
    },
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ForInit {
    InitDecl(VariableDeclaration),
    InitExp(Option<Expression>),
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
    /// Represents compount statements.
    Compound(Block),
    /// Labels are optinal to avoid creating dummy labels
    /// during parsing.  
    Break {
        label: Option<Identifier>,
    },
    Continue {
        label: Option<Identifier>,
    },
    While {
        condition: Expression,
        body: Box<Statement>,
        identifier: Option<Identifier>,
    },
    DoWhile {
        body: Box<Statement>,
        condition: Expression,
        identifier: Option<Identifier>,
    },
    For {
        initializer: ForInit,
        condition: Option<Expression>,
        post: Option<Expression>,
        body: Box<Statement>,
        identifier: Option<Identifier>,
    },
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

#[allow(dead_code)]
type FuncitonDefinition = Function;

#[derive(PartialEq, Clone)]
pub struct Function {
    pub name: Identifier,
    pub body: Block,
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
pub struct Program(pub Vec<FunctionDeclaration>);

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
