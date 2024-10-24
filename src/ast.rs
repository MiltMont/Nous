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

// impl Debug for Expression {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         match self {
//             Self::Constant(arg0) => f.debug_tuple("Constant").field(arg0).finish(),
//             Self::Unary(arg0, arg1) => f.debug_tuple("Unary").field(arg0).field(arg1).finish(),
//             Self::Binary(arg0, arg1, arg2) => {
//                 write!(f, "Binary({:?}, \n\t{:?}, {:?})", arg0, arg1, arg2)
//             }
//         }
//     }
// }

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
