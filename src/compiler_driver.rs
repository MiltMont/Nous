
use std::path::PathBuf;
use std::process::Command;
use clap::Parser;

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
                    &output_file.into_os_string().into_string().unwrap()
                ])
                .output()
                .expect("Failed file preprocessing");

            Ok(())
        } else {
            Err(format!("The file {} does not exists", self.file.display()))
        }
    }

    fn compile_preproc_file(&self) -> Result<(), String>{
        let mut preproc_file = self.file.clone(); 
        preproc_file.set_extension("i"); 

        if preproc_file.exists() {

            let mut output_assembler = PathBuf::from(&self.file); 
            output_assembler.set_extension("s");

            // TODO: Remove this and implement the compiler
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
            
            // Deleting the preprocessed file
            Command::new("rm")
                .arg(preproc_file.into_os_string().into_string().unwrap())
                .output()
                .expect("Error deleting preprocessed file");

            Ok(())
        } else {
            Err(format!("Error in compilation. The file {} does not exists", preproc_file.display()))
        }
    }

    fn assemble_file(&self) -> Result<(), String> {
    
        let mut assembly_file = self.file.clone(); 
        assembly_file.set_extension("s");

        if assembly_file.exists() {
            let mut output_file = self.file.clone(); 
            output_file.set_extension(""); 

            Command::new("gcc")
                .args([
                    &assembly_file.clone().into_os_string().into_string().unwrap(), 
                    "-o", 
                    &output_file.into_os_string().into_string().unwrap(),  
                ])
                .output()
                .expect("Failed assemblying file");
            
            // Deleting the assembly file. 
            Command::new("rm")
                .arg(
                    assembly_file.into_os_string().into_string().unwrap()
                )
                .output()
                .expect("Error deleting assembly file"); 

            Ok(())
        } else {
            Err(format!("Assembly file {} doesnt exist.", assembly_file.display()))
        }
    }

    pub fn run(self) -> Result<(), String>{
        self.preprocess_file()?; 
        self.compile_preproc_file()?; 
        self.assemble_file()?;

        Ok(()) 
    }

}