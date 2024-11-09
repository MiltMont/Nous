use std::{fmt::Debug, fs, path::PathBuf, rc::Rc};

use crate::{
    ast::{self, BinaryOperator, Identifier},
    parser::Parser,
};

/// A three address code program representation.
#[derive(Debug)]
pub struct Program(pub Function);

impl From<&mut TAC> for Program {
    fn from(value: &mut TAC) -> Self {
        value.to_tac_program()
    }
}

#[derive(Clone)]
pub struct Function {
    pub identifier: ast::Identifier,
    pub body: Instructions,
}

impl Debug for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "\n\tFunction(\n\tIdentifier: {:?} \n\tBody: {:?}\n\t)",
            &self.identifier, &self.body
        )
    }
}

#[derive(Clone, PartialEq, Eq)]
pub enum Instruction {
    Return(Val),
    Unary {
        operator: ast::UnaryOperator,
        src: Val,
        dst: Val,
    },
    Binary {
        binary_operator: ast::BinaryOperator,
        src_1: Val,
        src_2: Val,
        dst: Val,
    },
    Copy {
        src: Val,
        dst: Val,
    },
    Jump {
        target: Identifier,
    },
    JumpIfZero {
        condition: Val,
        target: Identifier,
    },
    JumpIfNotZero {
        condition: Val,
        target: Identifier,
    },
    Label(Identifier),
}

pub type Instructions = Vec<Instruction>;

impl Debug for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Return(arg0) => f.debug_tuple("\n\t\tReturn").field(arg0).finish(),
            Self::Unary { operator, src, dst } => {
                write!(f, "\n\t\tUnary({:?}, {:?}, {:?})", operator, src, dst)
            }
            Self::Binary {
                binary_operator,
                src_1,
                src_2,
                dst,
            } => {
                write!(
                    f,
                    "\n\t\tBinary({:?}, {:?}, {:?}, {:?})",
                    binary_operator, src_1, src_2, dst
                )
            }
            Self::Copy { src, dst } => write!(f, "\n\t\tCopy({:?}, {:?})", src, dst),
            Self::Jump { target } => write!(f, "\n\t\tJump({:?})", target),
            Self::JumpIfZero { condition, target } => {
                write!(f, "\n\t\tJumpIfZero({:?}, {:?})", condition, target)
            }
            Self::Label(identifier) => write!(f, "\n\t\tLabel({:?})", identifier),
            Self::JumpIfNotZero { condition, target } => {
                write!(f, "\n\t\tJumpIfNotZero({:?}, {:?})", condition, target)
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Val {
    Constant(i64),
    Var(ast::Identifier),
}

/// Constructs TAC intermediate representation from an ast
///
/// ```
/// # use nous::lexer::Token;
/// # use logos::Logos;
/// # use nous::parser::Parser;
/// # use nous::tac::TAC;
///
/// # let file = String::from("int main(void) { return 2; }");
///
/// let mut lexer = Token::lexer(&file);
/// let mut parser = Parser::from_lexer(&mut lexer);
/// let mut tac = TAC::from(&mut parser);
///
/// // Creating a tac program
/// let tac_program = tac.to_tac_program();
/// ```
#[derive(Debug, Clone)]
pub struct TAC {
    source: ast::Program,
    temp_count: usize,
    label_count: usize,
    instructions: Instructions,
}

impl From<String> for TAC {
    fn from(value: String) -> Self {
        let source = Parser::from(value).to_ast_program().expect("XD");
        Self {
            source,
            temp_count: 0,
            label_count: 0,
            instructions: Vec::new(),
        }
    }
}

impl From<&mut Parser> for TAC {
    fn from(value: &mut Parser) -> Self {
        let source = value.to_ast_program().expect("SOmething");

        Self {
            source,
            temp_count: 0,
            label_count: 0,
            instructions: Vec::new(),
        }
    }
}

impl From<PathBuf> for TAC {
    fn from(value: PathBuf) -> Self {
        let file = fs::read_to_string(value).expect("Should read file");

        TAC::from(file)
    }
}

#[allow(unreachable_code, unused)]
impl TAC {
    pub fn to_tac_program(&mut self) -> Program {
        self.parse_program()
    }

    fn parse_program(&mut self) -> Program {
        let function = self.parse_function(self.source.0.clone());

        Program(function)
    }

    fn parse_function(&mut self, function: ast::Function) -> Function {
        self.instructions = Vec::new();
        // let ret = self.parse_statement(function.body);
        let ret = self.parse_statement(todo!());

        self.instructions.push(ret);

        Function {
            identifier: function.name,
            body: self.instructions.clone(),
        }
    }

    fn parse_statement(&mut self, statement: ast::Statement) -> Instruction {
        match statement {
            ast::Statement::Return(expression) => {
                let val = self.parse_val(expression);

                Instruction::Return(val)
            }
            _ => todo!(),
        }
    }

    fn parse_val(&mut self, expression: ast::Expression) -> Val {
        match expression {
            ast::Expression::Constant(i) => Val::Constant(i),
            ast::Expression::Unary(op, inner) => {
                let src = self.parse_val(*inner);
                let dst_name = self.make_temporary_name();
                let dst = Val::Var(ast::Identifier(dst_name));
                self.instructions.push(Instruction::Unary {
                    operator: op,
                    src,
                    dst: dst.clone(),
                });
                dst
            }
            ast::Expression::Binary(op, e1, e2) => match op {
                ast::BinaryOperator::And => {
                    let v1 = self.parse_val(*e1);
                    let v2 = self.parse_val(*e2);

                    let temp_label_name = Rc::new(self.make_temporary_label(BinaryOperator::And));

                    self.instructions.append(
                        vec![
                            Instruction::JumpIfZero {
                                condition: v1,
                                target: Identifier(temp_label_name.to_string()),
                            },
                            Instruction::JumpIfZero {
                                condition: v2,
                                target: Identifier(temp_label_name.to_string()),
                            },
                            Instruction::Copy {
                                src: Val::Constant(1),
                                dst: Val::Var(Identifier(format!("result.{}", self.label_count))),
                            },
                            Instruction::Jump {
                                target: Identifier(format!("end.{}", self.label_count)),
                            },
                            Instruction::Label(Identifier(temp_label_name.to_string())),
                            Instruction::Copy {
                                src: Val::Constant(0),
                                dst: Val::Var(Identifier(format!("result.{}", self.label_count))),
                            },
                            Instruction::Label(Identifier(format!("end.{}", self.label_count))),
                        ]
                        .as_mut(),
                    );

                    Val::Var(Identifier(format!("result.{}", self.label_count)))
                }

                ast::BinaryOperator::Or => {
                    let v1 = self.parse_val(*e1);
                    let v2 = self.parse_val(*e2);

                    let temp_label_name = Rc::new(self.make_temporary_label(BinaryOperator::Or));

                    self.instructions.append(
                        vec![
                            Instruction::JumpIfNotZero {
                                condition: v1,
                                target: Identifier(temp_label_name.to_string()),
                            },
                            Instruction::JumpIfNotZero {
                                condition: v2,
                                target: Identifier(temp_label_name.to_string()),
                            },
                            // If no jumps are performed then both values
                            // are zero, meaning the result is 0.
                            Instruction::Copy {
                                src: Val::Constant(0),
                                dst: Val::Var(Identifier(format!("result.{}", self.label_count))),
                            },
                            Instruction::Jump {
                                target: Identifier(format!("end.{}", self.label_count)),
                            },
                            // If we jump to this label then one of the values
                            // is non-zero, meaning the result is 1.
                            Instruction::Label(Identifier(temp_label_name.to_string())),
                            Instruction::Copy {
                                src: Val::Constant(1),
                                dst: Val::Var(Identifier(format!("result.{}", self.label_count))),
                            },
                            Instruction::Label(Identifier(format!("end.{}", self.label_count))),
                        ]
                        .as_mut(),
                    );
                    Val::Var(Identifier(format!("result.{}", self.label_count)))
                }

                _ => {
                    let v1 = self.parse_val(*e1);
                    let v2 = self.parse_val(*e2);
                    let dst_name = self.make_temporary_name();
                    let dst = Val::Var(ast::Identifier(dst_name));
                    self.instructions.push(Instruction::Binary {
                        binary_operator: op,
                        src_1: v1,
                        src_2: v2,
                        dst: dst.clone(),
                    });
                    dst
                }
            },
            _ => todo!(),
        }
    }

    fn make_temporary_name(&mut self) -> String {
        self.temp_count += 1;
        format!("tmp.{}", self.temp_count)
    }

    fn make_temporary_label(&mut self, binop: BinaryOperator) -> String {
        self.label_count += 1;
        match binop {
            BinaryOperator::And => format!("and_false.{:?}", self.label_count),
            BinaryOperator::Or => format!("or_true.{:?}", self.label_count),
            _ => "Error!!".into(),
        }
    }
}
