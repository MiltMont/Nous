use std::{fmt::Debug, fs, path::PathBuf};

use crate::{
    ast::{self, BinaryOperator, Declaration, Identifier},
    parser::Parser,
    visitor::validation_passes,
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
        let mut source = Parser::from(value)
            .to_ast_program()
            .expect("Should return a program");

        // Validation passes are performed
        validation_passes(&mut source);

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
        for block in function.body.0 {
            self.process_block(block);
        }

        Function {
            identifier: function.name,
            body: self.instructions.clone(),
        }
    }

    fn process_block(&mut self, block: ast::BlockItem) {
        match block {
            ast::BlockItem::S(statement) => {
                if let Some(instruction) = self.parse_statement(statement) {
                    self.instructions.push(instruction);
                }
            }
            ast::BlockItem::D(declaration) => {
                self.process_declaration(declaration);
            }
        }
    }

    fn process_declaration(&mut self, declaration: Declaration) {
        if let Some(x) = declaration.initializer {
            // If a declaration includes an initializer,
            // we’ll handle it like a normal variable assignment
            let expression = ast::Expression::Assignment(
                Box::new(ast::Expression::Var(declaration.name)),
                Box::new(x),
            );
            self.parse_val(expression);
        }
    }

    fn parse_statement(&mut self, statement: ast::Statement) -> Option<Instruction> {
        match statement {
            ast::Statement::Return(expression) => {
                let val = self.parse_val(expression);

                Some(Instruction::Return(val))
            }
            // To convert an expression statement to TACKY, we just process
            // the inner expression. This will return a new temporary variable thath
            // holds the result of the expression, but we wont use that variable
            // again during TAC generation
            ast::Statement::Expression(expression) => {
                let val = self.parse_val(expression);
                None
            }
            // We wont emit instructions for a null statement
            ast::Statement::Null => None,
            ast::Statement::If {
                condition,

                then,
                else_statement,
            } => {
                let result_of_condition = self.parse_val(condition);
                let end_label = self.make_label("end");
                if let Some(else_stmt) = else_statement {
                    // A statement of the form `if(<condition>) then <statement1> else <statement2>`
                    // transaltes to:
                    // <instructions_for_condition>
                    // c = <result_of_condition>
                    // JumpIfZero(c, else_label)
                    // <instructions for statement1>
                    // Jump(end)
                    // Label(else_label)
                    // <instructions_for_statement2>
                    // Label(end)
                    let else_label = self.make_label("else");

                    self.instructions.push(Instruction::JumpIfZero {
                        condition: result_of_condition,
                        target: (&else_label).into(),
                    });

                    if let Some(instructions_for_statement1) = self.parse_statement(*then) {
                        self.instructions.push(instructions_for_statement1);
                    }

                    self.instructions.push(Instruction::Jump {
                        target: (&end_label).into(),
                    });
                    self.instructions
                        .push(Instruction::Label(else_label.into()));

                    if let Some(instructions_for_statement2) = self.parse_statement(*else_stmt) {
                        self.instructions.push(instructions_for_statement2);
                    }

                    self.instructions.push(Instruction::Label(end_label.into()));
                } else {
                    // A statement of the form `if(<condition>) then <statement>`
                    // should translate to:
                    //
                    // <instructions for condition>
                    // c = <result_of_condition>
                    // JumpIfZero(c, end)
                    // <instructions_for_statement>
                    // Label(end)
                    self.instructions.push(Instruction::JumpIfZero {
                        condition: result_of_condition,
                        target: (&end_label).into(),
                    });

                    let instructions_for_statement = self.parse_statement(*then);
                    if let Some(instructions) = instructions_for_statement {
                        self.instructions.push(instructions);
                    }
                    self.instructions.push(Instruction::Label(end_label.into()));
                };
                None
            }
            ast::Statement::Compound(block) => {
                for item in block.0 {
                    self.process_block(item);
                }
                None
            }
            ast::Statement::Break { label } => {
                // Since loop labeling is applied we
                // can guarantee that this is Some()
                // HACK: Should I push into instructions or return?
                self.instructions.push(Instruction::Jump {
                    target: format!("break_{}", label.unwrap().0).into(),
                });
                None
            }
            ast::Statement::Continue { label } => {
                // Since loop labeling is applied we
                // can guarantee that this is Some()
                // HACK: Should I push into instructions or return?
                self.instructions.push(Instruction::Jump {
                    target: format!("continue_{}", label.unwrap().0).into(),
                });
                None
            }
            ast::Statement::While {
                condition,
                body,
                identifier,
            } => {
                //Label(continue_label)
                //<instructions for condition>
                //v = <result of condition>
                //JumpIfZero(v, break_label)
                //<instructions for body>
                //Jump(continue_label)
                //Label(break_label)
                let continue_label: Identifier =
                    format!("continue_{}", identifier.as_ref().unwrap().0).into();
                let break_label: Identifier =
                    format!("break_{}", identifier.as_ref().unwrap().0).into();

                self.instructions
                    .push(Instruction::Label(continue_label.clone()));

                let instructions_for_condition = self.parse_val(condition);
                self.instructions.push(Instruction::JumpIfZero {
                    condition: instructions_for_condition,
                    target: break_label.clone(),
                });

                if let Some(instructions_for_body) = self.parse_statement(*body) {
                    self.instructions.push(instructions_for_body);
                }

                self.instructions.push(Instruction::Jump {
                    target: continue_label,
                });

                self.instructions.push(Instruction::Label(break_label));

                None
            }
            ast::Statement::DoWhile {
                body,
                condition,
                identifier,
            } => {
                //Label(start)
                //<instructions for body>
                //Label(continue_label)
                //<instructions for condition>
                //v = <result of condition>
                //JumpIfNotZero(v, start)
                //Label(break_label)
                let start: Identifier = format!("start_{}", identifier.as_ref().unwrap().0).into();
                self.instructions.push(Instruction::Label(start.clone()));
                if let Some(instructions_for_body) = self.parse_statement(*body) {
                    self.instructions.push(instructions_for_body);
                }
                self.instructions.push(Instruction::Label(
                    format!("continue_{}", identifier.as_ref().unwrap().0).into(),
                ));
                let instructions_for_condition = self.parse_val(condition);
                self.instructions.push(Instruction::JumpIfNotZero {
                    condition: instructions_for_condition,
                    target: start,
                });
                self.instructions.push(Instruction::Label(
                    format!("break_{}", identifier.as_ref().unwrap().0).into(),
                ));

                None
            }
            ast::Statement::For {
                initializer,
                condition,
                post,
                body,
                identifier,
            } => {
                //  First we process the initializer
                match initializer {
                    ast::ForInit::InitDecl(declaration) => {
                        self.process_declaration(declaration);
                    }
                    ast::ForInit::InitExp(expression) => {
                        if let Some(exp) = expression {
                            self.parse_val(exp);
                        }
                    }
                }

                let start: Identifier = format!("start_{}", identifier.as_ref().unwrap().0).into();
                let break_label: Identifier =
                    format!("break_{}", identifier.as_ref().unwrap().0).into();
                let continue_label: Identifier =
                    format!("continue_{}", identifier.unwrap().0).into();

                self.instructions.push(Instruction::Label(start.clone()));

                let result_of_condition = condition.map(|condition| self.parse_val(condition));

                if let Some(val) = result_of_condition {
                    self.instructions.push(Instruction::JumpIfZero {
                        condition: val,
                        target: break_label.clone(),
                    });
                }

                if let Some(instructions_for_body) = self.parse_statement(*body) {
                    self.instructions.push(instructions_for_body);
                }

                self.instructions.push(Instruction::Label(continue_label));

                if let Some(post) = post {
                    self.parse_val(post);
                }

                self.instructions.push(Instruction::Jump { target: start });

                self.instructions.push(Instruction::Label(break_label));

                None
            }
        }
    }

    fn parse_val(&mut self, expression: ast::Expression) -> Val {
        match expression {
            ast::Expression::Constant(i) => Val::Constant(i),
            ast::Expression::Unary(op, inner) => {
                let src = self.parse_val(*inner);
                let dst_name = self.make_temporary_name();
                let dst = Val::Var(dst_name.into());
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

                    //let temp_label_name = Rc::new(self.make_temporary_label(BinaryOperator::And));
                    let temp_label_name = self.make_label("and");

                    self.instructions.append(
                        vec![
                            Instruction::JumpIfZero {
                                condition: v1,
                                target: (&temp_label_name).into(),
                            },
                            Instruction::JumpIfZero {
                                condition: v2,
                                target: (&temp_label_name).into(),
                            },
                            Instruction::Copy {
                                src: Val::Constant(1),
                                dst: Val::Var(format!("result.{}", self.label_count).into()),
                            },
                            Instruction::Jump {
                                target: format!("end.{}", self.label_count).into(),
                            },
                            Instruction::Label((&temp_label_name).into()),
                            Instruction::Copy {
                                src: Val::Constant(0),
                                dst: Val::Var(format!("result.{}", self.label_count).into()),
                            },
                            Instruction::Label(format!("end.{}", self.label_count).into()),
                        ]
                        .as_mut(),
                    );

                    Val::Var(format!("result.{}", self.label_count).into())
                }

                ast::BinaryOperator::Or => {
                    let v1 = self.parse_val(*e1);
                    let v2 = self.parse_val(*e2);

                    //let temp_label_name = Rc::new(self.make_temporary_label(BinaryOperator::Or));
                    let temp_label_name = self.make_label("or");

                    self.instructions.append(
                        vec![
                            Instruction::JumpIfNotZero {
                                condition: v1,
                                target: (&temp_label_name).into(),
                            },
                            Instruction::JumpIfNotZero {
                                condition: v2,
                                target: (&temp_label_name).into(),
                            },
                            // If no jumps are performed then both values
                            // are zero, meaning the result is 0.
                            Instruction::Copy {
                                src: Val::Constant(0),
                                dst: Val::Var(format!("result.{}", self.label_count).into()),
                            },
                            Instruction::Jump {
                                target: format!("end.{}", self.label_count).into(),
                            },
                            // If we jump to this label then one of the values
                            // is non-zero, meaning the result is 1.
                            Instruction::Label((&temp_label_name).into()),
                            Instruction::Copy {
                                src: Val::Constant(1),
                                dst: Val::Var(format!("result.{}", self.label_count).into()),
                            },
                            Instruction::Label(format!("end.{}", self.label_count).into()),
                        ]
                        .as_mut(),
                    );
                    Val::Var(format!("result.{}", self.label_count).into())
                }
                _ => {
                    let v1 = self.parse_val(*e1);
                    let v2 = self.parse_val(*e2);
                    let dst_name = self.make_temporary_name();
                    let dst = Val::Var(dst_name.into());
                    self.instructions.push(Instruction::Binary {
                        binary_operator: op,
                        src_1: v1,
                        src_2: v2,
                        dst: dst.clone(),
                    });
                    dst
                }
            },
            ast::Expression::Var(i) => Val::Var(i),
            ast::Expression::Assignment(a, rhs) => {
                assert!(matches!(*a, ast::Expression::Var(_)));

                let result = self.parse_val(*rhs);
                let dst = self.parse_val(*a);

                self.instructions.push(Instruction::Copy {
                    src: result,
                    dst: dst.clone(),
                });

                dst
            }
            ast::Expression::Conditional {
                condition,
                exp1,
                exp2,
            } => {
                // The expression <condition> ? <e1> : <e2> will produce:
                //
                // <instructions_for_condition>
                // c = <result_of_condition>
                // JumpIfZero(c, e2_label)
                // <instructions_to-calculate_e1>
                // v1 = <result_of_e1>
                // result = v1
                // Jump(end)
                // Label(e2_label)
                // <instructions_to-calculate_e2>
                // v2 = <result_of_e2>
                // result = v2
                // Label(end)
                let result_of_condition = self.parse_val(*condition);
                let e2_label = self.make_label("exp2");
                let end_label = self.make_label("end");
                let result_label = self.make_label("result");
                self.instructions.push(Instruction::JumpIfZero {
                    condition: result_of_condition,
                    target: (&e2_label).into(),
                });
                let result_of_e1 = self.parse_val(*exp1);
                self.instructions.push(Instruction::Copy {
                    src: result_of_e1,
                    dst: Val::Var((&result_label).into()),
                });
                self.instructions.push(Instruction::Jump {
                    target: (&end_label).into(),
                });
                self.instructions.push(Instruction::Label(e2_label.into()));
                let result_of_e2 = self.parse_val(*exp2);
                self.instructions.push(Instruction::Copy {
                    src: result_of_e2,
                    dst: Val::Var((&result_label).into()),
                });
                self.instructions
                    .push(Instruction::Label((&end_label).into()));

                Val::Var((&result_label).into())
            }
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

    fn make_label(&mut self, prefix: &str) -> String {
        self.label_count += 1;
        match prefix {
            "and" => format!("and_false.{}", self.label_count),
            "or" => format!("or_false.{}", self.label_count),
            _ => format!("{prefix}{}", self.label_count),
        }
    }
}
