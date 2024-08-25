use clap::Parser;
use logos::Logos;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

use crate::assembly_parser::AssemblyParser;
use crate::lexer::Token;
use crate::parser::CParser;

#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
pub struct CompilerDriver {
    /// Path of the C program.
    #[arg(short, long)]
    file: PathBuf,

    /// Directs preprocessor to run the lexer,
    /// but stop before parsing.
    #[arg(long)]
    lex: bool,

    /// Directs preprocessor to run the lexer and parser,
    /// but stop before assembly generation.
    #[arg(long)]
    parse: bool,

    /// Directs it to performing lexing, parsing, and
    /// assembly generation, but stop before code
    /// emission.
    #[arg(long)]
    codegen: bool,
}

impl CompilerDriver {
    pub fn build() -> CompilerDriver {
        CompilerDriver::parse()
    }

    fn preprocess_file(&self) -> Result<(), String> {
        if self.file.exists() {
            let mut output_file = self.file.clone();
            output_file.set_extension("i");

            Command::new("gcc")
                .args([
                    "-E",
                    "-P",
                    &self.file.clone().into_os_string().into_string().unwrap(),
                    "-o",
                    &output_file.into_os_string().into_string().unwrap(),
                ])
                .output()
                .expect("Failed file preprocessing");

            Ok(())
        } else {
            Err(format!("The file {} does not exists", self.file.display()))
        }
    }

    fn compile_preproc_file(&self) -> Result<(), String> {
        let mut preproc_file = self.file.clone();
        preproc_file.set_extension("i");

        if preproc_file.exists() {
            let mut output_assembler = PathBuf::from(&self.file);
            output_assembler.set_extension("s");

            /*
            Command::new("gcc")
            .args([
                "-S",
                "-O",
                    "-fno-asynchronous-unwind-tables",
                    "-fcf-protection=none",
                    &preproc_file.clone().into_os_string().into_string().unwrap(),
                    "-o",
                    &output_assembler.into_os_string().into_string().unwrap(),
                ])
                .output()
                .expect("Error compiling file");
            */

            // Basic compiler implementation
            let path = preproc_file.clone().into_os_string().into_string().unwrap();
            let file = fs::read_to_string(path).expect("Unable to read file");
            let mut lexer = Token::lexer(&file);
            let mut parser = CParser::build(&mut lexer);
            let output_path = output_assembler
                .clone()
                .into_os_string()
                .into_string()
                .unwrap();

            match parser.parse_program() {
                Ok(program) => {
                    let assembly = AssemblyParser::build(program); 
                    println!("Writing: "); 
                    assembly.write(output_path); 
                }
                Err(e) => panic!("{e}"),
            }

            // Deleting the preprocessed file
            Command::new("rm")
                .arg(preproc_file.into_os_string().into_string().unwrap())
                .output()
                .expect("Error deleting preprocessed file");

            Ok(())
        } else {
            Err(format!(
                "Error in compilation. The file {} does not exists",
                preproc_file.display()
            ))
        }
    }

    fn assemble_file(&self) -> Result<(), String> {
        let mut assembly_file = self.file.clone();
        assembly_file.set_extension("s");

        if assembly_file.exists() {
            dbg!("Assembly exists at {:?}", &assembly_file);
            let mut output_file = self.file.clone();
            output_file.set_extension("");

            Command::new("gcc")
                .args([
                    &assembly_file
                        .clone()
                        .into_os_string()
                        .into_string()
                        .unwrap(),
                    "-o",
                    &output_file.into_os_string().into_string().unwrap(),
                ])
                .output()
                .expect("Failed assembling file");

            Command::new("rm")
                .arg(assembly_file.into_os_string().into_string().unwrap())
                .output()
                .expect("Error deleting assembly file");

            Ok(())
        } else {
            Err(format!(
                "Assembly file {} doesnt exist.",
                assembly_file.display()
            ))
        }
    }

    fn lex_file(&self) -> Result<(), String> {
        if self.file.exists() {
            let file = fs::read_to_string(&self.file).expect("Unable to read file.");
            let lexer = Token::lexer(&file);
            let tokens: Vec<Token>= lexer.clone().map(|x| x.unwrap()).collect(); 
            println!("{:?}", lexer);
            println!("{:?}", tokens);  
            Ok(())
        } else {
            Err("Failed lexing file, no such file".to_string())
        }
    }

    fn parse_file(&self) -> Result<(), String> {
        if self.file.exists() {
            let file = fs::read_to_string(&self.file).expect("Unable to read file."); 
            let mut lexer = Token::lexer(&file); 
            let mut parser = CParser::build(&mut lexer);

            if let Ok(program) = parser.parse_program(){
                println!("{:?}", program); 
                Ok(())
            } else {
                Err(format!("{:?}", parser.errors))
            }
        } else {
            Err("Failed parsing file, no such file".to_string())
        }
    }

    fn code_gen(&self) -> Result<(), String> {
        if self.file.exists() {
            let file = fs::read_to_string(&self.file).expect("Unable to read file."); 
            let mut lexer = Token::lexer(&file); 
            let mut parser = CParser::build(&mut lexer);

            if let Ok(program) = parser.parse_program(){
                let assembly = AssemblyParser::build(program); 
                println!("{:?}", assembly); 
                Ok(())
            } else {
                Err(format!("{:?}", parser.errors))
            }
        } else {
            Err("Failed parsing file, no such file".to_string())
        } 
    }

    pub fn run(self) -> Result<(), String> {

        if self.lex {
            self.lex_file()?; 
            return Ok(()); 
        }

        if self.parse {
            self.parse_file()?; 
            return Ok(()); 
        }

        if self.codegen {
            self.code_gen()?; 
            return Ok(())
        }

        self.preprocess_file()?;
        self.compile_preproc_file()?;
        self.assemble_file()?;

        Ok(())
    }
}
