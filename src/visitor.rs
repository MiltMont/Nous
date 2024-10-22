use crate assembly::{Program, Instruction};

struct AssemblyPass {
    program: Program,
    instructions: Vec<Instruction>
}

impl AssemblyPass {
    /// Constructs a visitor from an 
    /// Assembly instance whenever the program 
    /// field is not None, otherwise it 
    /// returns an Error.  
    pub fn build(assembly: assembly::Program) -> Result<Self, String>{
        todo!()
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
    /// the original assembly value and returns
    /// the modified instance. 
    pub fn modify_assembly(self) -> Program {
        todo!()
    }
}