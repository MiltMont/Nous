use std::collections::{HashMap, VecDeque};

use crate::{
    assembly::{self, BinaryOperator, Instruction, Operand, Reg},
    visitor::{Visitor, VisitorWithContext},
};

/// Pushes a new AllocateStack to the front of the instruction stream.
#[derive(Debug)]
pub struct AllocateStack;

impl VisitorWithContext<assembly::Instructions, i64> for AllocateStack {
    fn visit(&mut self, item: &mut assembly::Instructions, offset: &mut i64) {
        let mut new_instructions: VecDeque<assembly::Instruction> = VecDeque::from(item.clone());
        new_instructions.push_front(Instruction::AllocateStack(*offset));
        *item = new_instructions.into();
    }
}

/// Explores the instruction set and rewrites
/// each binary operation found considering the
/// following restrictions:
///
/// 1. The `add` and `sub` instructions, like `mov`, can't use
///     memory addresses as both the source and destination operands.
///
/// 2. The `imul` instruction can't use a memory address as its
///     destination, regardless of its source operand.
///     To fix an instructions destination operand, we use the `R11` register
///     instead of `R10`.
///     To fix `imul` we load the destination into R11, multiply it by the source
///     operand, and then store the result back to the destination address.
///
/// 3. Whenever `idiv` needs to operate on a constant, we copy that constant into
///     the `R10` register first.
#[derive(Debug)]
pub struct RewriteBinaryOp;

impl Visitor<assembly::Instructions> for RewriteBinaryOp {
    fn visit(&mut self, item: &mut assembly::Instructions) {
        let mut new_instructions: Vec<assembly::Instruction> = Vec::new();
        for instruction in item.iter() {
            match instruction {
                Instruction::Idiv(operand) => {
                    new_instructions.push(Instruction::Mov {
                        src: operand.clone(),
                        dst: Operand::Register(Reg::R10),
                    });
                    new_instructions.push(Instruction::Idiv(Operand::Register(Reg::R10)));
                }
                Instruction::Binary(operator, src, dst) => match operator {
                    BinaryOperator::Add => {
                        new_instructions.push(Instruction::Mov {
                            src: src.clone(),
                            dst: Operand::Register(Reg::R10),
                        });

                        new_instructions.push(Instruction::Binary(
                            BinaryOperator::Add,
                            Operand::Register(Reg::R10),
                            dst.clone(),
                        ));
                    }
                    BinaryOperator::Sub => {
                        new_instructions.push(Instruction::Mov {
                            src: src.clone(),
                            dst: Operand::Register(Reg::R10),
                        });

                        new_instructions.push(Instruction::Binary(
                            BinaryOperator::Sub,
                            Operand::Register(Reg::R10),
                            dst.clone(),
                        ));
                    }
                    BinaryOperator::Mult => {
                        new_instructions.push(Instruction::Mov {
                            src: dst.clone(),
                            dst: Operand::Register(Reg::R11),
                        });

                        new_instructions.push(Instruction::Binary(
                            BinaryOperator::Mult,
                            src.clone(),
                            Operand::Register(Reg::R11),
                        ));

                        new_instructions.push(Instruction::Mov {
                            src: Operand::Register(Reg::R11),
                            dst: dst.clone(),
                        });
                    }
                    _ => unimplemented!(),
                },
                _ => new_instructions.push(instruction.clone()),
            }
        }
        *item = new_instructions;
    }
}

/// The `cmp` instruction can't use memory addresses for
/// both operands, also the second operand of a `cmp`
/// instruction can't be a constant either.
#[derive(Debug)]
pub struct RewriteCmp;

impl Visitor<assembly::Instructions> for RewriteCmp {
    fn visit(&mut self, instructions: &mut assembly::Instructions) {
        let mut new_instructions: assembly::Instructions = Vec::new();

        for instruction in instructions.iter() {
            if let Instruction::Cmp(a, b) = instruction {
                if matches!(a, Operand::Stack(_)) && matches!(b, Operand::Stack(_)) {
                    new_instructions.push(Instruction::Mov {
                        src: a.clone(),
                        dst: Operand::Register(Reg::R10),
                    });
                    new_instructions.push(Instruction::Cmp(Operand::Register(Reg::R10), b.clone()));
                } else if matches!(b, Operand::Imm(_)) {
                    new_instructions.push(Instruction::Mov {
                        src: b.clone(),
                        dst: Operand::Register(Reg::R11),
                    });
                    new_instructions.push(Instruction::Cmp(a.clone(), Operand::Register(Reg::R11)));
                } else {
                    new_instructions.push(instruction.clone())
                }
            } else {
                new_instructions.push(instruction.clone());
            }
        }

        *instructions = new_instructions;
    }
}

/// Rewrites move instructions, whenever both `src` and `dst`
/// are Stack operands.
#[derive(Debug)]
pub struct RewriteMov;

impl Visitor<assembly::Instructions> for RewriteMov {
    fn visit(&mut self, instructions: &mut assembly::Instructions) {
        let mut new_instructions: assembly::Instructions = Vec::new();
        for instruction in instructions.iter() {
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
        *instructions = new_instructions;
    }
}

pub struct ReplacePseudoRegisters;

impl ReplacePseudoRegisters {
    fn get_stack_value(
        &mut self,
        operand: &Operand,
        pseudo_registers: &HashMap<Operand, i64>,
    ) -> Operand {
        if let Some(op) = pseudo_registers.get(operand) {
            Operand::Stack(*op)
        } else {
            operand.clone()
        }
    }
}

impl VisitorWithContext<assembly::Instruction, HashMap<Operand, i64>> for ReplacePseudoRegisters {
    fn visit(
        &mut self,
        instruction: &mut assembly::Instruction,
        pseudo_registers: &mut HashMap<Operand, i64>,
    ) {
        match instruction {
            Instruction::Mov { src, dst } => {
                *src = self.get_stack_value(src, pseudo_registers);
                *dst = self.get_stack_value(dst, pseudo_registers);
            }
            Instruction::Unary(_unary_operator, operand) => {
                *operand = self.get_stack_value(operand, pseudo_registers);
            }
            Instruction::Binary(_binary_operator, operand, operand1) => {
                *operand = self.get_stack_value(operand, pseudo_registers);
                *operand1 = self.get_stack_value(operand1, pseudo_registers);
            }
            Instruction::Idiv(operand) => {
                *operand = self.get_stack_value(operand, pseudo_registers)
            }
            Instruction::Cmp(operand, operand1) => {
                *operand = self.get_stack_value(operand, pseudo_registers);
                *operand1 = self.get_stack_value(operand1, pseudo_registers);
            }
            Instruction::SetCC(_cond_code, operand) => {
                *operand = self.get_stack_value(operand, pseudo_registers);
            }
            _ => {}
        }
    }
}
impl VisitorWithContext<assembly::Instructions, HashMap<Operand, i64>> for ReplacePseudoRegisters {
    fn visit(
        &mut self,
        instructions: &mut assembly::Instructions,
        pseudo_registers: &mut HashMap<Operand, i64>,
    ) {
        instructions
            .iter_mut()
            .for_each(|instruction| match instruction {
                Instruction::Mov { src, dst } => {
                    *src = self.get_stack_value(src, pseudo_registers);
                    *dst = self.get_stack_value(dst, pseudo_registers);
                }
                Instruction::Unary(_unary_operator, operand) => {
                    *operand = self.get_stack_value(operand, pseudo_registers);
                }
                Instruction::Binary(_binary_operator, operand, operand1) => {
                    *operand = self.get_stack_value(operand, pseudo_registers);
                    *operand1 = self.get_stack_value(operand1, pseudo_registers);
                }
                Instruction::Idiv(operand) => {
                    *operand = self.get_stack_value(operand, pseudo_registers)
                }
                Instruction::Cmp(operand, operand1) => {
                    *operand = self.get_stack_value(operand, pseudo_registers);
                    *operand1 = self.get_stack_value(operand1, pseudo_registers);
                }
                Instruction::SetCC(_cond_code, operand) => {
                    *operand = self.get_stack_value(operand, pseudo_registers);
                }
                _ => {}
            });
    }
}
