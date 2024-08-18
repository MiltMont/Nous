use crate::ast::Identifier;

pub struct Program(pub Function); 

pub struct Function {
   pub name: Identifier, 
    pub instruction: Vec<Instruction>
}

pub enum Instruction {
    Mov {
        src: Operand, 
        dst: Operand, 
    }, 
    Ret
}

pub enum Operand {
    Imm(i64), 
    Register, 
}