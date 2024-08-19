use std::env;

use crate::ast::{
    Expression, Function as ASTFunction, Identifier, Program as ASTProgram, Statement,
};

#[derive(Debug)]
pub struct Program(pub Function);

#[derive(Debug)]
pub struct Function {
    pub name: Identifier,
    pub instructions: Vec<Instruction>,
}

#[derive(Debug)]
pub enum Instruction {
    Mov { src: Operand, dst: Operand },
    Ret,
}

#[derive(Debug)]
pub enum Operand {
    Imm(i64),
    Register,
}

pub fn parse_program(program: ASTProgram) -> Program {
    Program(parse_function(program.0))
}

pub fn format_program(program: Program) -> String {
    // Check if OS is linux
    if env::consts::OS == "linux" {
        return format!(
            r##"{}.section .note.GNU-stack,"",@progbits"##,
            format_function(program.0)
        );
    }
    format!("{}", format_function(program.0))
}

pub fn parse_function(function: ASTFunction) -> Function {
    Function {
        name: function.name,
        instructions: parse_instruction(function.body),
    }
}

pub fn format_function(function: Function) -> String {
    let mut func = format!("\t.globl {}\n{}:\n", function.name.0, function.name.0);

    for instruction in function.instructions {
        func.push_str(&format!("{}\n", format_instruction(instruction)));
    }

    func
}

pub fn parse_instruction(statement: Statement) -> Vec<Instruction> {
    match statement {
        Statement::Return(exp) => vec![
            Instruction::Mov {
                src: parse_operand(exp),
                dst: Operand::Register,
            },
            Instruction::Ret,
        ],
    }
}

pub fn format_instruction(instruction: Instruction) -> String {
    match instruction {
        Instruction::Mov { src, dst } => {
            format!("\tmovl {}, {}", format_operand(src), format_operand(dst))
        }
        Instruction::Ret => format!("\tret"),
    }
}

pub fn parse_operand(constant: Expression) -> Operand {
    match constant {
        Expression::Constant(i) => Operand::Imm(i),
    }
}

pub fn format_operand(operand: Operand) -> String {
    match operand {
        Operand::Imm(i) => format!("${i}"),
        Operand::Register => format!("%eax"),
    }
}
