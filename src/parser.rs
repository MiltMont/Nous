use logos::Lexer;

use crate::{
    ast::{Expression, Function, Identifier, Program, Statement},
    lexer::Token,
};

pub struct Parser<'a> {
    pub lexer: &'a mut Lexer<'a, Token>,
    pub errors: Vec<String>,
}

impl<'a> Parser<'a> {
    pub fn new(lexer: &'a mut Lexer<'a, Token>) -> Self {
        Parser {
            lexer,
            errors: Vec::new(),
        }
    }

    pub fn parse_expression(&mut self) -> Result<Expression, String> {
        if let Some(Ok(Token::Constant(i))) = self.lexer.next() {
            Ok(Expression::Constant(i))
        } else {
            let error = format!("Expected a constant, found {:?}", self.lexer.slice());
            self.errors.push(error.clone());
            Err(error)
        }
    }

    pub fn parser_statement(&mut self) -> Result<Statement, String> {
        if self.lexer.next() == Some(Ok(Token::Return)) {
            let expression = self.parse_expression()?;

            if self.lexer.next() == Some(Ok(Token::Semicolon)) {
                Ok(Statement::Return(expression))
            } else {
                let error = format!("Expected ; found {:?}", self.lexer.slice());
                self.errors.push(error.clone());
                Err(error)
            }
        } else {
            let error = format!("Expected Return, found {:?}", self.lexer.slice());
            self.errors.push(error.clone());
            Err(error)
        }
    }

    pub fn parse_identifier(&mut self) -> Result<Identifier, String> {
        if let Some(Ok(Token::Identifier(s))) = self.lexer.next() {
            Ok(Identifier(s))
        } else {
            let error = format!("Expected an identifier, found {:?}", self.lexer.slice());
            self.errors.push(error.clone());

            Err(error)
        }
    }

    pub fn parse_function(&mut self) -> Result<Function, String> {
        if self.lexer.next() == Some(Ok(Token::Int)) {
            let identifier = self.parse_identifier()?;

            let structure = vec![Token::LParen, Token::Void, Token::RParen, Token::LBrace];

            for token in structure {
                if self.lexer.next() != Some(Ok(token.clone())) {
                    let error = format!("Expected {:?}, found {:?}", token, self.lexer.slice());
                    self.errors.push(error.clone());
                    return Err(error);
                }
            }

            let statement = self.parser_statement()?;

            if self.lexer.next() == Some(Ok(Token::RBrace)) {
                Ok(Function {
                    name: identifier,
                    body: statement,
                })
            } else {
                let error = format!("Expected }} got {:?}", self.lexer.slice());
                self.errors.push(error.clone());
                Err(error)
            }
        } else {
            let error = format!("Expected int keyword, found {:?}", self.lexer.slice());
            self.errors.push(error.clone());
            Err(error)
        }
    }

    pub fn parse_program(&mut self) -> Result<Program, String> {
        let function = self.parse_function()?;

        Ok(Program(function))
    }
}
