use crate::assembly::Assembly;
use crate::lexer::Token;
use crate::parser::Parser;
use crate::tac::TAC;
use crate::visitor::AssemblyPass;
use clap::Parser as ClapParser;
use logos::Logos;
use std::fs::{self, File};
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(ClapParser, Debug, Clone)]
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

    /// Directs preprocessor to run lexing, parsing, and
    /// assembly generation, but stop before code
    /// emission.
    #[arg(long)]
    codegen: bool,

    /// Directs preprocessor to run everything up to (and including)
    /// TAC generation.
    #[arg(long)]
    tac: bool,

    #[arg(long)]
    emitcode: bool,
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
            let mut parser = Parser::build(&mut lexer);
            let mut tac = TAC::build(parser.to_ast_program());
            let mut assembly = Assembly::new(tac.to_tac_program());

            let output_path = output_assembler
                .clone()
                .into_os_string()
                .into_string()
                .unwrap();
            println!("Writing: ");

            let path = Path::new(&output_path);
            let display = path.display();

            // Open a file in write-only mode, returns `io::Result<File>`
            let mut file = match File::create(path) {
                Err(why) => panic!("couldn't create {}: {}", display, why),
                Ok(file) => file,
            };

            match file.write_all(assembly.to_assembly_program().format().as_bytes()) {
                Err(why) => panic!("couldn't write to {}: {}", display, why),
                Ok(_) => println!("successfully wrote to {}", display),
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

    #[allow(dead_code)]
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

    /// Outputs the token stream.
    fn lex_file(&self) -> Result<(), String> {
        if self.file.exists() {
            let file = fs::read_to_string(&self.file).expect("Unable to read file.");
            let lexer = Token::lexer(&file);
            let tokens: Vec<Token> = lexer.clone().map(|x| x.unwrap()).collect();
            println!("{:?}", lexer);
            println!("{:?}", tokens);
            Ok(())
        } else {
            Err("Failed lexing file, no such file".to_string())
        }
    }

    /// Outputs the AST generated by the parser.
    fn parse_file(&self) -> Result<(), String> {
        if self.file.exists() {
            let file = fs::read_to_string(&self.file).expect("Unable to read file.");
            let mut lexer = Token::lexer(&file);
            let mut parser = Parser::build(&mut lexer);

            println!("{:?}", parser.to_ast_program());

            Ok(())
        } else {
            Err("Failed parsing file, no such file".to_string())
        }
    }

    fn code_gen(&self) -> Result<(), String> {
        if self.file.exists() {
            let file = fs::read_to_string(&self.file).expect("Unable to read file.");
            let mut lexer = Token::lexer(&file);
            let mut parser = Parser::build(&mut lexer);
            let mut tac = TAC::build(parser.to_ast_program());
            let mut assembly = Assembly::new(tac.to_tac_program());
            // Parsing the program
            assembly.parse_program();

            // Visiting the program
            let mut visitor = AssemblyPass::build(assembly);
            visitor.print_instructions(Some("Original instructions"));
            visitor.replace_pseudo_registers();
            visitor.print_instructions(Some("Replacing pseudo registers"));
            visitor.rewrite_mov();
            visitor.print_instructions(Some("Rewriting move instructions"));
            visitor.rewrite_binop();
            visitor.print_instructions(Some("Rewriting binary operators"));
            Ok(())
        } else {
            Err("Failed parsing file, no such file".to_string())
        }
    }

    /// Output the three adress code intermediate representation.
    fn tac_gen(&self) -> Result<(), String> {
        if self.file.exists() {
            let file = fs::read_to_string(&self.file).expect("Unable to read file");

            let mut lexer = Token::lexer(&file);
            let mut parser = Parser::build(&mut lexer);
            let mut tac = TAC::build(parser.to_ast_program());
            println!("{:?}", tac.to_tac_program());

            Ok(())
        } else {
            Err("Failed finding file, no such file".to_string())
        }
    }

    /// Emmits final assembly code
    fn emit_code(&self) -> Result<(), String> {
        if self.file.exists() {
            let file = fs::read_to_string(&self.file).expect("Reading file");
            let mut lexer = Token::lexer(&file);
            let mut parser = Parser::build(&mut lexer);
            let mut tac = TAC::build(parser.to_ast_program());
            let mut assembly = Assembly::new(tac.to_tac_program());

            println!("{}", assembly.to_assembly_program().format());

            Ok(())
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
            return Ok(());
        }

        if self.tac {
            self.tac_gen()?;
            return Ok(());
        }

        if self.emitcode {
            self.emit_code()?;
            return Ok(());
        }

        self.preprocess_file()?;
        self.compile_preproc_file()?;
        //self.assemble_file()?;

        Ok(())
    }
}
