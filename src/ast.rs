use logos::Lexer;

use crate::lexer::Token;

#[derive(Debug, PartialEq)]
pub enum Expression {
    Constant(i64), 
}

#[derive(Debug, PartialEq)]
pub enum Statement {
    Return(Expression)
} 

#[derive(Debug, PartialEq)]
pub struct Identifier(pub String); 

#[derive(Debug, PartialEq)]
pub struct Function {
    pub name: Identifier, 
    pub body: Statement, 
}

#[derive(Debug, PartialEq)]
pub struct Program(pub Function);

pub fn parse_statement(lexer: &mut Lexer<'_, Token>) -> Result<Statement, String> {
    if lexer.next() == Some(Ok(Token::Return)) {
        let exp = parse_expression(lexer)?;

        if lexer.next() == Some(Ok(Token::Semicolon)) {
            return Ok(Statement::Return(exp))
        } else {
            Err(format!("Expected Semicolon, found {:?}", lexer.slice()))
        }
         

    } else {
        return Err(format!("Expected Int, found {:?}", lexer.slice()));
    }
}

pub fn parse_expression(lexer: &mut Lexer<'_, Token>) -> Result<Expression, String> {
    if let Some(Ok(Token::Constant(i))) = lexer.next() {
        Ok(Expression::Constant(i))
    } else {
        Err(format!("Expected a constant and found {:?}", lexer.slice()))
    }
}

pub fn parse_identifier(lexer: &mut Lexer<'_, Token>) -> Result<Identifier, String> {
    if let Some(Ok(Token::Identifier(s))) = lexer.next() {
        Ok(Identifier(s))
    } else {
        Err(format!("Whatttt"))
    }
}

pub fn parse_function(lexer: &mut Lexer<'_, Token>) -> Result<Function, String> {
    if lexer.next() == Some(Ok(Token::Int)) {
        let ident = parse_identifier(lexer)?;
        ; 

        let test = vec![Token::LParen, Token::Void, Token::RParen, Token::LBrace];

        for token in test {
            if lexer.next() != Some(Ok(token)) {
                return Err(format!("XD"));
            }
        }

        let stm = parse_statement(lexer)?; 

        if lexer.next() == Some(Ok(Token::RBrace)) {
            Ok(Function {
                name: ident, 
                body: stm 
            })
        } else {
            Err(format!("Wow"))
        } 
    } else {
        Err(format!("Noneseañlkdj"))
    }
}

pub fn parse_program(lexer: &mut Lexer<'_, Token>) -> Result<Program, String> {
    let func = parse_function(lexer)?;

    Ok(Program(func))
}