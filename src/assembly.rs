use std::fmt::Debug;

use crate::ast::Identifier;

#[derive(Clone)]
pub struct AssemblyProgram(pub AssemblyFunction);

impl Debug for AssemblyProgram {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        //f.debug_tuple("AssemblyProgram").field(&self.0).finish()
        write!(f, "Program(\n\t{:?}\n)", &self.0)
    }
}

#[derive(Clone)]
pub struct AssemblyFunction {
    pub name: Identifier,
    pub instructions: Vec<AssemblyInstruction>,
}

impl Debug for AssemblyFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Function(\n\t\tname: {:?} \n\t\tinstructions: \n\t\t{:?}\n\t)",
            &self.name.0, &self.instructions
        )
    }
}

#[derive(Clone)]
pub enum AssemblyInstruction {
    Mov { src: Operand, dst: Operand },
    Unary(UnaryOperator, Operand),
    AllocateStack(i64),
    Ret,
}

impl Debug for AssemblyInstruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Mov { src, dst } => f
                .debug_struct("\n\tMov")
                .field("src", src)
                .field("dst", dst)
                .finish(),
            Self::Unary(arg0, arg1) => f.debug_tuple("\n\tUnary").field(arg0).field(arg1).finish(),
            Self::AllocateStack(arg0) => f.debug_tuple("\n\tAllocateStack").field(arg0).finish(),
            Self::Ret => write!(f, "\n\tRet\n\t\t"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum UnaryOperator {
    Neg,
    Not,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum Operand {
    Imm(i64),
    Register(Reg),
    Pseudo(Identifier),
    Stack(i64),
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum Reg {
    AX,
    R10,
}
