use crate::{assembly::{Program, Instruction}};

struct AssemblyPass {
    program: Program,
    instructions: Vec<Instruction>
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
    pub fn build(assembly_program: Program) -> Self {
        // Takes ownership of the assembly program and clones
        // its instruction set. 
        let instructions: Vec<Instruction> = assembly_program.0.instructions.clone();
        Self {
            program: assembly_program, 
            instructions: instructions, 
        }
    }

    // TODO: Implement 
    //
    // replace_pseudo_registers()
    // rewrite_mov()
    // rewrite_binop()
    // allocate_stack()
    // 

    pub fn replace_pseudo_registers(&mut self) -> &mut Self {
        todo!()
    }

    pub fn rewrite_mov(&mut self) -> &mut Self {
        todo!()
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