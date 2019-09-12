use super::*;

extern crate rust_simple_stack_processor;

use rust_simple_stack_processor::StackMachineError;
use rust_simple_stack_processor::TrapHandled;
use rust_simple_stack_processor::TrapHandler;

#[test]
fn test_begin_while_repeat() {
    let tokenizer = ForthTokenizer::new("BEGIN 123 WHILE 456 REPEAT");
    let mut fc = ForthCompiler::new();
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
            Opcode::LDI(-6),
            Opcode::JR,
            Opcode::RET
        ]
    );
}
#[test]
fn test_begin_until() {
    let tokenizer = ForthTokenizer::new("BEGIN 123 LEAVE 456 UNTIL");
    let mut fc = ForthCompiler::new();
    let ol = fc
        .compile_tokens_compile_and_remove_word_definitions(&tokenizer)
        .unwrap();
    assert_eq!(
        &ol,
        &vec![
            Opcode::LDI(123),
            Opcode::LDI(4),
            Opcode::JR,
            Opcode::LDI(456),
            Opcode::LDI(-6),
            Opcode::JRZ,
            Opcode::RET
        ]
    );
}

#[test]
fn test_begin_again() {
    let tokenizer = ForthTokenizer::new("BEGIN 123 AGAIN");
    let mut fc = ForthCompiler::new();
    let ol = fc
        .compile_tokens_compile_and_remove_word_definitions(&tokenizer)
        .unwrap();
    assert_eq!(
        &ol,
        &vec![Opcode::LDI(123), Opcode::LDI(-3), Opcode::JR, Opcode::RET]
    );
}

#[test]
fn test_begin_again_leave() {
    let tokenizer = ForthTokenizer::new("BEGIN 123 LEAVE 456 LEAVE 789 AGAIN");
    let mut fc = ForthCompiler::new();
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
            Opcode::LDI(-9),
            Opcode::JR,
            Opcode::RET
        ]
    );
}

#[test]
fn test_execute_intrinsics_1() {
    let mut fc = ForthCompiler::new();

    fc.execute_string("123 321 ADD 2 MUL", GasLimit::Limited(100))
        .unwrap();

    assert_eq!(&fc.sm.st.number_stack, &vec![888_i64]);

    fc.execute_string("123 321 ADD 2 MUL", GasLimit::Limited(100))
        .unwrap();

    assert_eq!(&fc.sm.st.number_stack, &vec![888_i64, 888]);
}
#[test]
fn test_compile_1() {
    let mut fc = ForthCompiler::new();

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
    let mut fc = ForthCompiler::new();

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
    let mut fc = ForthCompiler::new();

    fc.execute_string(
            "2 2 SUB POP : RickTest 123 321 ADD 2 MUL ; RickTest : RickTestB 123 321 ADD 2 MUL ; 3 3 SUB POP",
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
    let mut fc = ForthCompiler::new();

    fc.execute_string(
        "2 2 SUB POP : RickTest 123 321 ADD 2 MUL ; : RickTestB 123 321 ADD 2 MUL ; 3 3 SUB",
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
    let mut fc = ForthCompiler::new();

    match fc.execute_string(
        "2 2 SUB POP : RickTest 123 321 ADD 2 MUL ; : : RickTestB 123 321 ADD 2 MUL ; 3 3 SUB",
        GasLimit::Limited(100),
    ) {
        Err(ForthError::MissingCommandAfterColon) => (),
        r => panic!("Incorrect error type returned {:?}", r),
    }
}

#[test]
fn test_compile_fail_2() {
    let mut fc = ForthCompiler::new();

    match fc.execute_string(
        "2 2 SUB POP : RickTest 123 321 ADD 2 MUL ; ; : RickTestB 123 321 ADD 2 MUL ; 3 3 SUB",
        GasLimit::Limited(100),
    ) {
        Err(ForthError::SemicolonBeforeColon) => (),
        r => panic!("Incorrect error type returned {:?}", r),
    }
}

#[test]
fn test_compile_fail_3() {
    let mut fc = ForthCompiler::new();

    match fc.execute_string(
            "2 2 SUB POP : RickTest 123 321 ADD 2 MUL ; : RickTestB 123 321 ADD 2 MUL ; : ERROR 3 3 SUB",
            GasLimit::Limited(100),
        ) {
            Err(ForthError::MissingSemicolonAfterColon) => (),
            r => panic!("Incorrect error type returned {:?}", r),
        }
}

#[test]
fn test_if_else_1() {
    let mut fc = ForthCompiler::new();

    fc.execute_string(
        "1 2 3 POP POP POP 0 IF 1 2 ADD ELSE 3 4 ADD THEN",
        GasLimit::Limited(100),
    )
    .unwrap();

    assert_eq!(&fc.sm.st.number_stack, &vec![3_i64]);
}

#[test]
fn test_if_else_2() {
    let mut fc = ForthCompiler::new();

    fc.execute_string(
        "1 2 3 POP POP POP 1 IF 1 2 ADD ELSE 3 4 ADD THEN",
        GasLimit::Limited(100),
    )
    .unwrap();

    assert_eq!(&fc.sm.st.number_stack, &vec![7_i64]);
}

#[test]
fn test_if_else_3() {
    let mut fc = ForthCompiler::new();

    fc.execute_string("0 IF 1 2 ADD ELSE 3 4 ADD THEN", GasLimit::Limited(100))
        .unwrap();

    assert_eq!(&fc.sm.st.number_stack, &vec![3_i64]);
}

#[test]
fn test_if_else_4() {
    let mut fc = ForthCompiler::new();

    fc.execute_string("1 IF 1 2 ADD ELSE 3 4 ADD THEN", GasLimit::Limited(100))
        .unwrap();

    assert_eq!(&fc.sm.st.number_stack, &vec![7_i64]);
}

#[test]
fn test_trap_1() {
    let mut fc = ForthCompiler::new();

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
    let mut fc = ForthCompiler::new();

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
    let mut fc = ForthCompiler::new();

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
    let mut fc = ForthCompiler::new();

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
