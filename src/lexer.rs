use logos::Logos;

#[derive(Hash, Eq, Logos, Debug, PartialEq, Clone)]
// TODO: Skip block comments #[logos(skip r"\/*(?:[^*]|\*[^/])*\*\/")]
#[logos(skip r"[ \t\n\f]+")]
#[logos(skip r"//[^\n]*")] // Skips comments
pub enum Token {
    #[regex("[a-zA-Z_]+", |lex| lex.slice().to_string())]
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
