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

    pub fn rewrite_mov(&mut self) -> &mut  Self {

        let temp_instructions: Vec<AssemblyInstruction> = self.program.as_mut().unwrap().0.instructions.clone(); 
        let mut new_instructions = vec![]; 

        for instruction in temp_instructions {
            match &instruction {
                AssemblyInstruction::Mov { src, dst } => {
                    if matches!(src, Operand::Stack(_)) && matches!(dst, Operand::Stack(_)) {

                        new_instructions.push(AssemblyInstruction::Mov { src: src.clone(), dst: Operand::Register(Reg::R10) }); 
                        new_instructions.push(AssemblyInstruction::Mov { src: Operand::Register(Reg::R10), dst: dst.clone()}); 

                    } else {
                        new_instructions.push(instruction.clone()); 
                    }
                },
                _ => {new_instructions.push(instruction);} 
            }
        }

        self.program.as_mut().unwrap().0.instructions = new_instructions; 

        self
    }

    // Formating functions 

}

