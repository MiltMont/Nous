use logos::Logos; 

#[derive(Logos, Debug, PartialEq)]
// TODO: Skip block comments #[logos(skip r"\/*(?:[^*]|\*[^/])*\*\/")]
#[logos(skip r"[ \t\n\f]+")] 
#[logos(skip r"//[^\n]*")] // Skips comments
pub enum Token {
    
    #[regex("[a-zA-Z_]+", |lex| lex.slice().to_owned())]
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

    // Keywords
    #[token("int")]
    Int, 

    #[token("void")]
    Void, 

    #[token("return")]
    Return, 

}