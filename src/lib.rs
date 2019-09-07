extern crate rust_simple_stack_processor;

use rust_forth_tokenizer::ForthToken;
use rust_forth_tokenizer::ForthTokenizer;
pub use rust_simple_stack_processor::GasLimit;
use rust_simple_stack_processor::Opcode;
use rust_simple_stack_processor::StackMachine;

mod error;

pub use error::ForthError;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::convert::TryInto;

// This macro lets you statically initialize a hashmap
macro_rules! hashmap {
    ($( $key: expr => $val: expr ),*) => {{
         let mut map = ::std::collections::HashMap::new();
         $( map.insert($key, $val); )*
         map
    }}
}

pub struct ForthCompiler {
    // This is the Stack Machine processor that runs the compiled Forth instructions
    pub sm: StackMachine,
    // These are the words that we know how to work with regardless, things like POP, MUL, etc
    intrinsic_words: HashMap<&'static str, Vec<Opcode>>,
    // This is where we remember where we put compiled words in the *memory* of the StackMachine
    // We run the interactive opcodes after these compiled words, and then erase the memory after
    // the compiled words again for the next batch of interactive opcodes.
    word_addresses: HashMap<String, usize>,
    // This is the location in memory that points to the location after the last compiled opcode
    // So its an ideal place to run interactive compiled opcodes
    last_function: usize,
}

impl ForthCompiler {
    pub fn new() -> ForthCompiler {
        ForthCompiler {
            sm: StackMachine::new(),
            intrinsic_words: hashmap![
            "POP" => vec![Opcode::POP],
            "SWAP" => vec![Opcode::SWAP],
            "ADD" => vec![Opcode::ADD],
            "SUB" => vec![Opcode::SUB],
            "MUL" => vec![Opcode::MUL],
            "DIV" => vec![Opcode::DIV],
            "DUP" => vec![Opcode::DUP],
            "TRAP" => vec![Opcode::TRAP],
            "INC" => vec![Opcode::LDI(1),Opcode::ADD],
            "DEC" => vec![Opcode::LDI(-1),Opcode::ADD]
            ],
            word_addresses: HashMap::new(),
            last_function: 0,
        }
    }
}

// This struct tracks information for Forth IF statements
#[derive(Debug)]
struct DeferredIfStatement {
    if_location: usize,
    else_location: Option<usize>,
}

impl DeferredIfStatement {
    pub fn new(if_location: usize) -> DeferredIfStatement {
        DeferredIfStatement {
            if_location: if_location,
            else_location: None,
        }
    }
}

impl ForthCompiler {
    fn compile_tokens_compile_and_remove_word_definitions(
        &mut self,
        token_source: &ForthTokenizer,
    ) -> Result<Vec<Opcode>, ForthError> {
        // This is the interactive compiled token list
        let mut tvi = Vec::new();

        // Because we consume tokens in an inner loop, we can't use the normal for loop to read the tokens
        let mut iter = token_source.into_iter();
        while let Some(token) = iter.next() {
            match token {
                // If a colon token, then compile the word definition
                ForthToken::Colon => {
                    // Get the next token which has to be a command token, or its an error, this token will be the name to compile to
                    if let Some(ForthToken::Command(word_name)) = iter.next() {
                        // This is the list of tokens we will be compiling
                        let mut tvc = Vec::new();
                        let mut found_semicolon = false;
                        // Because this is an inner loop using the outer iterator, we can't use the normal for loop syntax
                        while let Some(token) = iter.next() {
                            match token {
                                ForthToken::SemiColon => {
                                    // We have found the end of the word definition, so compile to opcodes and put into memory...
                                    self.compile_tokens_as_word(word_name, &tvc)?;
                                    found_semicolon = true;
                                    break;
                                }
                                _ => tvc.push(token),
                            }
                        }
                        if !found_semicolon {
                            return Err(ForthError::MissingSemicolonAfterColon);
                        }
                    } else {
                        // The command token has to be right after the colon token, we don't permit things like comments, we could though...
                        return Err(ForthError::MissingCommandAfterColon);
                    }
                }
                ForthToken::SemiColon => {
                    return Err(ForthError::SemicolonBeforeColon);
                }
                _ => {
                    tvi.push(token);
                }
            }
        }

        let mut compiled_tokens = self.compile_token_vector(&tvi)?;

        // We need to return after running the interactive opcodes, so put the return in now
        compiled_tokens.push(Opcode::RET);

        Ok(compiled_tokens)
    }

    fn compile_tokens_as_word(
        &mut self,
        word_name: &str,
        tokens: &[ForthToken],
    ) -> Result<(), ForthError> {
        // Remove anything extraneous from the end of the opcode array (*processor memory*),
        // typically previous immediate mode tokens
        self.sm.st.opcodes.resize(self.last_function, Opcode::NOP);

        // Get the compiled assembler from the token vector
        let mut compiled = self.compile_token_vector(tokens)?;
        // Put the return OpCode onto the end
        compiled.push(Opcode::RET);
        // The current function start is the end of the last function
        let function_start = self.last_function;
        // Move last function pointer
        self.last_function += compiled.len();
        // Add the function to the opcode memory
        self.sm.st.opcodes.append(&mut compiled);
        // Remember where to find it...
        self.word_addresses
            .insert(word_name.to_owned(), function_start);
        //        println!("Token Memory {:?}", self.sm.st.opcodes);
        //        println!("Word Addresses {:?}", self.word_addresses);
        //        println!("Last function {}", self.last_function);
        Ok(())
    }

    fn compile_token_vector(
        &mut self,
        token_vector: &[ForthToken],
    ) -> Result<Vec<Opcode>, ForthError> {
        // Stack of if statements, they are deferred until the THEN Forth word
        let mut deferred_if_statements = Vec::new();
        // List of compiled processor opcodes that we are building up
        let mut tv: Vec<Opcode> = Vec::new();

        // Go through all the Forth tokens and turn them into processor Opcodes (for our StackMachine emulated processor)
        for t in token_vector.iter() {
            match t {
                ForthToken::DropLineComment(_) => (),
                ForthToken::ParenthesizedRemark(_) => (),
                ForthToken::StringToken(_) => (),
                ForthToken::Number(n) => {
                    // Numbers get pushed as a LDI opcode
                    tv.push(Opcode::LDI(*n));
                }
                ForthToken::Command(s) => {
                    // Remember where we are in the list of opcodes in case we hit a IF statement, LOOP etc...
                    let current_instruction = tv.len();

                    match s.as_ref() {
                        "IF" => {
                            deferred_if_statements
                                .push(DeferredIfStatement::new(current_instruction));
                            //println!("(IF)Deferred If Stack {:?}", deferred_if_statements);
                            tv.push(Opcode::LDI(0));
                            tv.push(Opcode::JRNZ);
                        }
                        "ELSE" => {
                            if let Some(x) = deferred_if_statements.last_mut() {
                                x.else_location = Some(current_instruction);
                                //println!("(ELSE) Deferred If Stack {:?}", deferred_if_statements);
                                tv.push(Opcode::LDI(0));
                                tv.push(Opcode::JR);
                            } else {
                                return Err(ForthError::InvalidSyntax(
                                    "ELSE without IF".to_owned(),
                                ));
                            }
                        }
                        "THEN" => {
                            // This only works if there isn't an ELSE statement, it needs to jump differently if there is an ELSE statement
                            //println!("(THEN) Deferred If Stack {:?}", deferred_if_statements);
                            if let Some(x) = deferred_if_statements.pop() {
                                //println!("(if let Some(x)) Deferred If Stack {:?}", x);
                                let if_jump_location = x.if_location;
                                let if_jump_offset = match x.else_location {
                                    None => (current_instruction as u64
                                        - (x.if_location + 1) as u64)
                                        .try_into()
                                        .unwrap(),
                                    Some(el) => (current_instruction as u64 - el as u64 + 1)
                                        .try_into()
                                        .unwrap(),
                                };
                                let (else_jump_location, else_jump_offset): (
                                    Option<usize>,
                                    Option<i64>,
                                ) = match x.else_location {
                                    Some(x) => (
                                        Some(x),
                                        Some(
                                            i64::try_from(
                                                current_instruction as u64 - (x + 1) as u64,
                                            )
                                            .unwrap(),
                                        ),
                                    ),
                                    None => (None, None),
                                };
                                //println!("if structure: {:?}", x);
                                tv[if_jump_location] = Opcode::LDI(if_jump_offset);
                                if let (Some(location), Some(offset)) =
                                    (else_jump_location, else_jump_offset)
                                {
                                    tv[location] = Opcode::LDI(offset);
                                }
                            } else {
                                return Err(ForthError::InvalidSyntax(
                                    "THEN without IF".to_owned(),
                                ));
                            }
                        }
                        _ => {
                            if let Some(offset) = self.word_addresses.get(*s) {
                                tv.push(Opcode::LDI(*offset as i64));
                                tv.push(Opcode::CALL);
                            } else {
                                if let Some(ol) = self.intrinsic_words.get::<str>(s) {
                                    tv.append(&mut ol.clone());
                                } else {
                                    return Err(ForthError::UnknownToken(s.to_string()));
                                }
                            }
                        }
                    }
                }
                ForthToken::Colon => {
                    panic!("Colon should never reach this function");
                }
                ForthToken::SemiColon => {
                    panic!("SemiColon should never reach this function");
                }
            }
        }

        //println!("Compiled Codes {:?}", tv);
        //println!("Total size of Codes {:?}", tv.len());
        return Ok(tv);
    }

    fn execute_tokens(
        &mut self,
        token_source: &ForthTokenizer,
        gas_limit: GasLimit,
    ) -> Result<(), ForthError> {
        let mut ol = self.compile_tokens_compile_and_remove_word_definitions(token_source)?;
        //println!("Compiled Opcodes: {:?}", ol);
        self.sm.st.opcodes.resize(self.last_function, Opcode::NOP);
        self.sm.st.opcodes.append(&mut ol);
        self.sm.execute(self.last_function, gas_limit)?;
        //println!("Total opcodes defined: {}", self.sm.st.opcodes.len());
        //println!("Total opcodes executed: {}", self.sm.st.gas_used());

        Ok(())
    }

    pub fn execute_string(&mut self, s: &str, gas_limit: GasLimit) -> Result<(), ForthError> {
        let tokenizer = ForthTokenizer::new(&s);
        self.execute_tokens(&tokenizer, gas_limit)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    extern crate rust_simple_stack_processor;

    use rust_simple_stack_processor::StackMachineError;
    use rust_simple_stack_processor::TrapHandled;
    use rust_simple_stack_processor::TrapHandler;

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
}
