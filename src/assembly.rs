use std::{collections::HashMap, env, fmt::Debug};

use crate::{
    ast::{self, Identifier},
    tac,
};

#[derive(Clone)]
pub struct Program(pub Function);

impl Program {
    pub fn format(&self) -> String {
        if env::consts::OS == "linux" {
            format!(
                r#"{}.section .note.GNU-stack,"",@progbits"#,
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
    Cmp(Operand, Operand),
    Jmp(Identifier),
    JumpCC(CondCode, Identifier),
    SetCC(CondCode, Operand),
    Label(Identifier),
}

#[derive(Clone)]
pub enum CondCode {
    E,
    NE,
    G,
    GE,
    L,
    LE,
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
            Instruction::Binary(binary_operator, operand, operand1) => format!(
                "{}\t{}, {}",
                binary_operator.format(),
                operand.format(),
                operand1.format()
            ),
            Instruction::Idiv(operand) => format!("idivl\t{}", operand.format()),
            Instruction::Cdq => "cdq".to_string(),
            _ => todo!(),
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
            _ => todo!(),
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

impl BinaryOperator {
    pub fn format(&self) -> String {
        match self {
            Self::Add => "addl".to_string(),
            Self::Sub => "subl".to_string(),
            Self::Mult => "imull".to_string(),
            o => format!("The operation {o:?} should not be formated"),
        }
    }
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
            Operand::Pseudo(_) => panic!("Pseudo registers are never formated"),
            Operand::Stack(s) => format!("-{}(%rbp)", s),
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
            Reg::DX => "%edx".to_string(),
            Reg::R11 => "%r11d".to_string(),
        }
    }
}

/// Assembly program representation.
///
/// It is indended to be used as follows:
///
/// ```
/// # use nous::parser::Parser;
/// # use logos::Logos;
/// # use nous::lexer::Token;
/// # use nous::tac::TAC;
/// # use nous::assembly::Assembly;
/// # let file = String::from("int main(void) { return 2; }");
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
///
/// # use nous::parser::Parser;
/// # use logos::Logos;
/// # use nous::lexer::Token;
/// # use nous::tac::TAC;
/// # use nous::assembly::Assembly;
/// # use nous::assembly;
/// # let file = String::from("int main(void) { return 2; }");
/// # let mut lexer = Token::lexer(&file);
/// # let mut parser: Parser = Parser::build(&mut lexer);
/// # let mut tac: TAC = TAC::build(parser.to_ast_program());
/// # let mut assembly: Assembly = Assembly::new(tac.to_tac_program());
/// let mut assembly_program: assembly::Program = assembly.to_assembly_program();
/// ```
///
pub struct Assembly {
    source: tac::Program,
    pub program: Option<Program>,
    pub pseudo_registers: HashMap<Operand, i64>,
    pub offset: i64,
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
    pub fn to_assembly_program(mut self) -> Program {
        // Parsing the program
        self.parse_program();

        self.program.expect("Should return the processed program")
    }

    pub fn parse_program(&mut self) -> Program {
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
                ast::BinaryOperator::Divide => {
                    vec![
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
                    ]
                }
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
            _ => todo!(),
        }
    }

    fn parse_unary_operator(&self, operator: ast::UnaryOperator) -> UnaryOperator {
        match operator {
            ast::UnaryOperator::Negate => UnaryOperator::Neg,
            ast::UnaryOperator::Complement => UnaryOperator::Not,
            _ => todo!(),
        }
    }

    fn parse_binary_operator(&self, operator: ast::BinaryOperator) -> BinaryOperator {
        match operator {
            ast::BinaryOperator::Add => BinaryOperator::Add,
            ast::BinaryOperator::Subtract => BinaryOperator::Sub,
            ast::BinaryOperator::Multiply => BinaryOperator::Mult,
            ast::BinaryOperator::Divide => BinaryOperator::Divide,
            ast::BinaryOperator::Remainder => BinaryOperator::Remainder,
            _ => todo!(),
        }
    }

    fn parse_operand(&mut self, operand: &tac::Val) -> Operand {
        match operand {
            tac::Val::Constant(i) => Operand::Imm(*i),
            tac::Val::Var(id) => {
                // Update the offset whenever we encounter a new identifier.
                if let std::collections::hash_map::Entry::Vacant(e) =
                    self.pseudo_registers.entry(Operand::Pseudo(id.clone()))
                {
                    self.offset += 4;

                    e.insert(self.offset);
                }

                Operand::Pseudo(id.clone())
            }
        }
    }
}
