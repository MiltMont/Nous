use crate::{
    ast::{self, Identifier, Statement},
    visitor::VisitorWithContext,
};

#[derive(Default)]
pub struct LoopLabeling {
    pub current_label: Option<Identifier>,
    offset: i32,
}

impl LoopLabeling {
    pub fn annotate(&self) -> ast::Identifier {
        if let Some(label) = &self.current_label {
            label.clone()
        } else {
            panic!("No current label")
        }
    }

    pub fn make_label(&mut self) -> ast::Identifier {
        self.offset += 1;
        format!("label.{}", self.offset).into()
    }
}

impl VisitorWithContext<ast::Statement, Option<Identifier>> for LoopLabeling {
    fn visit(&mut self, item: &mut ast::Statement, current_label: &mut Option<Identifier>) {
        match item {
            Statement::Break { label } => {
                if current_label.is_some() {
                    *label = current_label.clone();
                } else {
                    panic!("Break statement outside of a loop")
                }
            }
            Statement::Continue { label } => {
                if current_label.is_some() {
                    *label = current_label.clone();
                } else {
                    panic!("NOOO")
                }
            }
            Statement::While {
                condition: _,
                body,
                identifier,
            } => {
                // Modify current label to be a generated label.
                let current_label = self.make_label();
                // Modify while identifier.
                *identifier = Some(current_label.clone());
                // Visit the body.
                self.visit(&mut **body, &mut Some(current_label));
            }
            Statement::DoWhile {
                body,
                condition: _,
                identifier,
            } => {
                // Create a new label
                let current_label = self.make_label();
                // Modify identifier
                *identifier = Some(current_label.clone());
                // Visit bodyh
                self.visit(&mut **body, &mut Some(current_label));
            }
            Statement::For {
                initializer: _,
                condition: _,
                post: _,
                body,
                identifier,
            } => {
                // Create a new label
                let current_label = self.make_label();
                // Modify identifier
                *identifier = Some(current_label.clone());
                // Visit body
                self.visit(&mut **body, &mut Some(current_label));
            }
            Statement::If {
                condition: _,
                then,
                else_statement,
            } => {
                self.visit(&mut **then, current_label);
                if let Some(else_stm) = else_statement {
                    self.visit(&mut **else_stm, current_label);
                }
            }
            Statement::Compound(block) => {
                for item in block.0.iter_mut() {
                    self.visit(item, current_label);
                }
            }
            _ => {}
        }
    }
}

impl VisitorWithContext<ast::BlockItem, Option<Identifier>> for LoopLabeling {
    fn visit(&mut self, item: &mut ast::BlockItem, ident: &mut Option<Identifier>) {
        match item {
            ast::BlockItem::S(statement) => self.visit(statement, ident),
            ast::BlockItem::D(_) => {}
        }
    }
}
