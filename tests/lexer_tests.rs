#[cfg(test)]
mod tests {
    use std::iter::zip;
    use std::{fs::File, io::BufReader};
    use std::io::prelude::*;

    use logos::{Lexer, Logos};
    use nous::lexer::Token;

    /// Valid programs.
    #[test]
    fn test_tabs() -> std::io::Result<()> {
        let contents = read_file("tests/lexer_files/valid/newlines.c")?; 

        let lexer = Token::lexer(&contents);

        let tokens: Vec<Token> = vec![
            Token::Int, 
            Token::Identifier(String::from("main")), 
            Token::LParen, 
            Token::Void, 
            Token::RParen, 
            Token::LBrace, 
            Token::Return, 
            Token::Constant(0), 
            Token::Semicolon,
            Token::RBrace, 
        ];


        compare_tokens(lexer, tokens);
        Ok(())
    }

    #[test]
    fn test_newlines() -> std::io::Result<()> {
        let contents = read_file("tests/lexer_files/valid/newlines.c")?;

        let lexer = Token::lexer(&contents); 

        let tokens: Vec<Token> = vec![
            Token::Int, 
            Token::Identifier(String::from("main")), 
            Token::LParen, 
            Token::Void, 
            Token::RParen, 
            Token::LBrace, 
            Token::Return, 
            Token::Constant(0), 
            Token::Semicolon,
            Token::RBrace, 
        ];

        compare_tokens(lexer, tokens);

        Ok(())
    }

    #[test]
    fn test_no_newlines() -> std::io::Result<()> {
        let contents = read_file("tests/lexer_files/valid/no_newlines.c")?;

        let lexer = Token::lexer(&contents); 

        let tokens: Vec<Token> = vec![
            Token::Int, 
            Token::Identifier(String::from("main")), 
            Token::LParen, 
            Token::Void, 
            Token::RParen, 
            Token::LBrace, 
            Token::Return, 
            Token::Constant(0), 
            Token::Semicolon,
            Token::RBrace, 
        ];

        compare_tokens(lexer, tokens);

        Ok(())

    }

    #[test]
    fn test_multidigit() -> std::io::Result<()> {
        let contents = read_file("tests/lexer_files/valid/multi_digit.c")?; 

        let lexer = Token::lexer(&contents); 

        let tokens = vec![
            Token::Int, 
            Token::Identifier(String::from("main")), 
            Token::LParen, 
            Token::Void, 
            Token::RParen, 
            Token::LBrace, 
            Token::Return, 
            Token::Constant(100), 
            Token::Semicolon,
            Token::RBrace, 
        ];

        compare_tokens(lexer, tokens); 
        Ok(())
    }

    #[test]
    fn test_return_2() -> std::io::Result<()> {

        let contents = read_file("tests/lexer_files/valid/return_2.c")?; 

        let lexer = Token::lexer(&contents); 

        let tokens = vec![
            Token::Int, 
            Token::Identifier(String::from("main")), 
            Token::LParen, 
            Token::Void, 
            Token::RParen, 
            Token::LBrace, 
            Token::Return, 
            Token::Constant(2), 
            Token::Semicolon,
            Token::RBrace, 
        ];

        compare_tokens(lexer, tokens);

        Ok(())
 
    }

    #[test]
    fn test_spaces() -> std::io::Result<()> {
        let contents = read_file("tests/lexer_files/valid/spaces.c")?; 

        let lexer = Token::lexer(&contents); 

        let tokens = vec![
            Token::Int, 
            Token::Identifier(String::from("main")), 
            Token::LParen, 
            Token::Void, 
            Token::RParen, 
            Token::LBrace, 
            Token::Return, 
            Token::Constant(0), 
            Token::Semicolon,
            Token::RBrace, 
        ];

        compare_tokens(lexer, tokens);

        Ok(()) 
    }

    /// Invalid programs
    #[test]
    #[should_panic(expected="Unexpected sign")]
    fn test_at_sign() {
        let source = read_file("tests/lexer_files/invalid/at_sign.c").unwrap();

        let lexer = Token::lexer(&source);

        for result in lexer {
            if let Err(_) = result {
                panic!("Unexpected sign");
            }
        }
    }

    #[test]
    #[should_panic(expected="Invalid Token")]
    fn test_backslash() {
        let source = read_file("tests/lexer_files/invalid/backslash.c").unwrap(); 

        let lexer = Token::lexer(&source); 

        for result in lexer {
            if let Err(_) = result {
                panic!("Invalid Token"); 
            }
        }
    }

    #[test]
    #[should_panic(expected="Invalid Token")]
    fn test_backtick() {
        let source = read_file("tests/lexer_files/invalid/backtick.c").unwrap(); 

        let lexer = Token::lexer(&source); 

        for result in lexer {
            if let Err(_) = result {
                panic!("Invalid Token"); 
            }
        } 
    }

    #[test]
    #[should_panic(expected="Invalid identifier")]
    fn test_invalid_identifier() {
        let source = read_file("tests/lexer_files/invalid/invalid_identifier.c").unwrap(); 

        let lexer = Token::lexer(&source); 

        for result in lexer {
            if let Err(_) = result {
                panic!("Invalid identifier"); 
            }
        } 
    }


    #[test]
    #[should_panic(expected="Invalid identifier")]
    fn test_invalid_identifier_2() {
        let source = read_file("tests/lexer_files/invalid/invalid_identifier_2.c").unwrap(); 

        let lexer = Token::lexer(&source); 

        for result in lexer {
            if let Err(_) = result {
                panic!("Invalid identifier"); 
            }
        } 
    }

    fn read_file(path: &str) -> std::io::Result<String> {
        let file = File::open(path)?;
        let mut buf_reader = BufReader::new(file);
        let mut contents = String::new();
        buf_reader.read_to_string(&mut contents)?; 

        Ok(contents)    
    }

    fn compare_tokens(lexer: Lexer<Token>, tokens: Vec<Token>) {
        for (a,b) in zip(lexer, tokens) {
            if let Ok(token) = a {
                assert_eq!(token, b); 
            }
        }
    }
}