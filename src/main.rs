use logos::Logos;
use miette::Result;
use nous::{compiler_driver::CompilerDriver, lexer::Token};

fn main() -> Result<()> {
    CompilerDriver::build().run()?;

    //     let program = "
    //
    //     int x1 = 0;
    //     int testin320 = 1;
    //
    // ";
    //     let test = Token::lexer(program);
    //     let tokens = Vec::from_iter(test);
    //     println!("{tokens:#?}");
    Ok(())
}
