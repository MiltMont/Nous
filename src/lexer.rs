use logos::Logos; 

#[derive(Logos, Debug, PartialEq)]
#[logos(skip r"[ \t\n\f]+")] 
pub enum Token {
    
    #[regex("[a-zA-Z_]+", |lex| lex.slice().to_owned())]
    Identifier(String),

    #[regex(r"[0-9]+", |lex| lex.slice().parse::<f64>().unwrap())]
    Constant(f64), 

    #[token("(")]
    RParen, 

    #[token(")")]
    LParen, 

    #[token("{")]
    RBrace, 

    #[token("}")]
    LBrace, 

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