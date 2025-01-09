use std::collections::HashMap;

use crate::{
    ast::{Block, BlockItem, Declaration, Expression, ForInit, Identifier, Statement},
    visitor::VisitorWithContext,
};

/// This takes an ast program an performs variable
/// resolution on its block items.
///
/// This pass tracks which variables are in
/// scope throughout the program and resolves each reference to
/// a variable by finding the corresponding declaration.
///
/// It reports an error if a program declares the same variable
/// more than once or uses a variable that hasn't been delcared.
///
/// It renames each local variable with a globally unique
/// identifier.
#[derive(Default, Debug)]
pub struct VariableResolution {
    offset: usize,
}

impl VariableResolution {
    // TODO: Check if this assignement doesnt conflict with other assignements.
    fn make_temporary_name(&mut self, name: &str) -> String {
        self.offset += 1;
        format!("{}.{}", name, self.offset)
    }

    fn copy_variable_map(&mut self, variable_map: &VariableMap) -> VariableMap {
        variable_map
            .iter()
            .map(|(identifier, info)| {
                (
                    identifier.clone(),
                    VariableInfo {
                        name: info.name.clone(),
                        from_current_block: false,
                    },
                )
            })
            .collect()
    }
}

#[derive(Debug)]
pub struct VariableInfo {
    name: String,
    from_current_block: bool,
}

type VariableMap = HashMap<Identifier, VariableInfo>;

impl VisitorWithContext<Block, VariableMap> for VariableResolution {
    fn visit(&mut self, block: &mut Block, context: &mut VariableMap) {
        for item in block.0.iter_mut() {
            match item {
                crate::ast::BlockItem::S(statement) => self.visit(statement, context),
                crate::ast::BlockItem::D(declaration) => self.visit(declaration, context),
            }
        }
    }
}

impl VisitorWithContext<Statement, VariableMap> for VariableResolution {
    fn visit(&mut self, item: &mut Statement, context: &mut VariableMap) {
        match item {
            Statement::Return(expression) => self.visit(expression, context),
            Statement::Expression(expression) => self.visit(expression, context),
            Statement::If {
                condition,
                then,
                else_statement,
            } => {
                self.visit(condition, context);
                self.visit(&mut **then, context);
                if let Some(else_stm) = else_statement {
                    self.visit(&mut **else_stm, context);
                }
            }
            Statement::While {
                condition,
                body,
                identifier: _,
            } => {
                self.visit(condition, context);
                self.visit(&mut **body, context);
            }
            Statement::DoWhile {
                body,
                condition,
                identifier: _,
            } => {
                self.visit(condition, context);
                self.visit(&mut **body, context);
            }
            Statement::For {
                initializer,
                condition,
                post,
                body,
                identifier: _,
            } => {
                let mut new_variable_map = self.copy_variable_map(context);

                self.visit(initializer, &mut new_variable_map);
                if let Some(condition) = condition {
                    self.visit(condition, &mut new_variable_map);
                }

                if let Some(post) = post {
                    self.visit(post, &mut new_variable_map);
                }

                self.visit(&mut **body, &mut new_variable_map);
            }
            Statement::Compound(block) => {
                let mut new_variable_map = self.copy_variable_map(context);
                self.visit(block, &mut new_variable_map);
            }
            _ => {}
        }
    }
}

impl VisitorWithContext<Declaration, VariableMap> for VariableResolution {
    fn visit(&mut self, declaration: &mut Declaration, variable_map: &mut VariableMap) {
        if variable_map.contains_key(&declaration.name)
            && variable_map
                .get(&declaration.name)
                .unwrap()
                .from_current_block
        {
            // FIX: better error reporting.
            panic!("Duplicate variable declaration.")
        }

        let unique_name = self.make_temporary_name(&declaration.name.0);

        variable_map.insert(
            declaration.name.clone(),
            VariableInfo {
                name: unique_name.clone(),
                from_current_block: true,
            },
        );

        self.visit(&mut declaration.initializer, variable_map);
        declaration.name = unique_name.into();
    }
}

impl VisitorWithContext<Expression, VariableMap> for VariableResolution {
    fn visit(&mut self, expression: &mut Expression, variable_map: &mut VariableMap) {
        match expression {
            Expression::Constant(_) => {}
            Expression::Var(identifier) => {
                if let Some(variable_info) = variable_map.get(identifier) {
                    *identifier = variable_info.name.clone().into();
                } else {
                    panic!("Undeclared variable");
                }
            }
            Expression::Unary(_unary_operator, expression) => {
                self.visit(&mut **expression, variable_map);
            }
            Expression::Binary(_binary_operator, expression, expression1) => {
                self.visit(&mut **expression, variable_map);
                self.visit(&mut **expression1, variable_map);
            }
            Expression::Assignment(expression, expression1) => {
                if !matches!(**expression, Expression::Var(_)) {
                    panic!("Invalid LValue");
                }
                self.visit(&mut **expression, variable_map);
                self.visit(&mut **expression1, variable_map);
            }
            Expression::Conditional {
                condition,
                exp1,
                exp2,
            } => {
                self.visit(&mut **condition, variable_map);
                self.visit(&mut **exp1, variable_map);
                self.visit(&mut **exp2, variable_map);
            }
        }
    }
}

impl VisitorWithContext<Option<Expression>, VariableMap> for VariableResolution {
    fn visit(&mut self, item: &mut Option<Expression>, _context: &mut VariableMap) {
        if item.is_some() {
            self.visit(item.as_mut().unwrap(), _context);
        }
    }
}

impl VisitorWithContext<ForInit, VariableMap> for VariableResolution {
    fn visit(&mut self, item: &mut ForInit, context: &mut VariableMap) {
        match item {
            ForInit::InitDecl(declaration) => self.visit(declaration, context),
            ForInit::InitExp(expression) => self.visit(expression, context),
        }
    }
}

impl VisitorWithContext<BlockItem, VariableMap> for VariableResolution {
    fn visit(&mut self, item: &mut BlockItem, context: &mut VariableMap) {
        match item {
            BlockItem::S(statement) => self.visit(statement, context),
            BlockItem::D(declaration) => self.visit(declaration, context),
        }
    }
}
