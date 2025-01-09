use std::fmt::Debug;

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
