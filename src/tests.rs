use super::*;

extern crate rust_simple_stack_processor;

use rust_simple_stack_processor::StackMachineError;
use rust_simple_stack_processor::TrapHandled;
use rust_simple_stack_processor::TrapHandler;

#[test]
fn test_intrinsics_one_plus() {
    let tokenizer = ForthTokenizer::new("1+");
    let mut fc = ForthCompiler::default();
    let ol = fc
        .compile_tokens_compile_and_remove_word_definitions(&tokenizer)
        .unwrap();
    assert_eq!(&ol, &vec![Opcode::LDI(1), Opcode::ADD, Opcode::RET]);
}

#[test]
fn test_intrinsics_one_minus() {
    let tokenizer = ForthTokenizer::new("1-");
    let mut fc = ForthCompiler::default();
    let ol = fc
        .compile_tokens_compile_and_remove_word_definitions(&tokenizer)
        .unwrap();
    assert_eq!(&ol, &vec![Opcode::LDI(-1), Opcode::ADD, Opcode::RET]);
}

#[test]
fn test_intrinsics_two_plus() {
    let tokenizer = ForthTokenizer::new("2+");
    let mut fc = ForthCompiler::default();
    let ol = fc
        .compile_tokens_compile_and_remove_word_definitions(&tokenizer)
        .unwrap();
    assert_eq!(&ol, &vec![Opcode::LDI(2), Opcode::ADD, Opcode::RET]);
}

#[test]
fn test_intrinsics_two_minus() {
    let tokenizer = ForthTokenizer::new("2-");
    let mut fc = ForthCompiler::default();
    let ol = fc
        .compile_tokens_compile_and_remove_word_definitions(&tokenizer)
        .unwrap();
    assert_eq!(&ol, &vec![Opcode::LDI(-2), Opcode::ADD, Opcode::RET]);
}

#[test]
fn test_intrinsics_two_minus_run() {
    let mut fc = ForthCompiler::default();

    fc.execute_string("10 2-", GasLimit::Limited(100)).unwrap();

    assert_eq!(&fc.sm.st.number_stack, &vec![8_i64]);
}

#[test]
fn test_intrinsics_two_mul() {
    let tokenizer = ForthTokenizer::new("2*");
    let mut fc = ForthCompiler::default();
    let ol = fc
        .compile_tokens_compile_and_remove_word_definitions(&tokenizer)
        .unwrap();
    assert_eq!(&ol, &vec![Opcode::LDI(2), Opcode::MUL, Opcode::RET]);
}

#[test]
fn test_intrinsics_two_div() {
    let tokenizer = ForthTokenizer::new("2/");
    let mut fc = ForthCompiler::default();
    let ol = fc
        .compile_tokens_compile_and_remove_word_definitions(&tokenizer)
        .unwrap();
    assert_eq!(&ol, &vec![Opcode::LDI(2), Opcode::DIV, Opcode::RET]);
}

#[test]
fn test_intrinsics_two_div_run() {
    let mut fc = ForthCompiler::default();

    fc.execute_string("10 2/", GasLimit::Limited(100)).unwrap();

    assert_eq!(&fc.sm.st.number_stack, &vec![5_i64]);
}

#[test]
fn test_intrinsics_dup() {
    let tokenizer = ForthTokenizer::new("10 2 DUP");
    let mut fc = ForthCompiler::default();
    let ol = fc
        .compile_tokens_compile_and_remove_word_definitions(&tokenizer)
        .unwrap();
    assert_eq!(&ol, &vec![Opcode::LDI(10),Opcode::LDI(2), Opcode::DUP, Opcode::RET]);
}

#[test]
fn test_intrinsics_dup_run() {
    let mut fc = ForthCompiler::default();

    fc.execute_string("10 2 DUP", GasLimit::Limited(100)).unwrap();

    assert_eq!(&fc.sm.st.number_stack, &vec![10_i64,2,2]);
}

#[test]
fn test_intrinsics_two_dup() {
    let tokenizer = ForthTokenizer::new("10 2 2DUP");
    let mut fc = ForthCompiler::default();
    let ol = fc
        .compile_tokens_compile_and_remove_word_definitions(&tokenizer)
        .unwrap();
    assert_eq!(&ol, &vec![Opcode::LDI(10),Opcode::LDI(2), Opcode::DUP2, Opcode::RET]);
}

#[test]
fn test_intrinsics_two_dup_run() {
    let mut fc = ForthCompiler::default();

    fc.execute_string("10 2 2DUP", GasLimit::Limited(100)).unwrap();

    assert_eq!(&fc.sm.st.number_stack, &vec![10_i64,2,10,2]);
}

#[test]
fn test_intrinsics_drop() {
    let tokenizer = ForthTokenizer::new("10 2 DROP");
    let mut fc = ForthCompiler::default();
    let ol = fc
        .compile_tokens_compile_and_remove_word_definitions(&tokenizer)
        .unwrap();
    assert_eq!(&ol, &vec![Opcode::LDI(10),Opcode::LDI(2), Opcode::DROP, Opcode::RET]);
}

#[test]
fn test_intrinsics_drop_run() {
    let mut fc = ForthCompiler::default();

    fc.execute_string("10 2 DROP", GasLimit::Limited(100)).unwrap();

    assert_eq!(&fc.sm.st.number_stack, &vec![10_i64]);
}

#[test]
fn test_intrinsics_two_drop() {
    let tokenizer = ForthTokenizer::new("10 2 2DROP");
    let mut fc = ForthCompiler::default();
    let ol = fc
        .compile_tokens_compile_and_remove_word_definitions(&tokenizer)
        .unwrap();
    assert_eq!(&ol, &vec![Opcode::LDI(10),Opcode::LDI(2), Opcode::DROP,Opcode::DROP, Opcode::RET]);
}

#[test]
fn test_intrinsics_two_drop_run() {
    let mut fc = ForthCompiler::default();

    fc.execute_string("3 10 2 2DROP", GasLimit::Limited(100)).unwrap();

    assert_eq!(&fc.sm.st.number_stack, &vec![3_i64]);
}

#[test]
fn test_intrinsics_two_over() {
    let tokenizer = ForthTokenizer::new("1 2 3 4 2OVER");
    let mut fc = ForthCompiler::default();
    let ol = fc
        .compile_tokens_compile_and_remove_word_definitions(&tokenizer)
        .unwrap();
    assert_eq!(&ol, &vec![Opcode::LDI(1),Opcode::LDI(2),Opcode::LDI(3),Opcode::LDI(4), Opcode::OVER2, Opcode::RET]);
}

#[test]
fn test_intrinsics_two_over_run() {
    let mut fc = ForthCompiler::default();

    fc.execute_string("1 2 3 4 2OVER", GasLimit::Limited(100)).unwrap();

    assert_eq!(&fc.sm.st.number_stack, &vec![1_i64,2,3,4,1,2]);
}

#[test]
fn test_intrinsics_two_swap() {
    let tokenizer = ForthTokenizer::new("1 2 3 4 2SWAP");
    let mut fc = ForthCompiler::default();
    let ol = fc
        .compile_tokens_compile_and_remove_word_definitions(&tokenizer)
        .unwrap();
    assert_eq!(&ol, &vec![Opcode::LDI(1),Opcode::LDI(2),Opcode::LDI(3),Opcode::LDI(4), Opcode::SWAP2, Opcode::RET]);
}

#[test]
fn test_intrinsics_two_swap_run() {
    let mut fc = ForthCompiler::default();

    fc.execute_string("1 2 3 4 2SWAP", GasLimit::Limited(100)).unwrap();

    assert_eq!(&fc.sm.st.number_stack, &vec![3_i64,4,1,2]);
}
#[test]
fn test_i() {
    let tokenizer = ForthTokenizer::new("I");
    let mut fc = ForthCompiler::default();
    let ol = fc
        .compile_tokens_compile_and_remove_word_definitions(&tokenizer)
        .unwrap();
    assert_eq!(&ol, &vec![Opcode::GETLP, Opcode::RET]);
}

#[test]
fn test_j() {
    let tokenizer = ForthTokenizer::new("J");
    let mut fc = ForthCompiler::default();
    let ol = fc
        .compile_tokens_compile_and_remove_word_definitions(&tokenizer)
        .unwrap();
    assert_eq!(&ol, &vec![Opcode::GETLP2, Opcode::RET]);
}

#[test]
fn test_do_loop() {
    let tokenizer = ForthTokenizer::new("10 0 DO 123 LEAVE 456 LOOP");
    let mut fc = ForthCompiler::default();
    let ol = fc
        .compile_tokens_compile_and_remove_word_definitions(&tokenizer)
        .unwrap();
    assert_eq!(
        &ol,
        &vec![
            Opcode::LDI(10),
            Opcode::LDI(0),
            Opcode::PUSHLP,
            Opcode::LDI(123),
            Opcode::LDI(6),
            Opcode::JR,
            Opcode::LDI(456),
            Opcode::INCLP,
            Opcode::CMPLOOP,
            Opcode::LDI(-7),
            Opcode::JRZ,
            Opcode::DROPLP,
            Opcode::RET
        ]
    );
}

#[test]
fn test_do_loop_simple() {
    let tokenizer = ForthTokenizer::new("10 0 DO I LOOP");
    let mut fc = ForthCompiler::default();
    let ol = fc
        .compile_tokens_compile_and_remove_word_definitions(&tokenizer)
        .unwrap();
    assert_eq!(
        &ol,
        &vec![
            Opcode::LDI(10),
            Opcode::LDI(0),
            Opcode::PUSHLP,
            Opcode::GETLP,
            Opcode::INCLP,
            Opcode::CMPLOOP,
            Opcode::LDI(-4),
            Opcode::JRZ,
            Opcode::DROPLP,
            Opcode::RET
        ]
    );
}

#[test]
fn test_do_loop_simple_run_1() {
    let mut fc = ForthCompiler::default();

    fc.execute_string("10 0 DO I LOOP", GasLimit::Limited(250))
        .unwrap();

    assert_eq!(
        &fc.sm.st.number_stack,
        &vec![0_i64, 1, 2, 3, 4, 5, 6, 7, 8, 9]
    );
}

#[test]
fn test_do_plus_loop() {
    let tokenizer = ForthTokenizer::new("10 0 DO 123 LEAVE 456 2 +LOOP");
    let mut fc = ForthCompiler::default();
    let ol = fc
        .compile_tokens_compile_and_remove_word_definitions(&tokenizer)
        .unwrap();
    assert_eq!(
        &ol,
        &vec![
            Opcode::LDI(10),
            Opcode::LDI(0),
            Opcode::PUSHLP,
            Opcode::LDI(123),
            Opcode::LDI(7),
            Opcode::JR,
            Opcode::LDI(456),
            Opcode::LDI(2),
            Opcode::ADDLP,
            Opcode::CMPLOOP,
            Opcode::LDI(-8),
            Opcode::JRZ,
            Opcode::DROPLP,
            Opcode::RET
        ]
    );
}

#[test]
fn test_do_plus_loop_simple_run_1() {
    let mut fc = ForthCompiler::default();

    fc.execute_string("10 0 DO I 2 +LOOP", GasLimit::Limited(250))
        .unwrap();

    assert_eq!(&fc.sm.st.number_stack, &vec![0_i64, 2, 4, 6, 8]);
}

#[test]
fn test_do_loop_compound_run_1() {
    let mut fc = ForthCompiler::default();

    fc.execute_string(
        "100 10 DO 10 0 DO J I ADD LOOP 10 +LOOP",
        GasLimit::Limited(1000),
    )
    .unwrap();

    assert_eq!(
        &fc.sm.st.number_stack,
        &vec![
            10_i64, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20_i64, 21, 22, 23, 24, 25, 26, 27, 28, 29,
            30_i64, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40_i64, 41, 42, 43, 44, 45, 46, 47, 48, 49,
            50_i64, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60_i64, 61, 62, 63, 64, 65, 66, 67, 68, 69,
            70_i64, 71, 72, 73, 74, 75, 76, 77, 78, 79, 80_i64, 81, 82, 83, 84, 85, 86, 87, 88, 89,
            90_i64, 91, 92, 93, 94, 95, 96, 97, 98, 99,
        ]
    );
}

#[test]
fn test_begin_while_repeat() {
    let tokenizer = ForthTokenizer::new("BEGIN 123 WHILE 456 REPEAT");
    let mut fc = ForthCompiler::default();
    let ol = fc
        .compile_tokens_compile_and_remove_word_definitions(&tokenizer)
        .unwrap();
    assert_eq!(
        &ol,
        &vec![
            Opcode::LDI(123),
            Opcode::LDI(4),
            Opcode::JRZ,
            Opcode::LDI(456),
            Opcode::LDI(-5),
            Opcode::JR,
            Opcode::RET
        ]
    );
}

#[test]
fn test_begin_while_repeat_run() {
    let mut fc = ForthCompiler::default();

    fc.execute_string("10 BEGIN 1- DUP WHILE REPEAT", GasLimit::Limited(100))
        .unwrap();

    assert_eq!(&fc.sm.st.number_stack, &vec![0_i64]);
}

#[test]
fn test_begin_while_leave_repeat_run() {
    let mut fc = ForthCompiler::default();

    fc.execute_string(
        "10 BEGIN 1- DUP 5 SUB NOT IF LEAVE THEN DUP WHILE REPEAT",
        GasLimit::Limited(100),
    )
    .unwrap();

    assert_eq!(&fc.sm.st.number_stack, &vec![5_i64]);
}

#[test]
fn test_begin_until() {
    let tokenizer = ForthTokenizer::new("10 BEGIN 0 IF LEAVE THEN 1- DUP NOT UNTIL");
    let mut fc = ForthCompiler::default();
    let ol = fc
        .compile_tokens_compile_and_remove_word_definitions(&tokenizer)
        .unwrap();
    assert_eq!(
        &ol,
        &vec![
            Opcode::LDI(10),
            Opcode::LDI(0),
            Opcode::LDI(3),
            Opcode::JRZ,
            Opcode::LDI(7),
            Opcode::JR,
            Opcode::LDI(-1),
            Opcode::ADD,
            Opcode::DUP,
            Opcode::NOT,
            Opcode::LDI(-10),
            Opcode::JRZ,
            Opcode::RET
        ]
    );
}

#[test]
fn test_begin_until_run() {
    let mut fc = ForthCompiler::default();

    fc.execute_string(
        "10 BEGIN 0 IF LEAVE THEN 1- DUP NOT UNTIL",
        GasLimit::Limited(250),
    )
    .unwrap();

    assert_eq!(&fc.sm.st.number_stack, &vec![0_i64]);
}

#[test]
fn test_begin_again() {
    let tokenizer = ForthTokenizer::new("10 BEGIN 1- DUP NOT IF LEAVE THEN AGAIN");
    let mut fc = ForthCompiler::default();
    let ol = fc
        .compile_tokens_compile_and_remove_word_definitions(&tokenizer)
        .unwrap();
    // Currently this assert is all wrong, it has to be updated for the changes to the test
    assert_eq!(
        &ol,
        &vec![
            Opcode::LDI(10),
            Opcode::LDI(-1),
            Opcode::ADD,
            Opcode::DUP,
            Opcode::NOT,
            Opcode::LDI(3),
            Opcode::JRZ,
            Opcode::LDI(3),
            Opcode::JR,
            Opcode::LDI(-9),
            Opcode::JR,
            Opcode::RET
        ]
    );
}

#[test]
fn test_begin_again_run() {
    let mut fc = ForthCompiler::default();

    fc.execute_string(
        "10 BEGIN 1- DUP NOT IF LEAVE THEN AGAIN",
        GasLimit::Limited(250),
    )
    .unwrap();

    assert_eq!(&fc.sm.st.number_stack, &vec![0_i64]);
}

#[test]
fn test_begin_again_leave() {
    let tokenizer = ForthTokenizer::new("BEGIN 123 LEAVE 456 LEAVE 789 AGAIN");
    let mut fc = ForthCompiler::default();
    let ol = fc
        .compile_tokens_compile_and_remove_word_definitions(&tokenizer)
        .unwrap();
    assert_eq!(
        &ol,
        &vec![
            Opcode::LDI(123),
            Opcode::LDI(7),
            Opcode::JR,
            Opcode::LDI(456),
            Opcode::LDI(4),
            Opcode::JR,
            Opcode::LDI(789),
            Opcode::LDI(-8),
            Opcode::JR,
            Opcode::RET
        ]
    );
}

#[test]
fn test_execute_intrinsics_1() {
    let mut fc = ForthCompiler::default();

    fc.execute_string("123 321 ADD 2 MUL", GasLimit::Limited(100))
        .unwrap();

    assert_eq!(&fc.sm.st.number_stack, &vec![888_i64]);

    fc.execute_string("123 321 ADD 2 MUL", GasLimit::Limited(100))
        .unwrap();

    assert_eq!(&fc.sm.st.number_stack, &vec![888_i64, 888]);
}
#[test]
fn test_compile_1() {
    let mut fc = ForthCompiler::default();

    fc.execute_string(
        ": RickTest 123 321 ADD 2 MUL ; RickTest",
        GasLimit::Limited(100),
    )
    .unwrap();

    assert_eq!(&fc.sm.st.number_stack, &vec![888_i64]);

    fc.execute_string("123 321 ADD 2 MUL RickTest", GasLimit::Limited(100))
        .unwrap();

    assert_eq!(&fc.sm.st.number_stack, &vec![888_i64, 888, 888]);
}

#[test]
fn test_compile_2() {
    let mut fc = ForthCompiler::default();

    fc.execute_string(
        ": RickTest 123 321 ADD 2 MUL ; RickTest : RickTestB 123 321 ADD 2 MUL ;",
        GasLimit::Limited(100),
    )
    .unwrap();

    assert_eq!(&fc.sm.st.number_stack, &vec![888_i64]);

    fc.execute_string("123 321 ADD 2 MUL RickTest", GasLimit::Limited(100))
        .unwrap();

    assert_eq!(&fc.sm.st.number_stack, &vec![888_i64, 888, 888]);
}

#[test]
fn test_compile_3() {
    let mut fc = ForthCompiler::default();

    fc.execute_string(
            "2 2 SUB DROP : RickTest 123 321 ADD 2 MUL ; RickTest : RickTestB 123 321 ADD 2 MUL ; 3 3 SUB DROP",
            GasLimit::Limited(100),
        )
        .unwrap();

    assert_eq!(&fc.sm.st.number_stack, &vec![888_i64]);

    fc.execute_string("123 321 ADD 2 MUL RickTest", GasLimit::Limited(100))
        .unwrap();

    assert_eq!(&fc.sm.st.number_stack, &vec![888_i64, 888, 888]);
}

#[test]
fn test_compile_4() {
    let mut fc = ForthCompiler::default();

    fc.execute_string(
        "2 2 SUB DROP : RickTest 123 321 ADD 2 MUL ; : RickTestB 123 321 ADD 2 MUL ; 3 3 SUB",
        GasLimit::Limited(100),
    )
    .unwrap();

    assert_eq!(&fc.sm.st.number_stack, &vec![0_i64]);

    fc.execute_string("123 321 ADD 2 MUL RickTest", GasLimit::Limited(100))
        .unwrap();

    assert_eq!(&fc.sm.st.number_stack, &vec![0_i64, 888, 888]);
}

#[test]
fn test_compile_fail_1() {
    let mut fc = ForthCompiler::default();

    match fc.execute_string(
        "2 2 SUB DROP : RickTest 123 321 ADD 2 MUL ; : : RickTestB 123 321 ADD 2 MUL ; 3 3 SUB",
        GasLimit::Limited(100),
    ) {
        Err(ForthError::MissingCommandAfterColon) => (),
        r => panic!("Incorrect error type returned {:?}", r),
    }
}

#[test]
fn test_compile_fail_2() {
    let mut fc = ForthCompiler::default();

    match fc.execute_string(
        "2 2 SUB DROP : RickTest 123 321 ADD 2 MUL ; ; : RickTestB 123 321 ADD 2 MUL ; 3 3 SUB",
        GasLimit::Limited(100),
    ) {
        Err(ForthError::SemicolonBeforeColon) => (),
        r => panic!("Incorrect error type returned {:?}", r),
    }
}

#[test]
fn test_compile_fail_3() {
    let mut fc = ForthCompiler::default();

    match fc.execute_string(
            "2 2 SUB DROP : RickTest 123 321 ADD 2 MUL ; : RickTestB 123 321 ADD 2 MUL ; : ERROR 3 3 SUB",
            GasLimit::Limited(100),
        ) {
            Err(ForthError::MissingSemicolonAfterColon) => (),
            r => panic!("Incorrect error type returned {:?}", r),
        }
}

#[test]
fn test_if_else_1() {
    let mut fc = ForthCompiler::default();

    fc.execute_string(
        "1 2 3 DROP DROP DROP -7 IF 1 2 ADD ELSE 3 4 ADD THEN",
        GasLimit::Limited(100),
    )
    .unwrap();

    assert_eq!(&fc.sm.st.number_stack, &vec![3_i64]);
}

#[test]
fn test_if_else_2() {
    let mut fc = ForthCompiler::default();

    fc.execute_string(
        "1 2 3 DROP DROP DROP 0 IF 1 2 ADD ELSE 3 4 ADD THEN",
        GasLimit::Limited(100),
    )
    .unwrap();

    assert_eq!(&fc.sm.st.number_stack, &vec![7_i64]);
}

#[test]
fn test_if_else_3() {
    let mut fc = ForthCompiler::default();

    fc.execute_string("1 IF 1 2 ADD ELSE 3 4 ADD THEN", GasLimit::Limited(100))
        .unwrap();

    assert_eq!(&fc.sm.st.number_stack, &vec![3_i64]);
}

#[test]
fn test_if_else_4() {
    let mut fc = ForthCompiler::default();

    fc.execute_string("0 IF 1 2 ADD ELSE 3 4 ADD THEN", GasLimit::Limited(100))
        .unwrap();

    assert_eq!(&fc.sm.st.number_stack, &vec![7_i64]);
}

#[test]
fn test_trap_1() {
    let mut fc = ForthCompiler::default();

    // Simulate a IO OUT command, at TRAP(100)
    fc.sm
        .trap_handlers
        .push(Box::from(TrapHandler::new(100, |_trap_id, st| {
            let io_port = st
                .number_stack
                .pop()
                .ok_or(StackMachineError::NumberStackUnderflow)?;
            let io_value = st
                .number_stack
                .pop()
                .ok_or(StackMachineError::NumberStackUnderflow)?;
            println!(
                "Simulated IO OUT command to Port: {} and Value: {}",
                io_port, io_value
            );
            Ok(TrapHandled::Handled)
        })));

    fc.execute_string(
        ": IO_OUT 100 TRAP ; 123456 1000 IO_OUT",
        GasLimit::Limited(100),
    )
    .unwrap();

    // Nothing left over
    assert_eq!(&fc.sm.st.number_stack, &vec![]);
}

#[test]
fn test_trap_2() {
    let mut fc = ForthCompiler::default();

    // Simulate a IO IN command, at TRAP(101)
    fc.sm
        .trap_handlers
        .push(Box::from(TrapHandler::new(101, |_trap_id, st| {
            let io_port = st
                .number_stack
                .pop()
                .ok_or(StackMachineError::NumberStackUnderflow)?;
            let io_value = 654321_i64;
            println!(
                "Simulated IO IN command from Port: {} and Value: {}",
                io_port, io_value
            );
            st.number_stack.push(io_value);
            Ok(TrapHandled::Handled)
        })));

    fc.execute_string(": IO_IN 101 TRAP ; 1000 IO_IN", GasLimit::Limited(100))
        .unwrap();

    // Value from IO port on stack
    assert_eq!(&fc.sm.st.number_stack, &vec![654321]);
}

#[test]
fn test_trap_3() {
    let mut fc = ForthCompiler::default();

    // Simulate a IO OUT command, at TRAP(100), but define the port number inside a Forth Word as well
    fc.sm
        .trap_handlers
        .push(Box::from(TrapHandler::new(100, |_trap_id, st| {
            let io_port = st
                .number_stack
                .pop()
                .ok_or(StackMachineError::NumberStackUnderflow)?;
            let io_value = st
                .number_stack
                .pop()
                .ok_or(StackMachineError::NumberStackUnderflow)?;
            println!(
                "Simulated IO OUT command to Port: {} and Value: {}",
                io_port, io_value
            );
            Ok(TrapHandled::Handled)
        })));

    fc.execute_string(
        ": IO_OUT 100 TRAP ; : OUT_DISPLAY 1000 IO_OUT ; 123456 OUT_DISPLAY",
        GasLimit::Limited(100),
    )
    .unwrap();

    // Nothing left over
    assert_eq!(&fc.sm.st.number_stack, &vec![]);
}

#[test]
fn test_trap_4() {
    let mut fc = ForthCompiler::default();

    // Simulate a IO IN command, at TRAP(101), but define the port number inside a Forth word as well
    fc.sm
        .trap_handlers
        .push(Box::from(TrapHandler::new(101, |_trap_id, st| {
            let io_port = st
                .number_stack
                .pop()
                .ok_or(StackMachineError::NumberStackUnderflow)?;
            let io_value = 654321_i64;
            println!(
                "Simulated IO IN command from Port: {} and Value: {}",
                io_port, io_value
            );
            st.number_stack.push(io_value);
            Ok(TrapHandled::Handled)
        })));

    fc.execute_string(
        ": IO_IN 101 TRAP ; : IN_KEYBOARD 1000 IO_IN ; IN_KEYBOARD",
        GasLimit::Limited(100),
    )
    .unwrap();

    // Value from IO port on stack
    assert_eq!(&fc.sm.st.number_stack, &vec![654321]);
}

#[test]
fn test_intrinsics_eq_run_1() {
    let mut fc = ForthCompiler::default();

    fc.execute_string("1 1 =", GasLimit::Limited(100)).unwrap();

    assert_eq!(&fc.sm.st.number_stack, &vec![-1_i64]);
}

#[test]
fn test_intrinsics_eq_run_2() {
    let mut fc = ForthCompiler::default();

    fc.execute_string("1 2 =", GasLimit::Limited(100)).unwrap();

    assert_eq!(&fc.sm.st.number_stack, &vec![0_i64]);
}

#[test]
fn test_intrinsics_ne_run_1() {
    let mut fc = ForthCompiler::default();

    fc.execute_string("1 1 <>", GasLimit::Limited(100)).unwrap();

    assert_eq!(&fc.sm.st.number_stack, &vec![0_i64]);
}

#[test]
fn test_intrinsics_ne_run_2() {
    let mut fc = ForthCompiler::default();

    fc.execute_string("1 2 <>", GasLimit::Limited(100)).unwrap();

    assert_eq!(&fc.sm.st.number_stack, &vec![-1_i64]);
}
