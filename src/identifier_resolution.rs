use std::collections::HashMap;

use crate::{
    ast::{
        Block, BlockItem, Declaration, Expression, ForInit, FunctionDeclaration, Identifier,
        Statement, VariableDeclaration,
    },
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
pub struct IdentifierResolution {
    pub offset: usize,
}

#[allow(dead_code)]
impl IdentifierResolution {
    // TODO: Check if this assignement doesnt conflict with other assignements.
    fn make_temporary_name(&mut self, name: &str) -> String {
        self.offset += 1;
        format!("{}.{}", name, self.offset)
    }

    fn copy_identifier_map(&mut self, identifier_map: &IdentifierMap) -> IdentifierMap {
        identifier_map
            .iter()
            .map(|(identifier, info)| {
                (
                    identifier.clone(),
                    IdentifierInfo {
                        name: info.name.clone(),
                        from_current_scope: false,
                        // FIX: Should this be false?
                        has_linkage: info.has_linkage,
                    },
                )
            })
            .collect()
    }

    fn identifier_helper_function(
        &mut self,
        identifier: Identifier,
        identifier_map: &mut IdentifierMap,
        has_linkage: bool,
    ) -> Identifier {
        if let Some(decl_info) = identifier_map.get(&identifier) {
            if decl_info.from_current_scope {
                panic!("Duplicate variable declaration")
            }
        }

        let unique_name = self.make_temporary_name(&identifier.0);

        identifier_map.insert(
            identifier.clone(),
            IdentifierInfo {
                name: unique_name.clone(),
                from_current_scope: true,
                // TODO: What shold this be?
                has_linkage,
            },
        );

        unique_name.into()
    }
}

#[derive(Debug)]
pub struct IdentifierInfo {
    name: String,
    pub from_current_scope: bool,
    has_linkage: bool,
}

type IdentifierMap = HashMap<Identifier, IdentifierInfo>;

impl VisitorWithContext<Block, IdentifierMap> for IdentifierResolution {
    fn visit(&mut self, block: &mut Block, context: &mut IdentifierMap) {
        for item in block.0.iter_mut() {
            match item {
                crate::ast::BlockItem::S(statement) => self.visit(statement, context),
                crate::ast::BlockItem::D(declaration) => self.visit(declaration, context),
            }
        }
    }
}

impl VisitorWithContext<Statement, IdentifierMap> for IdentifierResolution {
    fn visit(&mut self, item: &mut Statement, context: &mut IdentifierMap) {
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
                let mut new_variable_map = self.copy_identifier_map(context);

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
                let mut new_variable_map = self.copy_identifier_map(context);
                self.visit(block, &mut new_variable_map);
            }
            _ => {}
        }
    }
}

impl VisitorWithContext<Declaration, IdentifierMap> for IdentifierResolution {
    fn visit(&mut self, declaration: &mut Declaration, identifier_map: &mut IdentifierMap) {
        match declaration {
            Declaration::FuncDecl(function_declaration) => {
                self.visit(function_declaration, identifier_map)
            }
            Declaration::VarDecl(variable_declaration) => {
                self.visit(variable_declaration, identifier_map)
            }
        }
    }
}

impl VisitorWithContext<VariableDeclaration, IdentifierMap> for IdentifierResolution {
    fn visit(&mut self, declaration: &mut VariableDeclaration, identifier_map: &mut IdentifierMap) {
        // Local variable declarations have no linkage.
        let unique_name =
            self.identifier_helper_function(declaration.name.clone(), identifier_map, false);
        self.visit(&mut declaration.initializer, identifier_map);
        declaration.name = unique_name;
    }
}

impl VisitorWithContext<Expression, IdentifierMap> for IdentifierResolution {
    fn visit(&mut self, expression: &mut Expression, identifier_map: &mut IdentifierMap) {
        match expression {
            Expression::Constant(_) => {}
            Expression::Var(identifier) => {
                if let Some(variable_info) = identifier_map.get(identifier) {
                    *identifier = variable_info.name.clone().into();
                } else {
                    panic!("Undeclared variable");
                }
            }
            Expression::Unary(_unary_operator, expression) => {
                self.visit(&mut **expression, identifier_map);
            }
            Expression::Binary(_binary_operator, expression, expression1) => {
                self.visit(&mut **expression, identifier_map);
                self.visit(&mut **expression1, identifier_map);
            }
            Expression::Assignment(expression, expression1) => {
                if !matches!(**expression, Expression::Var(_)) {
                    panic!("Invalid LValue");
                }
                self.visit(&mut **expression, identifier_map);
                self.visit(&mut **expression1, identifier_map);
            }
            Expression::Conditional {
                condition,
                exp1,
                exp2,
            } => {
                self.visit(&mut **condition, identifier_map);
                self.visit(&mut **exp1, identifier_map);
                self.visit(&mut **exp2, identifier_map);
            }
            Expression::FunctionCall { name, arguments } => {
                if let Some(function_name) = identifier_map.get(name) {
                    *name = function_name.name.clone().into();

                    arguments
                        .iter_mut()
                        .for_each(|argument| self.visit(argument, identifier_map));
                } else {
                    panic!("Undeclared function!")
                }
            }
        }
    }
}

impl VisitorWithContext<Option<Expression>, IdentifierMap> for IdentifierResolution {
    fn visit(&mut self, item: &mut Option<Expression>, _context: &mut IdentifierMap) {
        if item.is_some() {
            self.visit(item.as_mut().unwrap(), _context);
        }
    }
}

impl VisitorWithContext<ForInit, IdentifierMap> for IdentifierResolution {
    fn visit(&mut self, item: &mut ForInit, context: &mut IdentifierMap) {
        match item {
            ForInit::InitDecl(_declaration) => todo!(),
            ForInit::InitExp(expression) => self.visit(expression, context),
        }
    }
}

impl VisitorWithContext<BlockItem, IdentifierMap> for IdentifierResolution {
    fn visit(&mut self, item: &mut BlockItem, context: &mut IdentifierMap) {
        match item {
            BlockItem::S(statement) => self.visit(statement, context),
            BlockItem::D(declaration) => self.visit(declaration, context),
        }
    }
}

impl VisitorWithContext<FunctionDeclaration, IdentifierMap> for IdentifierResolution {
    fn visit(&mut self, declaration: &mut FunctionDeclaration, identifier_map: &mut IdentifierMap) {
        if let Some(prev_entry) = identifier_map.get(&declaration.name) {
            if prev_entry.from_current_scope && !(prev_entry.has_linkage) {
                panic!("Duplicate declaration")
            }
        }

        identifier_map.insert(
            declaration.name.clone(),
            IdentifierInfo {
                name: declaration.name.0.clone(),
                from_current_scope: true,
                has_linkage: true,
            },
        );

        let mut inner_map = self.copy_identifier_map(identifier_map);

        declaration
            .parameters
            .iter_mut()
            .for_each(|parameter| self.visit(parameter, &mut inner_map));

        if let Some(decl_body) = declaration.body.as_mut() {
            self.visit(decl_body, &mut inner_map);
        }
    }
}

impl VisitorWithContext<Identifier, IdentifierMap> for IdentifierResolution {
    fn visit(&mut self, identifier: &mut Identifier, identifier_map: &mut IdentifierMap) {
        // TODO: Should this have linkage?
        *identifier = self.identifier_helper_function(identifier.clone(), identifier_map, true);
    }
}
