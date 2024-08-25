use std::{collections::VecDeque, fmt::format};

use logos::Lexer;

use crate::{
    ast::{Expression, Function, Identifier, Program, Statement, UnaryOperator}, lexer::Token
};

pub struct CParser {
    pub tokens: VecDeque<Token>,
    pub current_token: Token,
    pub peek_token: Token,
    pub errors: Vec<String>,
}

impl CParser {
    pub fn build(lexer: &mut Lexer<Token>) -> Self {
        let mut tokens: VecDeque<Token> =
            VecDeque::from_iter(lexer.into_iter().map(|x| x.unwrap()));

        let current = tokens.pop_front().unwrap();
        let peek = tokens.pop_front().unwrap();

        Self {
            tokens: tokens.clone(),
            errors: Vec::new(),
            current_token: current,
            peek_token: peek,
        }
    }

    fn parse_unary_operator(&mut self) -> Result<UnaryOperator, String> {
        todo!()
    } 

    fn parse_statement(&mut self) -> Result<Statement, String> {
        if self.current_token_is(Token::Return) {
            self.next_token();

            let expression = self.parse_expression()?;

            self.next_token();

            if self.current_token_is(Token::Semicolon) {
                self.next_token();
                Ok(Statement::Return(expression))
            } else {
                let error = format!("Expected ; but found {:?}", self.current_token); 
                self.errors.push(error.clone()); 
                Err(error)
            }
        } else {
            let error = format!("Expected RETURN but found {:?}", self.current_token); 
            self.errors.push(error.clone()); 
            Err(error)
        }
    }

    fn parse_expression(&mut self) -> Result<Expression, String> {
        match self.current_token {
            Token::Constant(i) => Ok(Expression::Constant(i)),
            Token::Negation => {
                self.next_token(); 
                Ok(Expression::Unary(
                UnaryOperator::Negate,
                Box::new(self.parse_expression().unwrap()),
            ))
            }, 
            Token::BitComp => {
                self.next_token(); 
                Ok(
                    Expression::Unary(UnaryOperator::Complement, Box::new(self.parse_expression().unwrap()))
                )
            }, 
            Token::LParen => {
                self.next_token(); 
                let inner_exp = self.parse_expression(); 
                self.next_token(); 
                if self.current_token_is(Token::RParen) {
                    inner_exp
                } else {
                    let error = format!("Missing closing parenthesis"); 
                    self.errors.push(error.clone()); 
                    Err(error)
                }
            }
            ,
            _ => 
            {
                let error =format!(
                    "Malformed expression, found {:?}",
                    self.current_token
                ); 
                self.errors.push(error.clone()); 
                Err(error)
            }
            ,
        }
    }

    fn parser_identifier(&mut self) -> Result<Identifier, String> {
        if let Token::Identifier(s) = self.current_token.clone() {
            self.next_token();
            Ok(Identifier(s.to_string()))
        } else {

            let error = format!(
                "Error parsing identifier, got {:?}",
                self.current_token.clone()
            ); 

            self.errors.push(error.clone()); 
            Err(error)
        }
    }

    fn parse_function(&mut self) -> Result<Function, String> {
        if self.current_token_is(Token::Int) {
            self.next_token();
            let identifier = self.parser_identifier()?;

            let structure = vec![Token::LParen, Token::Void, Token::RParen, Token::LBrace];

            for token in structure {
                if !self.current_token_is(token.clone()) {
                    self.next_token();

                    let error = format!(
                        "Expected {:?}, got {:?}",
                        token, self.current_token
                    ); 

                    self.errors.push(error.clone()); 
                    return Err(error); 
                } else {
                    self.next_token();
                }
            }

            let statement = self.parse_statement()?;

            if self.current_token_is(Token::RBrace) {
                self.next_token();
                Ok(Function {
                    name: identifier,
                    body: statement,
                })
            } else {

                let error = format!(
                    "Expected }} but found {:?}",
                    self.current_token.clone()
                ); 
                self.errors.push(error.clone()); 
                return Err(error); 
            }
        } else {
            let error =format!("Expected int but found {:?}", self.current_token);  
            self.errors.push(error.clone()); 
            Err(error)
        }
    }

    pub fn parse_program(&mut self) -> Result<Program, String> {
        let function = self.parse_function()?;

        Ok(Program(function))
    }

    fn current_token_is(&self, token: Token) -> bool {
        self.current_token == token
    }

    fn peek_token_is(&self, token: Token) -> bool {
        self.peek_token == token
    }

    fn next_token(&mut self) {
        self.current_token = self.peek_token.clone();
        self.peek_token = self
            .tokens
            .pop_front()
            .unwrap_or(self.current_token.clone());
    }
}
