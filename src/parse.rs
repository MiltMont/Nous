use std::collections::VecDeque;

use logos::Lexer;

use crate::{ast, lexer::Token};

/// Turns a stream of Tokens into an AST Program
pub struct Parser {
    /// Queue of tokens
    pub tokens: VecDeque<Token>,
    /// Current token in token stream
    pub current_token: Token,
    /// Next token in token stream
    pub peek_token: Token,
}

impl Parser {
    /// Returns a Parser given a Lexer.
    pub fn build(lexer: &mut Lexer<Token>) -> Self {
        let mut tokens: VecDeque<Token> =
            VecDeque::from_iter(lexer.into_iter().map(|x| x.expect("Building token queue")));

        let current_token = tokens.pop_front().unwrap();
        let peek_token = tokens.pop_front().unwrap();

        Self {
            tokens,
            current_token,
            peek_token,
        }
    }

    /// Generates and AST from the constructed parser.
    pub fn to_ast_program(&mut self) -> ast::Program {
        self.parse_program().expect("Parser: Parsing program.")
    }

    /// Consumes the next token in token stream
    fn next_token(&mut self) {
        self.current_token = self.peek_token.clone();
        self.peek_token = self
            .tokens
            .pop_front()
            .unwrap_or(self.current_token.clone());
    }

    /// Compares current token with a given token
    fn current_token_is(&self, token: &Token) -> bool {
        self.current_token == *token
    }

    /// Returns an AST Program or an Error string.
    fn parse_program(&mut self) -> Result<ast::Program, String> {
        let function = self.parse_function()?;
        Ok(ast::Program(function))
    }

    /// Returns an ast::Function or an Error String.
    fn parse_function(&mut self) -> Result<ast::Function, String> {
        if self.current_token_is(&Token::Int) {
            self.next_token();

            let identifier = self.parse_identifier()?;
            let expected_structure = vec![Token::LParen, Token::Void, Token::RParen, Token::LBrace];

            // Check if incoming token stream matches the expected_structure
            for token in expected_structure {
                if !self.current_token_is(&token) {
                    return Err(format!(
                        "Expected {:?}, got {:?}",
                        token, self.current_token
                    ));
                } else {
                    self.next_token();
                }
            }

            let statement = self.parse_statement()?;

            if self.current_token_is(&Token::RBrace) {
                self.next_token();
                Ok(ast::Function {
                    name: identifier,
                    body: statement,
                })
            } else {
                Err(format!("Expected }} but found {:?}", self.current_token))
            }
        } else {
            Err(format!("Expected int but found {:?}", self.current_token))
        }
    }

    /// Returns an ast::Identifier or an Error String.
    fn parse_identifier(&mut self) -> Result<ast::Identifier, String> {
        if let Token::Identifier(s) = self.current_token.clone() {
            self.next_token();
            Ok(ast::Identifier(s.to_string()))
        } else {
            Err(format!(
                "Error parsing identifier, got {:?}",
                self.current_token
            ))
        }
    }

    fn parse_expression(&mut self) -> Result<ast::Expression, String> {
        match self.current_token {
            Token::Constant(i) => Ok(ast::Expression::Constant(i)),
            Token::Negation => {
                self.next_token();
                Ok(ast::Expression::Unary(
                    ast::UnaryOperator::Negate,
                    Box::new(
                        self.parse_expression()
                            .expect("Parsing negation expression"),
                    ),
                ))
            }
            Token::BitComp => {
                self.next_token();
                Ok(ast::Expression::Unary(
                    ast::UnaryOperator::Complement,
                    Box::new(self.parse_expression().expect("Parsing BitComp expression")),
                ))
            }
            Token::LParen => {
                self.next_token();
                let inner_expression = self.parse_expression();
                self.next_token();
                if self.current_token_is(&Token::RParen) {
                    inner_expression
                } else {
                    Err(String::from("Missing clossing parenthesis"))
                }
            }
            _ => Err(format!(
                "Malformed expression found {:?}",
                self.current_token
            )),
        }
    }

    fn parse_statement(&mut self) -> Result<ast::Statement, String> {
        if self.current_token_is(&Token::Return) {
            self.next_token();

            let expression = self.parse_expression()?;

            self.next_token();

            if self.current_token_is(&Token::Semicolon) {
                self.next_token();
                Ok(ast::Statement::Return(expression))
            } else {
                Err(format!("Expected ; but found {:?}", self.current_token))
            }
        } else {
            Err(format!(
                "Expected RETURN but found {:?}",
                self.current_token
            ))
        }
    }
}
