use nous::compiler_driver::CompilerDriver;

fn main() -> Result<(), String> {
    CompilerDriver::build().run()?;

    Ok(())
}

