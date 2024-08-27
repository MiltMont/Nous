use crate::ast::Identifier;

#[derive(Debug, Clone)]
pub struct AssemblyProgram(pub AssemblyFunction);

#[derive(Debug, Clone)]
pub struct AssemblyFunction {
    pub name: Identifier,
    pub instructions: Vec<AssemblyInstruction>,
}

#[derive(Debug, Clone)]
pub enum AssemblyInstruction {
    Mov { src: Operand, dst: Operand },
    Unary(UnaryOperator, Operand),
    AllocateStack(i64),
    Ret,
}

#[derive(Debug, Clone)]
pub enum UnaryOperator {
    Neg,
    Not,
}

#[derive(Debug, Clone)]
pub enum Operand {
    Imm(i64),
    Register(Reg),
    Pseudo(Identifier),
    Stack(i64),
}

#[derive(Debug, Clone)]
pub enum Reg {
    AX,
    R10,
}
