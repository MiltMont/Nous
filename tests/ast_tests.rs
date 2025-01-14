use nous::{
    ast::{Block, BlockItem, Expression, FunctionDeclaration, Program, Statement},
    utils::parser_from_path,
};

#[test]
fn test_return_2() {
    let mut parser = parser_from_path("playground/return_2.c");

    let expected_expression = Expression::Constant(2);

    let expected_program = Program(vec![FunctionDeclaration {
        name: "main".into(),
        parameters: vec![],
        body: Some(Block(vec![BlockItem::S(Statement::Return(
            expected_expression,
        ))])),
    }]);

    assert_eq!(parser.to_ast_program().unwrap(), expected_program)
}
