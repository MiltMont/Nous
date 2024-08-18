#[derive(Debug, PartialEq)]
pub enum Expression {
    Constant(i64),
}

#[derive(Debug, PartialEq)]
pub enum Statement {
    Return(Expression),
}

#[derive(Debug, PartialEq)]
pub struct Identifier(pub String);

#[derive(Debug, PartialEq)]
pub struct Function {
    pub name: Identifier,
    pub body: Statement,
}

#[derive(Debug, PartialEq)]
pub struct Program(pub Function);
