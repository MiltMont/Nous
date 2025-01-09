use crate::assembly::Assembly;
use crate::assembly_passes::{
    AllocateStack, ReplacePseudoRegisters, RewriteBinaryOp, RewriteCmp, RewriteMov,
};
use crate::errors::Result;
use crate::lexer::Token;
use crate::loop_labeling::LoopLabeling;
use crate::parser::Parser;
use crate::tac;
use crate::tac::TAC;
use crate::variable_resolution::VariableResolution as VarRes;
use crate::visitor::{apply_visitor_with_context, visit_collection, visit_collection_with_context};
use clap::{Parser as ClapParser, Subcommand};
use logos::Logos;
use miette::Result as MResult;
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{self, prelude::*};
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(ClapParser)]
#[clap(author, version, about)]
pub struct CompilerDriver {
    /// Path of the C program.
    #[clap(short = 'f', long)]
    file_path: PathBuf,

    #[command(subcommand)]
    cmd: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Directs preprocessor to run the lexer,
    /// but stop before parsing.
    Lex,
    /// Directs preprocessor to run the lexer and parser,
    /// but stop before assembly generation.
    Parse,
    /// Runs the compiler through the semantic analysis
    /// stage, stopping before tacky generation.
    Validate,
    /// Directs preprocessor to run everything up to (and including)
    /// TAC generation.
    Tac,
    /// Directs preprocessor to run lexing, parsing, and
    /// assembly generation, but stop before code
    /// emission.
    CodeGen,
    /// Directs preprocessor to run everything up to (and including)
    /// Assembly code generation.
    EmitCode,
}

#[allow(dead_code)]
impl CompilerDriver {
    pub fn build() -> CompilerDriver {
        CompilerDriver::parse()
    }

    fn preprocess_file(&self) -> Result<()> {
        if self.file_path.exists() {
            let mut output_file = self.file_path.clone();
            output_file.set_extension("i");

            Command::new("gcc")
                .args([
                    "-E",
                    "-P",
                    &self
                        .file_path
                        .clone()
                        .into_os_string()
                        .into_string()
                        .unwrap(),
                    "-o",
                    &output_file.into_os_string().into_string().unwrap(),
                ])
                .output()
                .expect("Failed file preprocessing");

            Ok(())
        } else {
            // Err(format!(
            //     "The file {} does not exists",
            //     self.file_path.display()
            // ))
            Err(crate::errors::Error::IoError(io::Error::other(
                "No such file",
            )))?
            // Err(crate::errors::Error::IoError(io::Error::last_os_error()))
        }
    }

    fn compile_preproc_file(&self) -> Result<()> {
        let mut preproc_file = self.file_path.clone();
        preproc_file.set_extension("i");

        if preproc_file.exists() {
            let mut output_assembler = PathBuf::from(&self.file_path);
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
            let mut assembly = Assembly::from(file);
            // Parsing the assembly program.
            assembly.parse_program();
            // Visiting the program
            apply_visitor_with_context(
                &mut assembly.program.as_mut().unwrap().0.instructions,
                ReplacePseudoRegisters,
                &mut assembly.pseudo_registers,
            );
            visit_collection(
                &mut assembly.program.as_mut().unwrap().0.instructions,
                RewriteMov,
            );
            visit_collection(
                &mut assembly.program.as_mut().unwrap().0.instructions,
                RewriteBinaryOp,
            );
            visit_collection(
                &mut assembly.program.as_mut().unwrap().0.instructions,
                RewriteCmp,
            );
            visit_collection_with_context(
                &mut assembly.program.as_mut().unwrap().0.instructions,
                AllocateStack,
                &mut assembly.offset.clone(),
            );
            //let mut assembly_pass = AssemblyPass::build(assembly);
            //assembly_pass
            //    .replace_pseudo_registers()
            //    .rewrite_binop()
            //    .rewrite_mov()
            //    .allocate_stack();
            //
            //let assembly_program = assembly_pass.modify_program();
            //println!("{:?}", assembly_program);
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

            match file.write_all(assembly.program.unwrap().format().as_bytes()) {
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
            // Err(format!(
            //     "Error in compilation. The file {} does not exists",
            //     preproc_file.display()
            // ))
            //
            //
            Err(crate::errors::Error::IoError(io::Error::other(
                "No such file",
            )))?
            // Err(crate::errors::Error::IoError(io::Error::last_os_error()))
        }
    }

    #[allow(dead_code)]
    fn assemble_file(&self) -> Result<()> {
        let mut assembly_file = self.file_path.clone();
        assembly_file.set_extension("s");

        if assembly_file.exists() {
            dbg!("Assembly exists at {:?}", &assembly_file);
            let mut output_file = self.file_path.clone();
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
            // Err(format!(
            //     "Assembly file {} doesnt exist.",
            //     assembly_file.display()
            // ))
            Err(crate::errors::Error::IoError(io::Error::other(
                "Assembly file doesnt exist.",
            )))?
            // Err(crate::errors::Error::IoError(io::Error::last_os_error()))
        }
    }

    /// Outputs the token stream.
    fn lex_file(&self) -> Result<()> {
        if self.file_path.exists() {
            let file = fs::read_to_string(&self.file_path).expect("Unable to read file.");
            let lexer = Token::lexer(&file);
            let tokn = Vec::from_iter(lexer);
            // let tokens: Vec<Token> = Vec::from_iter(lexer.clone().map(|x| x.unwrap()));
            // println!("{:?}", lexer);
            // println!("{:?}", tokens);
            println!("{:?}", tokn);
            Ok(())
        } else {
            Err(crate::errors::Error::IoError(io::Error::other(
                "Failed lexing file, no such file",
            )))?
            // Err(crate::errors::Error::IoError(io::Error::last_os_error()))
            // Err("Failed lexing file, no such file".to_string())
        }
    }

    /// Outputs the AST generated by the parser.
    fn parse_file(&self) -> Result<()> {
        if self.file_path.exists() {
            let mut parser = Parser::from(self.file_path.clone());
            //let ast_program: ast::Program = (&mut parser).into();
            let ast = parser.to_ast_program()?;
            println!("{:?}", ast);

            Ok(())
        } else {
            // Err("Failed parsing file, no such file".to_string())
            Err(crate::errors::Error::IoError(io::Error::other(
                "Failed parsing file, no such file",
            )))?
            // Err(crate::errors::Error::IoError(io::Error::last_os_error()))
        }
    }

    /// Output the three adress code intermediate representation.
    fn tac_gen(&self) -> Result<()> {
        if self.file_path.exists() {
            let mut tac = TAC::from(self.file_path.clone());
            let tac_program: tac::Program = (&mut tac).into();
            println!("{:?}", tac_program);

            Ok(())
        } else {
            Err(crate::errors::Error::IoError(io::Error::other(
                "Failed TAC generation, no such file",
            )))?
            // Err(crate::errors::Error::IoError(io::Error::last_os_error()))
            // Err("Failed finding file, no such file".to_string())
        }
    }

    fn code_gen(&self) -> Result<()> {
        if self.file_path.exists() {
            let mut assembly = Assembly::from(self.file_path.clone());
            // Parsing the program
            assembly.parse_program();

            // Visiting the program
            apply_visitor_with_context(
                &mut assembly.program.as_mut().unwrap().0.instructions,
                ReplacePseudoRegisters,
                &mut assembly.pseudo_registers,
            );
            visit_collection(
                &mut assembly.program.as_mut().unwrap().0.instructions,
                RewriteMov,
            );
            visit_collection(
                &mut assembly.program.as_mut().unwrap().0.instructions,
                RewriteBinaryOp,
            );
            visit_collection(
                &mut assembly.program.as_mut().unwrap().0.instructions,
                RewriteCmp,
            );
            visit_collection_with_context(
                &mut assembly.program.as_mut().unwrap().0.instructions,
                AllocateStack,
                &mut assembly.offset.clone(),
            );

            //let mut visitor = AssemblyPass::build(assembly);
            //visitor.print_instructions(Some("Original instructions"));
            //visitor.replace_pseudo_registers();
            //visitor.print_instructions(Some("Replacing pseudo registers"));
            //visitor.rewrite_mov();
            //visitor.print_instructions(Some("Rewriting move instructions"));
            //visitor.rewrite_binop();
            //visitor.print_instructions(Some("Rewriting binary operators"));
            //visitor.rewrite_cmp();
            //visitor.print_instructions(Some("Rewriting cmp operators"));

            Ok(())
        } else {
            Err(crate::errors::Error::IoError(io::Error::other(
                "Failed code generation, no such file",
            )))?
            // Err(crate::errors::Error::IoError(io::Error::last_os_error()))
            // Err("Failed parsing file, no such file".to_string())
        }
    }

    /// Emmits final assembly code
    fn emit_code(&self) -> Result<()> {
        if self.file_path.exists() {
            let mut assembly = Assembly::from(self.file_path.clone());
            assembly.parse_program();
            // Visiting the program
            apply_visitor_with_context(
                &mut assembly.program.as_mut().unwrap().0.instructions,
                ReplacePseudoRegisters,
                &mut assembly.pseudo_registers,
            );
            visit_collection(
                &mut assembly.program.as_mut().unwrap().0.instructions,
                RewriteMov,
            );
            visit_collection(
                &mut assembly.program.as_mut().unwrap().0.instructions,
                RewriteBinaryOp,
            );
            visit_collection(
                &mut assembly.program.as_mut().unwrap().0.instructions,
                RewriteCmp,
            );
            visit_collection_with_context(
                &mut assembly.program.as_mut().unwrap().0.instructions,
                AllocateStack,
                &mut assembly.offset.clone(),
            );
            //let mut visitor = AssemblyPass::build(assembly);
            //visitor
            //    .replace_pseudo_registers()
            //    .rewrite_mov()
            //    .rewrite_binop()
            //    .rewrite_cmp()
            //    .allocate_stack();
            //
            //let assembly_program = visitor.modify_program();
            //println!("{}", assembly_program.format());
            println!("{}", assembly.program.unwrap().format());

            Ok(())
        } else {
            // Err("Failed parsing file, no such file".to_string())
            Err(crate::errors::Error::IoError(io::Error::other(
                "Failed code emission, no such file",
            )))?
        }
    }

    fn validate(&self) -> Result<()> {
        if self.file_path.exists() {
            let mut parser = Parser::from(self.file_path.clone());
            let mut ast = parser.to_ast_program()?;

            apply_visitor_with_context(&mut ast.0.body.0, VarRes::default(), &mut HashMap::new());
            apply_visitor_with_context(&mut ast.0.body.0, LoopLabeling::default(), &mut None);

            println!("{ast:?}");

            Ok(())
        } else {
            Err(crate::errors::Error::IoError(io::Error::other(
                "Failed code emission, no such file",
            )))?
        }
    }

    pub fn run(self) -> MResult<()> {
        match self.cmd {
            Commands::Lex => self.lex_file()?,
            Commands::Parse => self.parse_file()?,
            Commands::CodeGen => self.code_gen()?,
            Commands::Tac => self.tac_gen()?,
            Commands::EmitCode => self.emit_code()?,
            Commands::Validate => self.validate()?,
        }
        // self.preprocess_file()?;
        // self.compile_preproc_file()?;
        // //self.assemble_file()?;

        Ok(())
    }
}
