use nous::{
    ast::{BinaryOperator, Identifier},
    tac::{self, Instruction, Val},
    utils::tac_from_path,
};

#[test]
fn test_binary_op() {
    let mut tac = tac_from_path("tests/files/nested_binaryop.c");
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

#[test]
fn test_if_statement() {
    let mut tac = tac_from_path("playground/test_if5.c");
    let program = tac.to_tac_program();

    let expected_instructions: tac::Instructions = vec![
        Instruction::Binary {
            binary_operator: BinaryOperator::GreaterThan,
            src_1: Val::Constant(2),
            src_2: Val::Constant(1),
            dst: Val::Var("tmp.1".into()),
        },
        Instruction::JumpIfZero {
            condition: Val::Var("tmp.1".into()),
            target: "else2".into(),
        },
        Instruction::Return(Val::Constant(3)),
        Instruction::Jump {
            target: "end1".into(),
        },
        Instruction::Label("else2".into()),
        Instruction::Binary {
            binary_operator: BinaryOperator::GreaterThan,
            src_1: Val::Constant(1),
            src_2: Val::Constant(1),
            dst: Val::Var("tmp.2".into()),
        },
        Instruction::JumpIfZero {
            condition: Val::Var("tmp.2".into()),
            target: "else4".into(),
        },
        Instruction::Return(Val::Constant(2)),
        Instruction::Jump {
            target: "end3".into(),
        },
        Instruction::Label("else4".into()),
        Instruction::Return(Val::Constant(1)),
        Instruction::Label("end3".into()),
        Instruction::Label("end1".into()),
    ];

    assert_eq!(expected_instructions, program.0.body);
}
