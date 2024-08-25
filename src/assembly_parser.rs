use std::{env, fs};

use crate::{assembly::{AssemblyFunction, AssemblyProgram, Instruction, Operand}, ast::{Expression, Function, Program, Statement}};

#[derive(Clone, Debug)]
pub struct AssemblyParser {
    pub program: AssemblyProgram, 
}

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

