use nous::{
    ast::{BinaryOperator, Identifier},
    tac::{Instruction, Val},
    utils::tac_from_file,
};

#[test]
fn test_binary_op() {
    let mut tac = tac_from_file("tests/files/nested_binaryop.c");
    let program = tac.to_tac_program();

    let expected_instructions = vec![
        Instruction::Binary {
            binary_operator: BinaryOperator::Subtract,
            src_1: Val::Constant(4),
            src_2: Val::Constant(2),
            dst: Val::Var(Identifier(String::from("tmp.1"))),
        },
        Instruction::Binary {
            binary_operator: BinaryOperator::Add,
            src_1: Val::Var(Identifier(String::from("tmp.1"))),
            src_2: Val::Constant(2),
            dst: Val::Var(Identifier(String::from("tmp.2"))),
        },
        Instruction::Binary {
            binary_operator: BinaryOperator::Subtract,
            src_1: Val::Var(Identifier(String::from("tmp.2"))),
            src_2: Val::Constant(3),
            dst: Val::Var(Identifier(String::from("tmp.3"))),
        },
        Instruction::Return(Val::Var(Identifier(String::from("tmp.3")))),
    ];

    assert_eq!(expected_instructions, program.0.body);
}
