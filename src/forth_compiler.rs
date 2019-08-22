extern crate rust_simple_stack_processor;

use rust_simple_stack_processor::GasLimit;
use rust_simple_stack_processor::Opcode;
use rust_simple_stack_processor::StackMachine;

use super::error::ForthError;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::convert::TryInto;

/// This Enum lists the token types that are used by the Forth interpreter
#[derive(Debug)]
pub enum Token {
    Number(i64),
    Command(String),
    Colon(String),
    SemiColon,
    End,
    Error(String),
}

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

/// This Enum determines whether the Forth interpreter is in Interpreting mode or Compiling mode
#[derive(Debug, PartialEq)]
enum Mode {
    Interpreting,
    Compiling(String),
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
    // Take a string containing Forth words and turn it into a list of Forth tokens
    fn tokenize_string(&self, s: &str) -> Result<Vec<Token>, ForthError> {
        let mut tv = Vec::new();

        let mut string_iter = s.split_whitespace();

        loop {
            match string_iter.next() {
                // If no more text in the string, then return what we have tokenized
                None => return Ok(tv),
                // If we have some text to process, then process it
                Some(string_token) => {
                    // Try to convert it to a number
                    tv.push(match string_token.parse::<i64>() {
                        // We found a number, then return it as a number token
                        Ok(n) => Token::Number(n),
                        // Wasn't a number, treat it as a *word*
                        Err(_) => match string_token {
                            // If its a colon, create a colon token
                            ":" => match &string_iter.next() {
                                // If we found a token, then we need to grab the next bit of text so we know what Forth word is being compiled
                                Some(next_token) => Token::Colon(next_token.to_string()),
                                // There has to be something after the colon, so this is an error since we didn't find anything
                                None => {
                                    return Err(ForthError::InvalidSyntax(String::from(
                                        "No token after :, but one needed to compile",
                                    )))
                                }
                            },
                            // Create a semicolon token
                            ";" => Token::SemiColon,
                            // Whatever else, assume its a Forth word
                            _ => Token::Command(string_token.to_owned()),
                        },
                    });
                }
            }
        }
    }

    fn compile_token_vector_compile_and_remove_word_definitions(
        &mut self,
        token_vector: &[Token],
    ) -> Result<Vec<Opcode>, ForthError> {
        // This is the interactive compiled token list
        let mut tvi = Vec::new();
        // This tracks whethere we are interpreting or compiling right now
        let mut mode = Mode::Interpreting;
        // This is where we start compiling the latest segment of word/interactive tokens
        let mut starting_position = 0;

        //println!(
        //    "compile_token_vector_compile_and_remove_word_definitions Compiling Forth tokens {:?}",
        //    token_vector
        //);
        // So, for every token we have been passed, check what it is...
        for i in 0..token_vector.len() {
            match &token_vector[i] {
                Token::Colon(s) => {
                    // Found Colon, so the user wants to compile a word presumably
                    match mode {
                        // If we are currently interpreting, then we can safely switch to compiling
                        Mode::Interpreting => {
                            // Make sure there is something to compile...
                            if i > starting_position {
                                // We end before the current token
                                // Compile whatever appeared before this compile statement
                                tvi.append(
                                    &mut self.compile_token_vector(
                                        &token_vector[starting_position..i],
                                    )?,
                                );
                            }
                            // Start compiling again after this token
                            starting_position = i + 1;
                            // Switch to compiling mode, remmeber the word we are trying to compile
                            mode = Mode::Compiling(String::from(s));
                        }
                        // We are already in compiling mode, so getting a colon is a syntax error
                        Mode::Compiling(_) => {
                            return Err(ForthError::InvalidSyntax(
                                "Second colon before semicolon".to_string(),
                            ));
                        }
                    }
                }
                Token::SemiColon => {
                    match mode {
                        // We are in interpreting mode, this is a syntax error
                        Mode::Interpreting => {
                            return Err(ForthError::InvalidSyntax(
                                "Semicolon before colon".to_string(),
                            ));
                        }
                        // We have found the end of the word definition, so compile to opcodes and put into memory...
                        Mode::Compiling(s) => {
                            // Remove anything extraneous from the end of the opcode array (*processor memory*),
                            // typically previous immediate mode tokens
                            self.sm.st.opcodes.resize(self.last_function, Opcode::NOP);

                            // Get the compiled assembler from the token vector
                            // stop compiling before the ending token
                            let mut compiled =
                                self.compile_token_vector(&token_vector[starting_position..i])?;
                            // Put the return OpCode onto the end
                            compiled.push(Opcode::RET);
                            // The current function start is the end of the last function
                            let function_start = self.last_function;
                            // Move last function pointer
                            self.last_function += compiled.len();
                            // Add the function to the opcode memory
                            self.sm.st.opcodes.append(&mut compiled);
                            // Remember where to find it...
                            self.word_addresses.insert(s, function_start);
                            // start compiling again after this token
                            starting_position = i + 1;
                            // Switch back to interpreting mode
                            mode = Mode::Interpreting;
                            //println!("Token Memory {:?}", self.sm.st.opcodes);
                            //println!("Word Addresses {:?}", self.word_addresses);
                            //println!("Last function {}", self.last_function);
                        }
                    }
                }
                _ => (),
            }
        }

        // Check for an error condition and report it
        // If we are not in interpreting mode when we have processed all the Forth tokens, then that's an error
        if mode != Mode::Interpreting {
            return Err(ForthError::MissingSemicolonAfterColon);
        }

        // Compile any tokens that remain after processing
        let mut compiled = self.compile_token_vector(&token_vector[starting_position..])?;
        tvi.append(&mut compiled);
        // We need to return after running the interactive opcodes, so put the return in now
        tvi.push(Opcode::RET);

        // Return the interactive tokens, the compiled ones are already in memory
        return Ok(tvi);
    }

    fn compile_token_vector(&mut self, token_vector: &[Token]) -> Result<Vec<Opcode>, ForthError> {
        // Stack of if statements, they are deferred until the THEN Forth word
        let mut deferred_if_statements = Vec::new();
        // List of compiled processor opcodes that we are building up
        let mut tv: Vec<Opcode> = Vec::new();

        // Go through all the Forth tokens and turn them into processor Opcodes (for our StackMachine emulated processor)
        for t in token_vector.iter() {
            match t {
                Token::Number(n) => {
                    // Numbers get pushed as a LDI opcode
                    tv.push(Opcode::LDI(*n));
                }
                Token::Command(s) => {
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
                            if let Some(offset) = self.word_addresses.get(s) {
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
                Token::Colon(_) => {
                    panic!("Colon should never reach this function");
                }
                Token::SemiColon => {
                    panic!("SemiColon should never reach this function");
                }
                Token::End => {
                    panic!("Token::End not coded yet");
                }
                Token::Error(_) => {
                    panic!("Token::Error not coded yet");
                }
            }
        }

        //println!("Compiled Codes {:?}", tv);
        //println!("Total size of Codes {:?}", tv.len());
        return Ok(tv);
    }

    fn execute_token_vector(
        &mut self,
        token_vector: &[Token],
        gas_limit: GasLimit,
    ) -> Result<(), ForthError> {
        let mut ol = self.compile_token_vector_compile_and_remove_word_definitions(token_vector)?;
        //println!("Compiled Opcodes: {:?}", ol);
        self.sm.st.opcodes.resize(self.last_function, Opcode::NOP);
        self.sm.st.opcodes.append(&mut ol);
        self.sm.execute(self.last_function, gas_limit)?;
        println!("Total opcodes defined: {}", self.sm.st.opcodes.len());
        println!("Total opcodes executed: {}", self.sm.st.gas_used());

        Ok(())
    }

    pub fn execute_string(&mut self, s: &str, gas_limit: GasLimit) -> Result<(), ForthError> {
        let tv = self.tokenize_string(s)?;
        self.execute_token_vector(&tv, gas_limit)?;
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
            Err(ForthError::UnknownToken(ref x)) if x == "RickTestB" => (),
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
            Err(ForthError::InvalidSyntax(_)) => (),
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
