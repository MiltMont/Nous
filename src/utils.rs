use std::{
    fs::File,
    io::{prelude::*, BufReader},
};

use logos::Logos;

use crate::{lexer::Token, parser::Parser, tac::TAC};

pub fn read_file(path: &str) -> std::io::Result<String> {
    let file = File::open(path)?;
    let mut buf_reader = BufReader::new(file);
    let mut contents = String::new();
    buf_reader.read_to_string(&mut contents)?;

    Ok(contents)
}

pub fn parser_from_file(path: &str) -> Parser {
    Parser::build(&mut Token::lexer(&read_file(path).unwrap()))
}

pub fn tac_from_file(path: &str) -> TAC {
    let mut parser = parser_from_file(path);
    TAC::build(parser.to_ast_program())
}
