use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;

use nous::assembly::Assembly;
use nous::visitor::AssemblyPass;

/// Writes to a file named `debug_test.s`
fn write_to_file(name: &str, content: &str) -> std::io::Result<()> {
    // Open the file in write mode, creating it if it doesn't exist
    let file_name = format!("{name}_debug_test.s");
    let mut file = File::create(file_name)?;

    // Write the content to the file
    file.write_all(content.as_bytes())?;
    Ok(())
}

fn compile_assembly(name: &str) -> Result<(), String> {
    let assembly_file = format!("{name}_debug_test.s");
    let output_binary = format!("{name}_debug_test");

    // Use Command to call clang
    let status = Command::new("clang")
        .arg("-o")
        .arg(&output_binary) // Specify the output binary name
        .arg(&assembly_file) // Specify the assembly file
        .status()
        .map_err(|e| format!("Failed to invoke clang: {}", e))?;

    if status.success() {
        println!(
            "Compilation successful: {} -> {}",
            assembly_file, output_binary
        );
        Ok(())
    } else {
        Err(format!("Clang failed with exit code: {:?}", status.code()))
    }
}

fn grab(name: &str) -> Result<i32, String> {
    let command = format!("./{name}_debug_test");

    // Spawn the C program as a subprocess.
    let output = Command::new(command)
        .output()
        .expect("Failed to execute command");

    // Capture the standard output of the C program.
    // let stdout = String::from_utf8_lossy(&output.stdout);

    // Capture the return code of the C program.
    if let Some(exit_code) = output.status.code() {
        Ok(exit_code)
    } else {
        Err("NOOOOO".into())
    }
}

fn clean_files(prefix: &str) -> Result<(), String> {
    let status = Command::new("rm")
        .arg(format!("{prefix}_debug_test.s"))
        .arg(format!("{prefix}_debug_test"))
        .status()
        .map_err(|e| format!("Got {e}"))?;
    if status.success() {
        Ok(())
    } else {
        Err("Failed to remove such files".to_string())
    }
}

#[test]
fn basic_return_2() {
    let program = Assembly::from(PathBuf::from("playground/return_2.c")).to_assembly_program();

    let file_name = "ret_2";
    write_to_file(file_name, &program.format()).expect("Should write to program file.");
    compile_assembly(file_name).expect("Should compile to assembly.");
    let status = grab(file_name).expect("Should grab status code");

    clean_files(file_name).expect("Cleaning files");
    assert_eq!(status, 2);
}

#[test]
fn test_expression_5() {
    let file_name = "exp_5";
    let mut assembly = Assembly::from(PathBuf::from("playground/test_expression5.c"));
    assembly.parse_program();
    let mut visitor = AssemblyPass::build(assembly);
    visitor
        .replace_pseudo_registers()
        .rewrite_mov()
        .rewrite_binop()
        .rewrite_cmp()
        .allocate_stack();

    let program = visitor.modify_program();

    write_to_file(file_name, &program.format()).expect("Should write to program file");

    compile_assembly(file_name).expect("Should compile assembly code");
    let status = grab(file_name).expect("Should grab status code");

    clean_files(file_name).expect("Cleaning files");
    assert_eq!(status, 0);
}

#[test]
fn test_expression_4() {
    let file_name = "exp_4";
    let mut assembly = Assembly::from(PathBuf::from("playground/test_expression4.c"));
    assembly.parse_program();
    let mut visitor = AssemblyPass::build(assembly);
    visitor
        .replace_pseudo_registers()
        .rewrite_mov()
        .rewrite_binop()
        .rewrite_cmp()
        .allocate_stack();

    let program = visitor.modify_program();

    write_to_file(file_name, &program.format()).expect("Should write to program file");

    compile_assembly(file_name).expect("Should compile assembly code");
    let status = grab(file_name).expect("Should grab status code");

    clean_files(file_name).expect("Cleaning files");
    assert_eq!(status, 5);
}

#[test]
fn test_expression_3() {
    let file_name = "exp_3";
    let mut assembly = Assembly::from(PathBuf::from("playground/test_expression3.c"));
    assembly.parse_program();
    let mut visitor = AssemblyPass::build(assembly);
    visitor
        .replace_pseudo_registers()
        .rewrite_mov()
        .rewrite_binop()
        .rewrite_cmp()
        .allocate_stack();

    let program = visitor.modify_program();

    write_to_file(file_name, &program.format()).expect("Should write to program file");

    compile_assembly(file_name).expect("Should compile assembly code");
    let status = grab(file_name).expect("Should grab status code");

    clean_files(file_name).expect("Cleaning files");
    assert_eq!(status, 3);
}

#[test]
fn test_expression_2() {
    let file_name = "exp_2";
    let mut assembly = Assembly::from(PathBuf::from("playground/test_expression.c"));
    assembly.parse_program();
    let mut visitor = AssemblyPass::build(assembly);
    visitor
        .replace_pseudo_registers()
        .rewrite_mov()
        .rewrite_binop()
        .rewrite_cmp()
        .allocate_stack();

    let program = visitor.modify_program();

    write_to_file(file_name, &program.format()).expect("Should write to program file");

    compile_assembly(file_name).expect("Should compile assembly code");
    let status = grab(file_name).expect("Should grab status code");

    clean_files(file_name).expect("Cleaning files");
    assert_eq!(status, 3);
}

#[test]
fn test_if_statement() {
    let file_name = "if_1";
    let mut assembly = Assembly::from(PathBuf::from("playground/test_if6.c"));
    assembly.parse_program();
    let mut visitor = AssemblyPass::build(assembly);
    visitor
        .replace_pseudo_registers()
        .rewrite_mov()
        .rewrite_binop()
        .rewrite_cmp()
        .allocate_stack();

    let program = visitor.modify_program();

    write_to_file(file_name, &program.format()).expect("Should write to program file");

    compile_assembly(file_name).expect("Should compile assembly code");
    let status = grab(file_name).expect("Should grab status code");

    clean_files(file_name).expect("Cleaning files");
    assert_eq!(status, 1);
}

#[test]
fn test_blocks() {
    let file_name = "blocks";
    let mut assembly = Assembly::from(PathBuf::from("playground/test_blocks.c"));
    assembly.parse_program();
    let mut visitor = AssemblyPass::build(assembly);
    visitor
        .replace_pseudo_registers()
        .rewrite_mov()
        .rewrite_binop()
        .rewrite_cmp()
        .allocate_stack();

    let program = visitor.modify_program();

    write_to_file(file_name, &program.format()).expect("Should write to program file");

    compile_assembly(file_name).expect("Should compile assembly code");
    let status = grab(file_name).expect("Should grab status code");

    clean_files(file_name).expect("Cleaning files");
    assert_eq!(status, 3);
}
