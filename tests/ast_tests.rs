#[cfg(test)]
pub mod tests {
    use logos::Logos;
    use nous::{ast::{parse_program, Expression, Function, Identifier, Program, Statement}, lexer::Token, utils::read_file};


    #[test]
    fn test_return_2() -> std::io::Result<()>{
        let source = read_file("tests/files/valid/return_2.c")?;

        let mut lexer = Token::lexer(&source);

        let test = Program(
            Function {
                name: Identifier("main".to_owned()), 
                body: Statement::Return(Expression::Constant(2))
            }
        );

        if let Ok(p) = parse_program(&mut lexer) {

            assert_eq!(p, test); 
        }

        Ok(())
    }
}