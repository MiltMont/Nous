use std::collections::{HashMap, VecDeque};

use crate::assembly::{Assembly, BinaryOperator, Instruction, Instructions, Operand, Program, Reg};

/// Visits an instance of an assembly program
/// and modifies it's instruction array.
/// Usage:
///
/// ```
/// # use nous::assembly::Assembly;
/// # use nous::visitor::AssemblyPass;
/// let file = String::from("int main(void) { return 2; }");
/// let mut assembly: Assembly = Assembly::from(file);
///
/// // The program must be parsed in order to build
/// // the visitor:
///
/// assembly.parse_program();
///
/// let mut visitor = AssemblyPass::build(assembly);
/// visitor.replace_pseudo_registers();
///
/// // Printing modified instructions can be done by
/// // calling:
///
/// visitor.print_instructions(None);
///
/// // A custom message can be passed:
///
/// visitor.print_instructions(Some("Printing instructions"));
/// ```
pub struct AssemblyPass {
    program: Program,
    instructions: Instructions,
    pseudo_registers: HashMap<Operand, i64>,
    offset: i64,
}

impl AssemblyPass {
    /// Tries to construct an AssemblyPass visitor
    /// given an Assembly instance. In order to do so, the
    /// program field in such instance must be not None.
    pub fn build(assembly: Assembly) -> Self {
        if let Some(program) = assembly.program {
            let instructions = program.0.instructions.clone();
            Self {
                program,
                instructions,
                pseudo_registers: assembly.pseudo_registers,
                offset: assembly.offset,
            }
        } else {
            panic!("The program must exists in order to create the AssemblyPass instance. Try parsing the program fist.")
        }
    }

    pub fn print_instructions(&self, debug_info: Option<&str>) {
        if let Some(info) = debug_info {
            println!("{info}");
        }
        println!("{:?}", self.instructions);
    }

    fn get_stack_value(&self, operand: &Operand) -> Operand {
        if self.pseudo_registers.contains_key(operand) {
            Operand::Stack(
                *self
                    .pseudo_registers
                    .get(operand)
                    .expect("Should return the operand stack value"),
            )
        } else {
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
            Instruction::Cmp(op1, op2) => {
                Instruction::Cmp(self.get_stack_value(op1), self.get_stack_value(op2))
            }
            Instruction::SetCC(cond, operand) => {
                Instruction::SetCC(cond.clone(), self.get_stack_value(operand))
            }
            i => i.clone(),
        }
    }

    /// Replaces pseudo registers on all instructions.
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

    /// Rewrites move instructions, whenever both `src` and `dst`
    /// are Stack operands.
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

    /// The `cmp` instruction can't use memory addresses for
    /// both operands, also the second operand of a `cmp`
    /// instruction can't be a constant either.
    pub fn rewrite_cmp(&mut self) -> &mut Self {
        let mut new_instructions: Vec<Instruction> = Vec::new();

        for instruction in &self.instructions {
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

        self.instructions = new_instructions;
        self
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
    pub fn rewrite_binop(&mut self) -> &mut Self {
        let mut new_instructions: Vec<Instruction> = Vec::new();

        for instruction in &self.instructions {
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
        self.instructions = new_instructions;
        self
    }

    /// Pushes a new AllocateStack to the front of the instruction stream.
    pub fn allocate_stack(&mut self) -> &mut Self {
        let mut new_instructions: VecDeque<Instruction> = VecDeque::from(self.instructions.clone());

        new_instructions.push_front(Instruction::AllocateStack(self.offset));

        self.instructions = new_instructions.into();

        self
    }

    /// Replaces the instruction set on
    /// the original program and returns
    /// the modified instance*.
    pub fn modify_program(&mut self) -> Program {
        self.program.0.instructions = self.instructions.clone();

        self.program.clone()
    }
}

pub trait Visitor<T> {
    fn visit(&mut self, item: &mut T);
}

pub fn apply_visitor<T, V>(vec: &mut [T], mut visitor: V)
where
    V: Visitor<T>,
{
    for item in vec.iter_mut() {
        visitor.visit(item);
    }
}

pub trait VisitorWithContext<T, C> {
    fn visit(&mut self, item: &mut T, context: &mut C);
}

pub fn apply_visitor_with_context<T, V, C>(vec: &mut [T], mut visitor: V, context: &mut C)
where
    V: VisitorWithContext<T, C>,
{
    for item in vec.iter_mut() {
        visitor.visit(item, context);
    }
}
