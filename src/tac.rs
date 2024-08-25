use crate::{assembly::AssemblyProgram, ast::{Expression, Identifier, UnaryOperator}};

#[derive(Debug, Clone)]
pub struct TacProgram(pub TacFunction); 

#[derive(Debug, Clone)]
pub struct TacFunction {
    identifier: Identifier, 
    body: Vec<Instruction>, 
}

#[derive(Debug, Clone)]
pub enum Instruction {
    Return(Val), 
    Unary {
        operator: UnaryOperator, 
        src: Val, 
        dst: Val, 
    }
}

#[derive(Debug, Clone)]
pub enum Val {
    Constant(i64), 
    Var(Identifier)
}

#[derive(Debug, Clone)]
pub struct TacGenerator {
    pub program: TacProgram, 
    temp_count: i64,
    instructions: Vec<Instruction>
}

impl TacGenerator {
    fn build(assembly_program: AssemblyProgram) -> Self {
        todo!() 
    }

    fn emit_instructions(&mut self, expression: Expression) -> Val {

        match expression {
            Expression::Constant(i) => 
                Val::Constant(i)
            ,
            Expression::Unary(op, inner) => {
                let src = self.emit_instructions(*inner); 
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