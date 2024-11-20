use miette::Result;
use nous::compiler_driver::CompilerDriver;

fn main() -> Result<()> {
    CompilerDriver::build().run()?;
    Ok(())
}
