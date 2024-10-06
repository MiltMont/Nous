#[cfg(test)]
pub mod tests {
    use logos::Logos;
    use nous::{
        ast::{Expression, Function, Identifier, Program, Statement},
        lexer::Token,
        parser::Parser,
        utils::read_file,
    };

    #[test]
    fn test_return_2() -> std::io::Result<()> {
        let source = read_file("tests/files/valid/return_2.c")?;

        let mut lexer = Token::lexer(&source);
        let mut parser = Parser::build(&mut lexer);

        let test = Program(Function {
            name: Identifier("main".to_owned()),
            body: Statement::Return(Expression::Constant(2)),
        });

        let program = parser.to_ast_program();

        assert_eq!(program, test);

        Ok(())
    }
}
