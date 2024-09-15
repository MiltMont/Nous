use std::{clone, collections::HashMap, path::is_separator, vec};

use crate::{
    assembly::{
        AssemblyFunction, AssemblyInstruction, AssemblyProgram, Operand, Reg,
        UnaryOperator as AssemblyUnaryOperator,
    },
    ast::UnaryOperator,
    tac::{Instruction, TacFunction, TacProgram, Val},
};

#[derive(Clone, Debug)]
pub struct AssemblyParser {
    pub source: TacProgram,
    pub program: Option<AssemblyProgram>,
    pub pseudo_registers: HashMap<Operand, i64>,
    offset: i64,
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
    pub fn new(tac_program: TacProgram) -> Self {
        Self {
            source: tac_program,
            program: None,
            pseudo_registers: HashMap::new(),
            offset: 0,
        }
    }

    pub fn convert_program(&mut self) -> AssemblyProgram {
        self.program = Some(AssemblyProgram(
            self.convert_function(self.source.0.clone()),
        ));

        self.program.clone().unwrap()
    }

    fn convert_function(&mut self, function: TacFunction) -> AssemblyFunction {
        let mut instructions = Vec::new();

        for instruction in function.body {
            instructions.append(&mut self.convert_instruction(instruction));
        }

        AssemblyFunction {
            name: function.identifier,
            instructions,
        }
    }

    fn convert_instruction(&mut self, instruction: Instruction) -> Vec<AssemblyInstruction> {
        match instruction {
            Instruction::Return(val) => {
                vec![
                    AssemblyInstruction::Mov {
                        src: self.convert_operand(val),
                        dst: Operand::Register(Reg::AX),
                    },
                    AssemblyInstruction::Ret,
                ]
            }
            Instruction::Unary { operator, src, dst } => {
                vec![
                    AssemblyInstruction::Mov {
                        src: self.convert_operand(src),
                        dst: self.convert_operand(dst.clone()),
                    },
                    AssemblyInstruction::Unary(
                        self.convert_operator(operator),
                        self.convert_operand(dst),
                    ),
                ]
            }
        }
    }

    fn convert_operator(&self, operator: UnaryOperator) -> AssemblyUnaryOperator {
        match operator {
            UnaryOperator::Complement => AssemblyUnaryOperator::Not,
            UnaryOperator::Negate => AssemblyUnaryOperator::Neg,
        }
    }

    fn convert_operand(&mut self, operand: Val) -> Operand {
        match operand {
            Val::Constant(i) => Operand::Imm(i),
            Val::Var(id) => {
                // Updating offset whenever we encounter a new identifier
                if !self
                    .pseudo_registers
                    .contains_key(&Operand::Pseudo(id.clone()))
                {
                    self.offset += 4;
                }

                self.pseudo_registers
                    .insert(Operand::Pseudo(id.clone()), self.offset);

                Operand::Pseudo(id)
            }
        }
    }

    fn obtain_stack(&self, operand: Operand) -> Operand {
        if self.pseudo_registers.contains_key(&operand) {
            Operand::Stack(*self.pseudo_registers.get(&operand).unwrap())
        } else {
            operand
        }
    }

    fn convert_register(&mut self, instruction: AssemblyInstruction) -> AssemblyInstruction {
        match instruction {
            AssemblyInstruction::Mov { src, dst } => AssemblyInstruction::Mov {
                src: self.obtain_stack(src),
                dst: self.obtain_stack(dst),
            },
            AssemblyInstruction::Unary(s, d) => AssemblyInstruction::Unary(s, self.obtain_stack(d)),
            AssemblyInstruction::AllocateStack(i) => AssemblyInstruction::AllocateStack(i),
            AssemblyInstruction::Ret => AssemblyInstruction::Ret,
        }
    }

    pub fn replace_pseudo_registers(mut self) -> Option<AssemblyProgram> {
        if let Some(program) = &self.program {
            let temp: Vec<AssemblyInstruction> = program.clone().0.instructions;
            let test: Vec<AssemblyInstruction> =
                temp.into_iter().map(|x| self.convert_register(x)).collect();

            Some(AssemblyProgram(AssemblyFunction {
                name: self.program.unwrap().0.name,
                instructions: test,
            }))
        } else {
            None
        }
    }

    pub fn replace_pseudo_reg(&mut self) -> &mut Self  {
        // TODO: Make this safe by removing unwraps.
        let temp_instructions: Vec<AssemblyInstruction> = self.program.as_mut().unwrap().0.instructions.clone(); 

        let new: Vec<AssemblyInstruction> = temp_instructions.into_iter().map(
            |x| self.convert_register(x)
        ).collect();

        self.program.as_mut().unwrap().0.instructions = new;  

        self 
    }


    pub fn allocate_stack(&mut self) -> Option<AssemblyProgram> {
        if let Some(program) = &self.program {
            // TODO: Fix this crappy implementation.
            let instructions = program.0.instructions.clone(); 

            let stack = AssemblyInstruction::AllocateStack(self.offset);

            let mut new_instructions: Vec<AssemblyInstruction> = vec![]; 

            new_instructions.push(stack); 

            for instruction in instructions {
                new_instructions.push(instruction); 
            }

            Some(AssemblyProgram(AssemblyFunction {
                name: self.program.clone().unwrap().0.name, 
                instructions: new_instructions 
            }))

        } else {
            None
        }
    }
}
