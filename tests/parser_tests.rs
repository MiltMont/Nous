use nous::{
    ast::{
        BinaryOperator, Block, BlockItem, Declaration, Expression, Function, Identifier, Program,
        Statement, UnaryOperator,
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

    let expected_program = Program(Function {
        name: "main".into(),
        body: nous::ast::Block(vec![BlockItem::S(nous::ast::Statement::Return(
            expected_expression,
        ))]),
    });

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

    let expected_program = Program(nous::ast::Function {
        name: "main".into(),
        // body: nous::ast::Statement::Return(expected_expression),
        body: nous::ast::Block(vec![BlockItem::S(nous::ast::Statement::Return(
            expected_expression,
        ))]),
    });

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

    let expected_program = Program(nous::ast::Function {
        name: "main".into(),
        body: nous::ast::Block(vec![BlockItem::S(nous::ast::Statement::Return(
            expected_expression,
        ))]),
    });

    assert_eq!(parser.to_ast_program().unwrap(), expected_program)
}

#[test]
fn test_expression() {
    let mut parser = parser_from_path("playground/test_expression.c");

    let expected_body = vec![
        BlockItem::D(nous::ast::Declaration {
            name: "x".into(),
            initializer: Some(Expression::Constant(3)),
        }),
        BlockItem::S(nous::ast::Statement::Return(Expression::Var("x".into()))),
    ];

    let expected_program = Program(Function {
        name: "main".into(),
        body: nous::ast::Block(expected_body),
    });

    assert_eq!(parser.to_ast_program().unwrap(), expected_program)
}

#[test]
fn test_no_expression() {
    let mut parser = parser_from_path("playground/test_expression2.c");

    let expected_body = vec![BlockItem::D(nous::ast::Declaration {
        name: "y".into(),
        initializer: None,
    })];

    let expected_program = Program(Function {
        name: "main".into(),
        body: Block(expected_body),
    });

    assert_eq!(parser.to_ast_program().unwrap(), expected_program);
}

#[test]
fn test_mixed_expression() {
    let mut parser = parser_from_path("playground/test_expression3.c");

    let expected_body = vec![
        BlockItem::D(nous::ast::Declaration {
            name: "x".into(),
            initializer: None,
        }),
        BlockItem::D(nous::ast::Declaration {
            name: "y".into(),
            initializer: Some(Expression::Constant(3)),
        }),
        BlockItem::S(nous::ast::Statement::Return(Expression::Var("y".into()))),
    ];

    let expected_program = Program(Function {
        name: Identifier("main".into()),
        body: Block(expected_body),
    });

    assert_eq!(parser.to_ast_program().unwrap(), expected_program);
}

#[test]
fn test_expr_dec() {
    let mut parser = parser_from_path("playground/test_expression5.c");

    let exptected_body = vec![
        BlockItem::D(Declaration {
            name: "temp".into(),
            initializer: Some(Expression::Constant(10)),
        }),
        BlockItem::D(Declaration {
            name: "x".into(),
            initializer: Some(Expression::Constant(10)),
        }),
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

    let exptected_program = Program(Function {
        name: "main".into(),
        body: Block(exptected_body),
    });

    assert_eq!(parser.to_ast_program().unwrap(), exptected_program);
}
