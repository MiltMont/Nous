use std::fmt::Debug;

use crate::ast;

/// A three address code program representation.
#[derive(Debug)]
pub struct Program(pub Function);

#[derive(Clone)]
pub struct Function {
    pub identifier: ast::Identifier,
    pub body: Vec<Instruction>,
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

#[derive(Clone)]
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
}

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
            } => write!(
                f,
                "\n\t\tBinary({:?}, {:?}, {:?}, {:?})",
                binary_operator, src_1, src_2, dst
            ),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Val {
    Constant(i64),
    Var(ast::Identifier),
}

/// Constructs TAC intermediate representation from an ast
///
/// ```
/// let mut lexer = Token::Lexer(&file);
/// let mut parser : Parser = Parser::build(&mut lexer);
/// let mut tac: TAC = TAC::build(parser.to_ast_program());
///
/// // Creating a tac program
/// let tac_program tac.to_tac_program();
/// ```
#[derive(Debug, Clone)]
pub struct TAC {
    source: ast::Program,
    temp_count: usize,
    instructions: Vec<Instruction>,
}

impl TAC {
    pub fn build(source: ast::Program) -> Self {
        Self {
            source,
            temp_count: 0,
            instructions: Vec::new(),
        }
    }

    pub fn to_tac_program(&mut self) -> Program {
        self.parse_program()
    }

    fn parse_program(&mut self) -> Program {
        let function = self.parse_function(self.source.0.clone());

        Program(function)
    }

    fn parse_function(&mut self, function: ast::Function) -> Function {
        self.instructions = Vec::new();
        let ret = self.parse_statement(function.body);

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
            ast::Expression::Binary(op, e1, e2) => {
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
        }
    }

    fn make_temporary_name(&mut self) -> String {
        self.temp_count += 1;
        format!("tmp.{}", self.temp_count)
    }
}
