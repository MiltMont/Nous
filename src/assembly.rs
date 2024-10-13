use std::{
    collections::{HashMap, VecDeque},
    env,
    fmt::Debug,
};

use crate::{
    ast::{self},
    tac::{self},
};

#[derive(Clone)]
pub struct Program(pub Function);

impl Program {
    pub fn format(&self) -> String {
        if env::consts::OS == "linux" {
            format!(
                "{}\n\t.section .note.GNU-stack,'',@progbits",
                self.0.format()
            )
        } else {
            self.0.format()
        }
    }
}

impl Debug for Program {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        //f.debug_tuple("AssemblyProgram").field(&self.0).finish()
        write!(f, "Program(\n\t{:?}\n)", &self.0)
    }
}

#[derive(Clone)]
pub struct Function {
    pub name: ast::Identifier,
    pub instructions: Vec<Instruction>,
}

impl Function {
    pub fn format(&self) -> String {
        let mut result = format!(
            "\t.globl {}\n{}:\n\tpushq\t%rbp\n\tmovq\t%rsp, %rbp\n",
            self.name.0, self.name.0
        );

        for instruction in &self.instructions {
            result.push_str(&format!("\t{}\n", instruction.format()));
        }

        result
    }
}

impl Debug for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Function(\n\t\tname: {:?} \n\t\tinstructions: \n\t\t{:?}\n\t)",
            &self.name.0, &self.instructions
        )
    }
}

#[derive(Clone)]
pub enum Instruction {
    Mov { src: Operand, dst: Operand },
    Unary(UnaryOperator, Operand),
    Binary(BinaryOperator, Operand, Operand),
    Idiv(Operand),
    Cdq,
    AllocateStack(i64),
    Ret,
}

impl Instruction {
    #[allow(unused_variables)]
    pub fn format(&self) -> String {
        match self {
            Instruction::Mov { src, dst } => {
                format!("movl\t{}, {}", src.format(), dst.format())
            }
            Instruction::Unary(operator, operand) => {
                format!("{}\t{}", operator.format(), operand.format())
            }
            Instruction::AllocateStack(i) => format!("subq\t${}, %rsp", i),
            Instruction::Ret => "movq\t%rbp, %rsp\n\tpopq\t%rbp\n\tret".to_string(),
            Instruction::Binary(binary_operator, operand, operand1) => todo!(),
            Instruction::Idiv(operand) => todo!(),
            Instruction::Cdq => todo!(),
        }
    }
}

impl Debug for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Mov { src, dst } => f
                .debug_struct("\n\tMov")
                .field("src", src)
                .field("dst", dst)
                .finish(),
            Self::Unary(arg0, arg1) => f.debug_tuple("\n\tUnary").field(arg0).field(arg1).finish(),
            Self::AllocateStack(arg0) => f.debug_tuple("\n\tAllocateStack").field(arg0).finish(),
            Self::Ret => write!(f, "\n\tRet\n\t\t"),
            Self::Idiv(operand) => f.debug_tuple("\n\tIdiv").field(operand).finish(),
            Self::Cdq => write!(f, "\n\tCdq"),
            Self::Binary(operator, src, dst) => f
                .debug_tuple("\n\tBinary")
                .field(operator)
                .field(src)
                .field(dst)
                .finish(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum UnaryOperator {
    Neg,
    Not,
}

impl UnaryOperator {
    pub fn format(&self) -> String {
        match self {
            UnaryOperator::Neg => String::from("negl"),
            UnaryOperator::Not => String::from("notl"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BinaryOperator {
    Add,
    Sub,
    Mult,
    Divide,
    Remainder,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum Operand {
    Imm(i64),
    Register(Reg),
    Pseudo(ast::Identifier),
    Stack(i64),
}

impl Operand {
    fn format(&self) -> String {
        match self {
            Operand::Imm(i) => format!("${}", i),
            Operand::Register(r) => r.format(),
            // TODO: Pseudo registers must be removed at this point.
            Operand::Pseudo(_) => todo!(),
            Operand::Stack(s) => format!("{}(%rbp)", s),
        }
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum Reg {
    AX,
    DX,
    R10,
    R11,
}

impl Reg {
    pub fn format(&self) -> String {
        match self {
            Reg::AX => "%eax".to_string(),
            Reg::R10 => "%r10d".to_string(),
            Reg::DX => todo!(),
            Reg::R11 => todo!(),
        }
    }
}

/// Assembly program representation.
///
/// It is indended to be used as follows:
///
/// ```
/// let mut lexer = Token::lexer(&file);
/// let mut parser: Parser = Parser::build(&mut lexer);
/// let mut tac: TAC = TAC::build(parser.to_ast_program());
/// let mut assembly: Assembly = Assembly::new(tac.to_tac_program());
/// ```
///
/// You can obtain an assembly program representation
/// by calling `.to_assembly_program()`
///
/// ```
/// let mut assembly_program: assembly::Program = assembly.to_assembly_program();
/// ```
///
pub struct Assembly {
    source: tac::Program,
    program: Option<Program>,
    pseudo_registers: HashMap<Operand, i64>,
    offset: i64,
}

impl Assembly {
    /// Creates an Assembly object.
    pub fn new(tac_program: tac::Program) -> Self {
        Self {
            source: tac_program,
            program: None,
            pseudo_registers: HashMap::new(),
            offset: 0,
        }
    }

    /// Converts an Assembly object into an Assembly Program object.
    pub fn to_assembly_program(&mut self) -> Program {
        // Parsing the program
        self.parse_program();

        //
        // Program post-processing.
        //
        self.program = self
            .replace_pseudo_registers()
            .rewrite_mov()
            .allocate_stack();

        self.program.clone().expect("Returning program")
    }

    fn parse_program(&mut self) -> Program {
        self.program = Some(Program(self.parse_function(self.source.0.clone())));

        self.program.clone().expect("Returning program")
    }

    fn parse_function(&mut self, function: tac::Function) -> Function {
        let mut instructions = Vec::new();
        for instruction in function.body {
            // Moves each element in self.parse_instruction into the instructions
            // vec
            instructions.append(&mut self.parse_instruction(instruction));
        }

        Function {
            name: function.identifier,
            instructions,
        }
    }

    #[allow(unused_variables)]
    fn parse_instruction(&mut self, instruction: tac::Instruction) -> Vec<Instruction> {
        match instruction {
            tac::Instruction::Return(val) => {
                vec![
                    Instruction::Mov {
                        src: self.parse_operand(&val),
                        dst: Operand::Register(Reg::AX),
                    },
                    Instruction::Ret,
                ]
            }
            tac::Instruction::Unary { operator, src, dst } => {
                vec![
                    Instruction::Mov {
                        src: self.parse_operand(&src),
                        dst: self.parse_operand(&dst),
                    },
                    Instruction::Unary(
                        self.parse_unary_operator(operator),
                        self.parse_operand(&dst),
                    ),
                ]
            }
            tac::Instruction::Binary {
                binary_operator,
                src_1,
                src_2,
                dst,
            } => match binary_operator {
                ast::BinaryOperator::Divide => vec![
                    Instruction::Mov {
                        src: self.parse_operand(&src_1),
                        dst: Operand::Register(Reg::AX),
                    },
                    Instruction::Cdq,
                    Instruction::Idiv(self.parse_operand(&src_2)),
                    Instruction::Mov {
                        src: Operand::Register(Reg::AX),
                        dst: self.parse_operand(&dst),
                    },
                ],
                ast::BinaryOperator::Remainder => vec![
                    Instruction::Mov {
                        src: self.parse_operand(&src_1),
                        dst: Operand::Register(Reg::AX),
                    },
                    Instruction::Cdq,
                    Instruction::Idiv(self.parse_operand(&src_2)),
                    Instruction::Mov {
                        src: Operand::Register(Reg::DX),
                        dst: self.parse_operand(&dst),
                    },
                ],
                _ => {
                    vec![
                        Instruction::Mov {
                            src: self.parse_operand(&src_1),
                            dst: self.parse_operand(&dst),
                        },
                        Instruction::Binary(
                            self.parse_binary_operator(binary_operator),
                            self.parse_operand(&src_2),
                            self.parse_operand(&dst),
                        ),
                    ]
                }
            },
        }
    }

    fn parse_unary_operator(&self, operator: ast::UnaryOperator) -> UnaryOperator {
        match operator {
            ast::UnaryOperator::Negate => UnaryOperator::Neg,
            ast::UnaryOperator::Complement => UnaryOperator::Not,
        }
    }

    fn parse_binary_operator(&self, operator: ast::BinaryOperator) -> BinaryOperator {
        match operator {
            ast::BinaryOperator::Add => BinaryOperator::Add,
            ast::BinaryOperator::Subtract => BinaryOperator::Sub,
            ast::BinaryOperator::Multiply => BinaryOperator::Mult,
            ast::BinaryOperator::Divide => BinaryOperator::Divide,
            ast::BinaryOperator::Remainder => BinaryOperator::Remainder,
        }
    }

    fn parse_operand(&mut self, operand: &tac::Val) -> Operand {
        match operand {
            tac::Val::Constant(i) => Operand::Imm(*i),
            tac::Val::Var(id) => {
                // Update the offset whenever we encounter a new identifier.
                if !self
                    .pseudo_registers
                    .contains_key(&Operand::Pseudo(id.clone()))
                {
                    self.offset += 4;
                }

                self.pseudo_registers
                    .insert(Operand::Pseudo(id.clone()), self.offset);

                Operand::Pseudo(id.clone())
            }
        }
    }

    /// Obtain the stack value of the operand
    fn obtain_stack_value(&self, operand: Operand) -> Operand {
        if self.pseudo_registers.contains_key(&operand) {
            Operand::Stack(
                *self
                    .pseudo_registers
                    .get(&operand)
                    .expect("Getting operand stack value"),
            )
        } else {
            // TODO: Is this necessary?
            operand
        }
    }

    /// Converts pseudo registers into their corresponding
    /// stack values
    fn convert_register(&mut self, instruction: Instruction) -> Instruction {
        match instruction {
            Instruction::Mov { src, dst } => Instruction::Mov {
                src: self.obtain_stack_value(src),
                dst: self.obtain_stack_value(dst),
            },
            Instruction::Unary(s, d) => Instruction::Unary(s, self.obtain_stack_value(d)),
            Instruction::AllocateStack(i) => Instruction::AllocateStack(i),
            Instruction::Ret => Instruction::Ret,
            Instruction::Binary(binary_operator, operand, operand1) => Instruction::Binary(
                binary_operator,
                self.obtain_stack_value(operand),
                self.obtain_stack_value(operand1),
            ),
            Instruction::Idiv(operand) => Instruction::Idiv(self.obtain_stack_value(operand)),
            Instruction::Cdq => Instruction::Cdq,
        }
    }

    // TODO: Modify these functions to be private

    /// If `self.program` exists then this method modifies
    /// the array of instructions on `self.program.0.instructions`
    /// to replace pseudo registers with their corresponding stack
    /// values
    pub fn replace_pseudo_registers(&mut self) -> &mut Self {
        let temp: Vec<Instruction> = self
            .program
            .as_mut()
            .expect("Cloning program instructions")
            .0
            .instructions
            .clone();

        let new_instructions: Vec<Instruction> =
            temp.into_iter().map(|x| self.convert_register(x)).collect();

        // Update instructions
        self.program
            .as_mut()
            .expect("Updating instructions")
            .0
            .instructions = new_instructions;

        self
    }

    /// Add an `Instruction::AllocateStack(self.offset)` instruction
    /// on the head of the instruction stack.
    pub fn allocate_stack(&mut self) -> Option<Program> {
        if let Some(program) = &self.program {
            let mut instructions: VecDeque<Instruction> =
                VecDeque::from(program.0.instructions.clone());
            let stack = Instruction::AllocateStack(self.offset);

            instructions.push_front(stack);

            Some(Program(Function {
                name: program.0.name.clone(),
                instructions: Vec::from(instructions),
            }))
        } else {
            None
        }
    }

    /// Rewrites the `Instruction::Mov` instructions in the instruction
    /// stream whenever both source and destination are both
    /// Stack operands.
    pub fn rewrite_mov(&mut self) -> &mut Self {
        let temp_instructions: Vec<Instruction> = self
            .program
            .as_mut()
            .expect("Cloning instructions")
            .0
            .instructions
            .clone();
        let mut new_instructions = Vec::new();

        for instruction in temp_instructions {
            match &instruction {
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
                        new_instructions.push(instruction.clone());
                    }
                }
                _ => new_instructions.push(instruction),
            }
        }

        self.program
            .as_mut()
            .expect("Modifying instruction stack")
            .0
            .instructions = new_instructions;

        self
    }
}
