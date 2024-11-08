use nous::{
    ast::{BinaryOperator, BlockItem, Expression, Function, Identifier, Program, UnaryOperator},
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
        name: Identifier(String::from("main")),
        body: vec![BlockItem::S(nous::ast::Statement::Return(
            expected_expression,
        ))],
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
        name: nous::ast::Identifier(String::from("main")),
        // body: nous::ast::Statement::Return(expected_expression),
        body: vec![BlockItem::S(nous::ast::Statement::Return(
            expected_expression,
        ))],
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
        name: nous::ast::Identifier(String::from("main")),
        // body: nous::ast::Statement::Return(expected_expression),
        body: vec![BlockItem::S(nous::ast::Statement::Return(
            expected_expression,
        ))],
    });

    assert_eq!(parser.to_ast_program().unwrap(), expected_program)
}

#[test]
fn test_expression() {
    let mut parser = parser_from_path("playground/test_expression.c");

    let expected_body = vec![
        BlockItem::D(nous::ast::Declaration {
            name: Identifier("x".into()),
            initializer: Some(Expression::Constant(3)),
        }),
        BlockItem::S(nous::ast::Statement::Return(Expression::Var(Identifier(
            "x".into(),
        )))),
    ];

    let expected_program = Program(Function {
        name: Identifier("main".into()),
        body: expected_body,
    });

    assert_eq!(parser.to_ast_program().unwrap(), expected_program)
}

#[test]
fn test_no_expression() {
    let mut parser = parser_from_path("playground/test_expression2.c");

    let expected_body = vec![BlockItem::D(nous::ast::Declaration {
        name: Identifier("y".into()),
        initializer: None,
    })];

    let expected_program = Program(Function {
        name: Identifier("main".into()),
        body: expected_body,
    });

    assert_eq!(parser.to_ast_program().unwrap(), expected_program);
}
