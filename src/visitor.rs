use std::{collections::HashMap, fmt::Debug};

use crate::{
    assembly::Assembly,
    assembly_passes::{
        AllocateStack, ReplacePseudoRegisters, RewriteBinaryOp, RewriteCmp, RewriteMov,
    },
    ast::Program,
    identifier_resolution::IdentifierResolution,
};

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

pub fn visit_collection<T, V>(vec: &mut Vec<T>, mut visitor: V)
where
    T: Debug,
    V: Visitor<Vec<T>> + Debug,
{
    visitor.visit(vec);
    eprint!("\n{visitor:#?}\n\n\t{:?}\n", &vec);
}

pub fn visit_collection_with_context<T, V, C>(vec: &mut Vec<T>, mut visitor: V, context: &mut C)
where
    T: Debug,
    V: VisitorWithContext<Vec<T>, C> + Debug,
{
    visitor.visit(vec, context);
    eprint!("\n{visitor:?}\n");
    eprint!("\n{vec:?}\n\n");
}

pub fn assembly_passes(assembly: &mut Assembly) {
    apply_visitor_with_context(
        &mut assembly.program.as_mut().unwrap().0.instructions,
        ReplacePseudoRegisters,
        &mut assembly.pseudo_registers,
    );
    visit_collection(
        &mut assembly.program.as_mut().unwrap().0.instructions,
        RewriteMov,
    );
    visit_collection(
        &mut assembly.program.as_mut().unwrap().0.instructions,
        RewriteBinaryOp,
    );
    visit_collection(
        &mut assembly.program.as_mut().unwrap().0.instructions,
        RewriteCmp,
    );
    visit_collection_with_context(
        &mut assembly.program.as_mut().unwrap().0.instructions,
        AllocateStack,
        &mut assembly.offset.clone(),
    );
}

#[allow(unused_variables)]
pub fn validation_passes(program: &mut Program) {
    apply_visitor_with_context(
        &mut program.0,
        IdentifierResolution::default(),
        &mut HashMap::new(),
    );
    //apply_visitor_with_context(&mut program.0, LoopLabeling::default(), &mut None);
}
