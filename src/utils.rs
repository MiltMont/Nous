use crate::{parser::Parser, tac::TAC};
use std::{
    fs::File,
    io::{prelude::*, BufReader},
};

pub fn read_file(path: &str) -> std::io::Result<String> {
    let file = File::open(path)?;
    let mut buf_reader = BufReader::new(file);
    let mut contents = String::new();
    buf_reader.read_to_string(&mut contents)?;

    Ok(contents)
}

pub fn parser_from_path(path: &str) -> Parser {
    let file = read_file(path).expect("Should return the file");
    Parser::from(file)
}

pub fn tac_from_path(path: &str) -> TAC {
    let mut parser = parser_from_path(path);
    TAC::from(&mut parser)
}
