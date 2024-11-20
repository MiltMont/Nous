use nous::{errors::Result, utils::parser_from_path, visitor::VariableResolution};

#[test]
// TODO: Improve error testing: Which error variant?
fn test_undeclared_ident() -> Result<()> {
    let mut parser = parser_from_path("playground/test_undeclared.c");

    let mut verify = VariableResolution::from(parser.to_ast_program()?);
    assert!(verify.get_updated_block_items().is_err());

    Ok(())
}

#[test]
fn test_duplicate_declaration() -> Result<()> {
    let mut parser = parser_from_path("playground/test_undeclared.c");

    let mut verify = VariableResolution::from(parser.to_ast_program()?);

    assert!(verify.get_updated_block_items().is_err());
    Ok(())
}

#[test]
fn test_invalid_lval() -> Result<()> {
    let mut parser = parser_from_path("playground/test_invalid_lvalue.c");
    let mut verify = VariableResolution::from(parser.to_ast_program()?);

    assert!(verify.get_updated_block_items().is_err());
    Ok(())
}
