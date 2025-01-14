use crate::assembly::Assembly;
use crate::errors::Result;
use crate::lexer::Token;
use crate::parser::Parser;
use crate::tac;
use crate::tac::TAC;
use crate::visitor::{assembly_passes, validation_passes};
use clap::{Parser as ClapParser, Subcommand};
use logos::Logos;
use miette::Result as MResult;
use std::fs::{self, File};
use std::io::{self, prelude::*};
use std::path::{Path, PathBuf};
use std::process::Command;

// TODO: Change this to handle multiple files.
// To handle multiple
// source files, your compiler driver should convert each one to assembly sepa-
// rately, then use the gcc command to assemble them and link them together.
#[derive(ClapParser)]
#[clap(author, version, about)]
pub struct CompilerDriver {
    /// Path of the C program.
    #[clap(short = 'f', long)]
    file_path: PathBuf,

    /// Tells the driver to invoke the linker or not
    #[clap(short = 'c')]
    invoke_linker: bool,

    #[command(subcommand)]
    cmd: Option<Commands>,
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
            assembly_passes(&mut assembly);
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

    /// When this is called, the compiler driver should
    /// first convert the source program to assembly as usual,
    /// then run the following command to convert the assembly
    /// program into an object file:
    ///
    /// `gcc -c ASSEMBLY_FILE -o OUTPUT_FILE`
    ///
    /// The output filename should be the original filename with
    /// a `.o` suffix. In other words, `-c /path/to/program.c` should
    /// produce an object file at `/path/to/program.o`.
    fn call_linker(&self) -> Result<()> {
        if self.file_path.exists() {
            let mut output_assembly = PathBuf::from(&self.file_path);
            output_assembly.set_extension("s");
            let mut output_object = output_assembly.clone();
            output_object.set_extension("o");
            let mut assembly = Assembly::from(self.file_path.clone());
            assembly.parse_program();
            assembly_passes(&mut assembly);

            // Writting assembly to /path/to/program.s
            fs::write(&output_assembly, assembly.program.unwrap().format())?;

            // Run required gcc command.
            Command::new("gcc")
                .args([
                    "-c",
                    output_assembly.to_str().unwrap(),
                    "-o",
                    output_object.to_str().unwrap(),
                ])
                .output()
                .expect("Should create object file");

            Ok(())
        } else {
            Err(crate::errors::Error::IoError(io::Error::other(
                "Failed lexing file, no such file",
            )))?
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
            assembly_passes(&mut assembly);

            Ok(())
        } else {
            Err(crate::errors::Error::IoError(io::Error::other(
                "Failed code generation, no such file",
            )))?
        }
    }

    /// Emmits final assembly code
    fn emit_code(&self) -> Result<()> {
        if self.file_path.exists() {
            let mut assembly = Assembly::from(self.file_path.clone());
            assembly.parse_program();
            // Visiting the program
            assembly_passes(&mut assembly);
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

            validation_passes(&mut ast);
            println!("{ast:?}");

            Ok(())
        } else {
            Err(crate::errors::Error::IoError(io::Error::other(
                "Failed code emission, no such file",
            )))?
        }
    }

    pub fn run(self) -> MResult<()> {
        if let Some(comand) = &self.cmd {
            match comand {
                Commands::Lex => self.lex_file()?,
                Commands::Parse => self.parse_file()?,
                Commands::CodeGen => self.code_gen()?,
                Commands::Tac => self.tac_gen()?,
                Commands::EmitCode => self.emit_code()?,
                Commands::Validate => self.validate()?,
            }
        }

        if self.invoke_linker {
            self.call_linker()?;
        }
        // self.preprocess_file()?;
        // self.compile_preproc_file()?;
        // //self.assemble_file()?;

        Ok(())
    }
}
