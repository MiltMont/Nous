use crate::ast::{Expression, Function as ASTFunction, Identifier, Program as ASTProgram, Statement};

#[derive(Debug)]
pub struct Program(pub Function); 

#[derive(Debug)]
pub struct Function {
    pub name: Identifier, 
    pub instruction: Vec<Instruction>
}

#[derive(Debug)]
pub enum Instruction {
    Mov {
        src: Operand, 
        dst: Operand, 
    }, 
    Ret
}

#[derive(Debug)]
pub enum Operand {
    Imm(i64), 
    Register, 
}

pub fn parse_program(program: ASTProgram) -> Program {
    Program(parse_function(program.0))
}

pub fn parse_function(function: ASTFunction) -> Function {
    Function {
        name: function.name,
        instruction: parse_instruction(function.body),
    }
}

pub fn parse_instruction(statement: Statement) -> Vec<Instruction> {
    match statement {
        Statement::Return(exp) => vec![
            Instruction::Mov { src: parse_operand(exp), dst: Operand::Register }, 
            Instruction::Ret
        ],
    }
}

pub fn parse_operand(constant: Expression) -> Operand {
    match constant {
        Expression::Constant(i) => Operand::Imm(i),
    }
}