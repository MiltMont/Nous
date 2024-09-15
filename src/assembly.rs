use std::{env::{self, consts::OS}, fmt::Debug, os};

use crate::ast::Identifier;

#[derive(Clone)]
pub struct AssemblyProgram(pub AssemblyFunction);

impl AssemblyProgram {
    pub fn format(&self) -> String {
        if env::consts::OS == "linux" {
            format!("{}\n\t.section .note.GNU-stack,'',@progbits", self.0.format())
        } else {
            self.0.format()
        }
    }
}

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

impl AssemblyFunction {
    pub fn format(&self) -> String {
        let mut result = format!("\t.globl {}\n{}:\n\tpushq\t%rbp\n\tmovq\t%rsp, %rbp\n", self.name.0, self.name.0); 

        for instruction in &self.instructions {
            result.push_str(&format!("\t{}\n",instruction.format())); 
        }

        result

    }
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

impl AssemblyInstruction {
    pub fn format(&self) -> String {
        match self {
            AssemblyInstruction::Mov { src, dst } => format!("movl\t{}, {}", src.format(), dst.format()),
            AssemblyInstruction::Unary(operator, operand) => format!("{}\t{}", operator.format(), operand.format()),
            AssemblyInstruction::AllocateStack(i) => format!("subq\t${}, %rsp", i),
            AssemblyInstruction::Ret => format!("movq\t%rbp, %rsp\n\tpopq\t%rbp\n\tret"),
        }
    }
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

impl UnaryOperator {
    pub fn format(&self) -> String {
        match self {
            UnaryOperator::Neg => String::from("negl"),
            UnaryOperator::Not => String::from("notl"),
        }
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum Operand {
    Imm(i64),
    Register(Reg),
    Pseudo(Identifier),
    Stack(i64),
}

impl Operand {
    fn format(&self) -> String {
        match self {
            Operand::Imm(i) => format!("${}", i),
            Operand::Register(r) => r.format(),  
            // Pseudo registers must be removed at this point. 
            Operand::Pseudo(p) => todo!(), 
            Operand::Stack(s) => format!("{}(%rbp)", s),
        }
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum Reg {
    AX,
    R10,
}

impl Reg {
    pub fn format(&self) -> String {
        match self {
            Reg::AX => format!("%eax"),
            Reg::R10 => format!("%r10d"),
        }
    }
}
