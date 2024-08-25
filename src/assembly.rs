
use crate::ast::Identifier;

#[derive(Debug, Clone)]
pub struct AssemblyProgram(pub AssemblyFunction);

#[derive(Debug, Clone)]
pub struct AssemblyFunction {
    pub name: Identifier,
    pub instructions: Vec<Instruction>,
}

#[derive(Debug, Clone)]
pub enum Instruction {
    Mov { src: Operand, dst: Operand },
    Ret,
}

#[derive(Debug, Clone)]
pub enum Operand {
    Imm(i64),
    Register,
}

