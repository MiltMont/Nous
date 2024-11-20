use crate::errors::{Error, Result};
use logos::Logos;

#[derive(Hash, Eq, Logos, Debug, PartialEq, Clone)]
// TODO: Skip block comments #[logos(skip r"\/*(?:[^*]|\*[^/])*\*\/")]
#[logos(skip r"[ \t\n\f]+")]
#[logos(skip r"//[^\n]*")] // Skips comments
pub enum Token {
    // FIX: This cannot parse identifiers like `temp1`
    #[regex("[a-zA-Z][a-zA-Z0-9_-]*", |lex| lex.slice().to_string())]
    Identifier(String),

    #[regex(r"[0-9]+", |lex| lex.slice().parse::<i64>().unwrap())]
    Constant(i64),

    #[token("(")]
    LParen,

    #[token(")")]
    RParen,

    #[token("{")]
    LBrace,

    #[token("}")]
    RBrace,

    #[token(";")]
    Semicolon,

    #[token("~")]
    BitComp, // Bitwise complement operator

    #[token("-")]
    Negation,

    #[token("--")]
    Decrement,

    // Keywords
    #[token("int")]
    Int,

    #[token("void")]
    Void,

    #[token("return")]
    Return,

    // Arithmetic operators
    /// Addition
    #[token("+")]
    Add,

    /// Multiplication
    #[token("*")]
    Mul,

    /// Division
    #[token("/")]
    Div,

    /// Remainder
    #[token("%")]
    Remainder,

    // Logical and relational operators.
    #[token("!")]
    Not,

    #[token("&&")]
    And,

    #[token("||")]
    Or,

    #[token("==")]
    EqualTo,

    #[token("!=")]
    NotEqualTo,

    #[token("<")]
    LessThan,

    #[token(">")]
    GreaterThan,

    #[token("<=")]
    LessThanOrEq,

    #[token(">=")]
    GreaterThanOrEq,

    /// Assignment operator
    #[token("=")]
    Assign,
}

impl Token {
    pub fn precedence(&self) -> Result<usize> {
        match self {
            Token::Mul => Ok(50),
            Token::Div => Ok(50),
            Token::Remainder => Ok(50),
            Token::Add => Ok(45),
            Token::Negation => Ok(45),
            Token::LessThan => Ok(35),
            Token::LessThanOrEq => Ok(35),
            Token::GreaterThan => Ok(35),
            Token::GreaterThanOrEq => Ok(35),
            Token::EqualTo => Ok(30),
            Token::NotEqualTo => Ok(30),
            Token::And => Ok(10),
            Token::Or => Ok(5),
            Token::Assign => Ok(1),
            token => Err(Error::Precedence {
                found: token.clone(),
            }),
        }
    }
}
