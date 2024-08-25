use std::{env, fs, ops::Neg};

use crate::{assembly::{AssemblyFunction, AssemblyProgram, Instruction as AssemblyInstruction, Operand, Reg, UnaryOperator as AssemblyUnaryOperator}, ast::{Expression, Function, Program, Statement, UnaryOperator}, tac::{Instruction, TacFunction, TacProgram, Val}};

#[derive(Clone, Debug)]
pub struct AssemblyParser {
    pub program: AssemblyProgram, 
}

/*
impl AssemblyParser {
    pub fn build(c_program: Program) -> Self {
        let program = AssemblyParser::parse_program(c_program); 

        Self {
            program
        }

    }

    pub fn write(self, path: String) {
        let contents = self.format_program(self.program.clone()); 
        fs::write(path, contents).expect("Unable to write file.");
    }

    fn parse_program(program: Program) -> AssemblyProgram {
        AssemblyProgram(AssemblyParser::parse_function(program.0))
    }
    
    fn format_program(&self, program: AssemblyProgram) -> String {
        // Check if OS is linux
        if env::consts::OS == "linux" {
            return format!(
                r##"{}.section .note.GNU-stack,"",@progbits"##,
                self.format_function(program.0)
            );
        }
        self.format_function(program.0).to_string()
    }
    

    fn parse_function(function: Function) -> AssemblyFunction {
        AssemblyFunction {
            name: function.name,
            instructions: AssemblyParser::parse_instruction(function.body),
        }
    }
    
    fn format_function(&self, function: AssemblyFunction) -> String {
        let mut func = format!("\t.globl {}\n{}:\n", function.name.0, function.name.0);
    
        for instruction in function.instructions {
            func.push_str(&format!("{}\n", self.format_instruction(instruction)));
        }
    
        func
    }

    fn parse_instruction(statement: Statement) -> Vec<Instruction> {
        match statement {
            Statement::Return(exp) => vec![
                Instruction::Mov {
                    src: AssemblyParser::parse_operand(exp),
                    dst: Operand::Register,
                },
                Instruction::Ret,
            ],
        }
    }
    
    fn format_instruction(&self, instruction: Instruction) -> String {
        match instruction {
            Instruction::Mov { src, dst } => {
                format!("\tmovl {}, {}", self.format_operand(src), self.format_operand(dst))
            }
            Instruction::Ret => "\tret".to_string(),
        }
    }

    fn parse_operand(constant: Expression) -> Operand {
        match constant {
            Expression::Constant(i) => Operand::Imm(i),
            Expression::Unary(_, _) => todo!(),
        }
    }
    
    fn format_operand(&self, operand: Operand) -> String {
        match operand {
            Operand::Imm(i) => format!("${i}"),
            Operand::Register => "%eax".to_string(),
        }
    }
    
}

 */

impl AssemblyParser {
    pub fn build(c_program: TacProgram) -> Self {
        let program = AssemblyParser::convert_program(c_program); 

        Self {
            program
        }

    }

    fn convert_program(program: TacProgram) -> AssemblyProgram {
        todo!()
    }

    fn convert_function(self, function: TacFunction) -> AssemblyFunction {
        let instructions = function.body.into_iter().map(|x| self.convert_instruction(x)).collect(); 

        AssemblyFunction { name: function.identifier, instructions: instructions }
    }

    fn convert_instruction(&self, instruction: Instruction) -> Vec<AssemblyInstruction> {
        match instruction {
            Instruction::Return(val) => {
                vec![
                    AssemblyInstruction::Mov { src: self.convert_operand(val), dst: Operand::Register(Reg::AX) },
                    AssemblyInstruction::Ret
                ]
            },
            Instruction::Unary { operator, src, dst } => {
                vec![
                    AssemblyInstruction::Mov { src: self.convert_operand(src), dst: self.convert_operand(dst.clone()) }, 
                    AssemblyInstruction::Unary(self.convert_operator(operator), self.convert_operand(dst))
                ]
            },
        }
    }

    fn convert_operator(&self, operator: UnaryOperator) -> AssemblyUnaryOperator {
        match operator {
            UnaryOperator::Complement => AssemblyUnaryOperator::Not,
            UnaryOperator::Negate => AssemblyUnaryOperator::Neg,
        }
    } 

    fn convert_operand(&self, operand: Val) -> Operand {
        match operand {
            Val::Constant(i) => Operand::Imm(i),
            Val::Var(id) => Operand::Pseudo(id),
        }
    }
}

