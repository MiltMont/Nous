use std::fmt::Debug;

use crate::ast::{Expression, Function, Identifier, Program, Statement, UnaryOperator};

#[derive(Clone)]
pub struct TacProgram(pub TacFunction);

impl Debug for TacProgram {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TacProgram(\n{:?}\n)",&self.0) 
    }
}

#[derive(Clone)]
pub struct TacFunction {
    pub identifier: Identifier, 
    pub body: Vec<Instruction>, 
}

impl Debug for TacFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
       
        write!(f, "\tFunction(\n\tIdentifier: {:?} \n\tBody: {:?}\n\t)", &self.identifier, &self.body)
    }
}

#[derive(Clone)]
pub enum Instruction {
    Return(Val), 
    Unary {
        operator: UnaryOperator, 
        src: Val, 
        dst: Val, 
    }
}

impl Debug for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Return(arg0) => f.debug_tuple("\n\t\tReturn").field(arg0).finish(),
            Self::Unary { operator, src, dst } => 
            write!(f, "\n\t\tUnary({:?}, {:?}, {:?})", operator, src, dst), 
        }
    }
}

#[derive(Debug, Clone)]
pub enum Val {
    Constant(i64), 
    Var(Identifier)
}

#[derive(Debug, Clone)]
pub struct TacGenerator {
    source: Program, 
    temp_count: usize,
    instructions: Vec<Instruction>
}

impl TacGenerator {
    pub fn build(source: Program) -> Self {
        Self {
            source, 
            temp_count: 0, 
            instructions: Vec::new()
        }
    }

    pub fn parse_program(&mut self) -> TacProgram {
        let function = self.parse_function(self.source.0.clone()); 

        TacProgram(function)
    }

    fn parse_function(&mut self, function: Function) -> TacFunction {

        // Remove instructions resulting from previous use 
        // of parse_statement. 
        self.instructions = Vec::new(); 
        let ret = self.parse_statement(function.body); 

        self.instructions.push(ret); 

        TacFunction {
            identifier: function.name, 
            body: self.instructions.clone()
        }
    }

    fn parse_statement(&mut self, statement: Statement) -> Instruction {
        match statement {
            Statement::Return(expression) => {
                let val = self.parse_val(expression); 

                Instruction::Return(val)
            },
        }
    }

    fn parse_val(&mut self, expression: Expression) -> Val {

        match expression {
            Expression::Constant(i) => 
                Val::Constant(i)
            ,
            Expression::Unary(op, inner) => {
                let src = self.parse_val(*inner); 
                let dst_name = self.make_temporary_name();
                let dst = Val::Var(Identifier(dst_name)); 
                self.instructions.push(
                    Instruction::Unary { operator: op, src, dst: dst.clone() }
                );
                dst
            },
        }
    }

    fn make_temporary_name(&mut self) -> String {
        self.temp_count += 1; 
        format!("tmp.{}", self.temp_count)
        
    }
}