#[cfg(test)]
mod tests {
    use nous::{
        ast::{BinaryOperator, Expression, Program},
        utils::parser_from_file,
    };

    // Testing binary operators
    #[test]
    fn test_same_precedence() {
        let mut parser = parser_from_file("tests/files/nested_binaryop.c");
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
            body: nous::ast::Statement::Return(expected_expression),
        });

        assert_eq!(parser.to_ast_program(), expected_program);
    }

    #[test]
    fn test_different_precedences() {
        let mut parser = parser_from_file("tests/files/binary_precedences.c");

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
            body: nous::ast::Statement::Return(expected_expression),
        });

        assert_eq!(parser.to_ast_program(), expected_program)
    }
}
