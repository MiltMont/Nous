use nous::{
    ast::{
        BinaryOperator, Block, BlockItem, Declaration, Expression, FunctionDeclaration, Program,
        Statement, UnaryOperator, VariableDeclaration,
    },
    utils::parser_from_path,
};
// Testing unary operators
#[test]
fn test_unary() {
    let mut parser = parser_from_path("playground/test_unary.c");
    let expected_expression = Expression::Unary(
        UnaryOperator::Negate,
        Box::new(Expression::Unary(
            UnaryOperator::Negate,
            Box::new(Expression::Constant(2)),
        )),
    );

    let expected_program = Program(vec![FunctionDeclaration {
        name: "main".into(),
        parameters: vec![],
        body: Some(Block(vec![BlockItem::S(Statement::Return(
            expected_expression,
        ))])),
    }]);

    assert_eq!(parser.to_ast_program().unwrap(), expected_program)
}

// Testing binary operators
#[test]
fn test_same_precedence() {
    let mut parser = parser_from_path("tests/files/nested_binaryop.c");
    let expected_expression = Expression::Binary(
        BinaryOperator::Subtract,
        Box::new(Expression::Binary(
            BinaryOperator::Add,
            Box::new(Expression::Binary(
                BinaryOperator::Subtract,
                Box::new(Expression::Constant(4)),
                Box::new(Expression::Constant(2)),
            )),
            Box::new(Expression::Constant(2)),
        )),
        Box::new(Expression::Constant(3)),
    );

    let expected_program = Program(vec![FunctionDeclaration {
        name: "main".into(),
        parameters: vec![],
        body: Some(Block(vec![BlockItem::S(Statement::Return(
            expected_expression,
        ))])),
    }]);

    assert_eq!(parser.to_ast_program().unwrap(), expected_program);
}

#[test]
fn test_different_precedences() {
    let mut parser = parser_from_path("tests/files/binary_precedences.c");

    let expected_expression = Expression::Binary(
        BinaryOperator::Subtract,
        Box::new(Expression::Binary(
            BinaryOperator::Add,
            Box::new(Expression::Binary(
                BinaryOperator::Multiply,
                Box::new(Expression::Constant(4)),
                Box::new(Expression::Constant(2)),
            )),
            Box::new(Expression::Constant(2)),
        )),
        Box::new(Expression::Constant(1)),
    );

    let expected_program = Program(vec![FunctionDeclaration {
        name: "main".into(),
        parameters: vec![],
        body: Some(Block(vec![BlockItem::S(Statement::Return(
            expected_expression,
        ))])),
    }]);
    assert_eq!(parser.to_ast_program().unwrap(), expected_program)
}

#[test]
fn test_expression() {
    let mut parser = parser_from_path("playground/test_expression.c");

    //let expected_body = vec![
    //    BlockItem::D(nous::ast::Declaration {
    //        name: "x".into(),
    //        initializer: Some(Expression::Constant(3)),
    //    }),
    //    BlockItem::S(nous::ast::Statement::Return(Expression::Var("x".into()))),
    //];

    let expected_body = vec![
        BlockItem::D(Declaration::VarDecl(VariableDeclaration {
            name: "x".into(),
            initializer: Some(Expression::Constant(3)),
        })),
        BlockItem::S(Statement::Return(Expression::Var("x".into()))),
    ];

    let expected_program = Program(vec![FunctionDeclaration {
        name: "main".into(),
        parameters: vec![],
        body: Some(Block(expected_body)),
    }]);

    assert_eq!(parser.to_ast_program().unwrap(), expected_program)
}

#[test]
fn test_no_expression() {
    let mut parser = parser_from_path("playground/test_expression2.c");

    let expected_body = vec![BlockItem::D(Declaration::VarDecl(VariableDeclaration {
        name: "y".into(),
        initializer: None,
    }))];

    let expected_program = Program(vec![FunctionDeclaration {
        name: "main".into(),
        parameters: vec![],
        body: Some(Block(expected_body)),
    }]);
    assert_eq!(parser.to_ast_program().unwrap(), expected_program);
}

#[test]
fn test_mixed_expression() {
    let mut parser = parser_from_path("playground/test_expression3.c");

    let expected_body = vec![
        BlockItem::D(Declaration::VarDecl(VariableDeclaration {
            name: "x".into(),
            initializer: None,
        })),
        BlockItem::D(Declaration::VarDecl(VariableDeclaration {
            name: "y".into(),
            initializer: Some(Expression::Constant(3)),
        })),
        BlockItem::S(Statement::Return(Expression::Var("y".into()))),
    ];

    let expected_program = Program(vec![FunctionDeclaration {
        name: "main".into(),
        parameters: vec![],
        body: Some(Block(expected_body)),
    }]);

    assert_eq!(parser.to_ast_program().unwrap(), expected_program);
}

#[test]
fn test_expr_dec() {
    let mut parser = parser_from_path("playground/test_expression5.c");

    let expected_body = vec![
        BlockItem::D(Declaration::VarDecl(VariableDeclaration {
            name: "temp".into(),
            initializer: Some(Expression::Constant(10)),
        })),
        BlockItem::D(Declaration::VarDecl(VariableDeclaration {
            name: "x".into(),
            initializer: Some(Expression::Constant(10)),
        })),
        BlockItem::S(Statement::Expression(Expression::Assignment(
            Box::new(Expression::Var("temp".into())),
            Box::new(Expression::Binary(
                BinaryOperator::Subtract,
                Box::new(Expression::Var("temp".into())),
                Box::new(Expression::Var("x".into())),
            )),
        ))),
        BlockItem::S(Statement::Return(Expression::Var("temp".into()))),
    ];

    let expected_program = Program(vec![FunctionDeclaration {
        name: "main".into(),
        parameters: vec![],
        body: Some(Block(expected_body)),
    }]);

    assert_eq!(parser.to_ast_program().unwrap(), expected_program);
}

#[test]
fn test_while() {
    let mut parser = parser_from_path("playground/test_while2.c");

    let nested_while = Statement::While {
        condition: Expression::Binary(
            BinaryOperator::LessThan,
            Box::new(Expression::Var("b".into())),
            Box::new(Expression::Constant(3)),
        ),
        body: Box::new(Statement::Compound(Block(vec![BlockItem::S(
            Statement::Continue { label: None },
        )]))),
        identifier: None,
    };

    let nested_if = Statement::If {
        condition: Expression::Binary(
            BinaryOperator::Equal,
            Box::new(Expression::Var("a".into())),
            Box::new(Expression::Constant(2)),
        ),
        then: Box::new(Statement::Compound(Block(vec![BlockItem::S(
            Statement::Break { label: None },
        )]))),
        else_statement: None,
    };

    let parent_while = Statement::While {
        condition: Expression::Binary(
            BinaryOperator::LessThan,
            Box::new(Expression::Var("a".into())),
            Box::new(Expression::Constant(3)),
        ),
        body: Box::new(Statement::Compound(Block(vec![
            BlockItem::S(nested_while),
            BlockItem::S(nested_if),
        ]))),
        identifier: None,
    };

    let expected_program = Program(vec![FunctionDeclaration {
        name: "main".into(),
        parameters: vec![],
        body: Some(Block(vec![
            BlockItem::D(Declaration::VarDecl(VariableDeclaration {
                name: "a".into(),
                initializer: Some(Expression::Constant(1)),
            })),
            BlockItem::D(Declaration::VarDecl(VariableDeclaration {
                name: "b".into(),
                initializer: Some(Expression::Constant(2)),
            })),
            BlockItem::S(parent_while),
        ])),
    }]);

    assert_eq!(parser.to_ast_program().unwrap(), expected_program);
}

#[test]
fn test_declaration_no_body() {
    let mut parser = parser_from_path("playground/test_function_declaration_no_body.c");

    let func1 = FunctionDeclaration {
        name: "test1".into(),
        parameters: vec!["a".into()],
        body: None,
    };

    let func2 = FunctionDeclaration {
        name: "test2".into(),
        parameters: vec!["a".into(), "b".into()],
        body: None,
    };

    let expected_program = Program(vec![func1, func2]);

    assert_eq!(parser.to_ast_program().unwrap(), expected_program);
}
