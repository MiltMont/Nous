use std::{borrow::Borrow, collections::HashMap};

use crate::assembly::{Instruction, Operand, Program, Reg};

/// Visits a program instruction stack
/// and makes modifications based on information
/// provided by its Assembly struct.
pub struct AssemblyPass {
    program: Program,
    instructions: Vec<Instruction>,
    pseudo_registers: HashMap<Operand, i64>,
}

/// Visits an instance of an assembly program
/// and modifies it's instruction array.
/// Usage:
///
/// ```
/// let mut program = assembly.to_assembly_program();
/// let mut visitor = AssemblyPass::new(program);
/// visitor.replace_pseudo_registers()
/// println!("{:?}", visitor.instructions);
///
/// ```
impl AssemblyPass {
    /// Constructs a visitor from a given
    /// assembly program instance.
    pub fn new(program: Program, pseudo_registers: HashMap<Operand, i64>) -> Self {
        // Takes ownership of the assembly program and clones
        // its instruction set.
        let instructions: Vec<Instruction> = program.0.instructions.clone();
        Self {
            program,
            instructions,
            pseudo_registers,
        }
    }

    pub fn print_instructions(&self, debug_info: Option<&str>) {
        if let Some(info) = debug_info {
            println!("{info}");
        }
        println!("{:?}", self.instructions);
    }

    // TODO: Implement
    //
    // rewrite_mov()
    // rewrite_binop()
    // allocate_stack()
    //

    fn get_stack_value(&self, operand: &Operand) -> Operand {
        // println!("{:?}", self.pseudo_registers.clone());
        if self.pseudo_registers.contains_key(operand) {
            Operand::Stack(
                *self
                    .pseudo_registers
                    .get(operand)
                    .expect("Should return the operand stack value"),
            )
        } else {
            // HACK: Why am I doing this?
            operand.clone()
        }
    }

    fn convert_register(&self, instruction: &Instruction) -> Instruction {
        match instruction {
            Instruction::Mov { src, dst } => Instruction::Mov {
                src: self.get_stack_value(src),
                dst: self.get_stack_value(dst),
            },
            Instruction::Unary(op, operand) => {
                Instruction::Unary(op.clone(), self.get_stack_value(operand))
            }
            Instruction::Binary(binop, x, y) => Instruction::Binary(
                binop.clone(),
                self.get_stack_value(x),
                self.get_stack_value(y),
            ),
            Instruction::Idiv(operand) => Instruction::Idiv(self.get_stack_value(operand)),
            Instruction::Cdq => Instruction::Cdq,
            Instruction::AllocateStack(i) => Instruction::AllocateStack(*i),
            Instruction::Ret => Instruction::Ret,
        }
    }

    pub fn replace_pseudo_registers(&mut self) -> &mut Self {
        let new_instructions: Vec<Instruction> = self
            .instructions
            .clone()
            .iter()
            .map(|x| self.convert_register(x))
            .collect();
        self.instructions = new_instructions;
        self
    }

    pub fn rewrite_mov(&mut self) -> &mut Self {
        let mut new_instructions: Vec<Instruction> = Vec::new();

        for instruction in &self.instructions {
            match instruction {
                Instruction::Mov { src, dst } => {
                    if matches!(src, Operand::Stack(_)) && matches!(dst, Operand::Stack(_)) {
                        new_instructions.push(Instruction::Mov {
                            src: src.clone(),
                            dst: Operand::Register(Reg::R10),
                        });
                        new_instructions.push(Instruction::Mov {
                            src: Operand::Register(Reg::R10),
                            dst: dst.clone(),
                        });
                    } else {
                        new_instructions.push(instruction.clone())
                    }
                }
                _ => new_instructions.push(instruction.clone()),
            }
        }
        self.instructions = new_instructions;
        self
    }

    pub fn rewrite_binop(&mut self) -> &mut Self {
        todo!()
    }

    pub fn allocate_stack(&mut self) -> &mut Self {
        todo!()
    }

    /// Replaces the instruction set on
    /// the original program and returns
    /// the modified instance.
    pub fn modify_program(&mut self) -> Program {
        self.program.0.instructions = self.instructions.clone();

        self.program.clone()
    }
}
