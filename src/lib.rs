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

#[cfg(test)]
mod tests;

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

impl Default for ForthCompiler {
    fn default() -> ForthCompiler {
        ForthCompiler {
            sm: StackMachine::default(),
            intrinsic_words: hashmap![
            "POP" => vec![Opcode::POP],
            "SWAP" => vec![Opcode::SWAP],
            "NOT" => vec![Opcode::NOT],
            "ADD" => vec![Opcode::ADD],
            "SUB" => vec![Opcode::SUB],
            "MUL" => vec![Opcode::MUL],
            "DIV" => vec![Opcode::DIV],
            "DUP" => vec![Opcode::DUP],
            "TRAP" => vec![Opcode::TRAP],
            "1+" => vec![Opcode::LDI(1),Opcode::ADD],
            "1-" => vec![Opcode::LDI(-1),Opcode::ADD],
            "2+" => vec![Opcode::LDI(2),Opcode::ADD],
            "2-" => vec![Opcode::LDI(-2),Opcode::ADD],
            "2*" => vec![Opcode::LDI(2),Opcode::MUL],
            "2/" => vec![Opcode::LDI(2),Opcode::DIV],
            "I" => vec![Opcode::GETLP],
            "J" => vec![Opcode::GETLP2]
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
            if_location,
            else_location: None,
        }
    }
}

// This struct tracks information for Forth Loop statements
#[derive(Debug)]
struct DeferredDoLoopStatement {
    prelude_start: usize,
    logical_start: usize,
}

impl DeferredDoLoopStatement {
    pub fn new(prelude_start: usize, logical_start: usize) -> DeferredDoLoopStatement {
        DeferredDoLoopStatement {
            prelude_start,
            logical_start,
        }
    }
}

#[derive(Debug)]
struct LoopExits {
    loop_exit_locations: Vec<usize>,
}

impl LoopExits {
    pub fn new() -> LoopExits {
        LoopExits {
            loop_exit_locations: Vec::new(),
        }
    }

    pub fn add_exit_point(&mut self, loop_exit_location: usize) {
        self.loop_exit_locations.push(loop_exit_location);
    }

    fn fixup_loop_exits(&self, opcode_vector: &mut Vec<Opcode>) {
        let loop_exit_point = opcode_vector.len();
        for leave_point in self.loop_exit_locations.iter() {
            let jump_forward =
                i64::try_from(loop_exit_point).unwrap() - i64::try_from(*leave_point).unwrap() - 1;
            opcode_vector[*leave_point] = Opcode::LDI(jump_forward);
        }
    }
}

#[derive(Debug)]
struct DeferredBeginLoopStatement {
    logical_start: usize,
}

impl DeferredBeginLoopStatement {
    pub fn new(logical_start: usize) -> DeferredBeginLoopStatement {
        DeferredBeginLoopStatement { logical_start }
    }
}

enum DeferredStatement {
    If(DeferredIfStatement),
    DoLoop(DeferredDoLoopStatement, LoopExits),
    BeginLoop(DeferredBeginLoopStatement, LoopExits),
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
        let mut deferred_statements = Vec::new();
        // List of compiled processor opcodes that we are building up
        let mut tv: Vec<Opcode> = Vec::new();

        // Go through all the Forth tokens and turn them into processor Opcodes (for our StackMachine emulated processor)
        for t in token_vector.iter() {
            match t {
                ForthToken::DropLineComment(_) => (),
                ForthToken::ParenthesizedRemark(_) => (),
                ForthToken::StringCommand(_, _) => (),
                ForthToken::Number(n) => {
                    // Numbers get pushed as a LDI opcode
                    tv.push(Opcode::LDI(*n));
                }
                ForthToken::Command(s) => {
                    // Remember where we are in the list of opcodes in case we hit a IF statement, LOOP etc...
                    let current_instruction = tv.len();

                    match s.as_ref() {
                        "DO" => {
                            let start_of_loop_code = current_instruction;
                            // This eats the loop parameters from the number stack...
                            tv.push(Opcode::PUSHLP);
                            let logical_start_of_loop = tv.len();
                            deferred_statements.push(DeferredStatement::DoLoop(
                                DeferredDoLoopStatement::new(
                                    start_of_loop_code,
                                    logical_start_of_loop,
                                ),
                                LoopExits::new(),
                            ));
                        }
                        "LOOP" => {
                            if let Some(DeferredStatement::DoLoop(loop_def, loop_exits)) =
                                deferred_statements.pop()
                            {
                                let jump_back = i64::try_from(loop_def.logical_start).unwrap()
                                    - i64::try_from(current_instruction).unwrap()
                                    // Have to jump back over the JR and the LDI
                                    - 3;
                                tv.push(Opcode::INCLP);
                                tv.push(Opcode::CMPLOOP);
                                tv.push(Opcode::LDI(jump_back));
                                tv.push(Opcode::JRZ);

                                loop_exits.fixup_loop_exits(&mut tv);
                            } else {
                                return Err(ForthError::InvalidSyntax(
                                    "LOOP without proper loop start like DO".to_owned(),
                                ));
                            }
                            tv.push(Opcode::DROPLP);
                        }
                        "+LOOP" => {
                            if let Some(DeferredStatement::DoLoop(loop_def, loop_exits)) =
                                deferred_statements.pop()
                            {
                                let jump_back = i64::try_from(loop_def.logical_start).unwrap()
                                    - i64::try_from(current_instruction).unwrap()
                                    // Have to jump back over the JR and the LDI
                                    - 3;
                                tv.push(Opcode::ADDLP);
                                tv.push(Opcode::CMPLOOP);
                                tv.push(Opcode::LDI(jump_back));
                                tv.push(Opcode::JRZ);

                                loop_exits.fixup_loop_exits(&mut tv);
                            } else {
                                return Err(ForthError::InvalidSyntax(
                                    "+LOOP without proper loop start like DO".to_owned(),
                                ));
                            }
                            tv.push(Opcode::DROPLP);
                        }
                        "LEAVE" => {
                            let most_recent_loop_statement =
                                deferred_statements.iter_mut().rev().find(|x| match **x {
                                    DeferredStatement::If(_) => false,
                                    DeferredStatement::DoLoop(_, _) => true,
                                    DeferredStatement::BeginLoop(_, _) => true,
                                });
                            if let Some(deferred_statement) = most_recent_loop_statement {
                                let loop_exits =
                                    match deferred_statement {
                                        DeferredStatement::DoLoop(_, loop_exits) => loop_exits,
                                        DeferredStatement::BeginLoop(_, loop_exits) => loop_exits,
                                        _ => return Err(ForthError::InvalidSyntax(
                                            "LEAVE without proper loop start like DO or BEGIN(1)"
                                                .to_owned(),
                                        )),
                                    };
                                // Record the exit point
                                loop_exits.add_exit_point(current_instruction);

                                // We fix up the jumps once we get the end of loop
                                tv.push(Opcode::LDI(0));
                                tv.push(Opcode::JR);
                            } else {
                                return Err(ForthError::InvalidSyntax(
                                    "LEAVE without proper loop start like DO or BEGIN(2)"
                                        .to_owned(),
                                ));
                            }
                        }
                        "BEGIN" => {
                            deferred_statements.push(DeferredStatement::BeginLoop(
                                DeferredBeginLoopStatement::new(current_instruction),
                                LoopExits::new(),
                            ));
                        }
                        "UNTIL" => {
                            if let Some(DeferredStatement::BeginLoop(loop_def, loop_exits)) =
                                deferred_statements.pop()
                            {
                                let jump_back = i64::try_from(loop_def.logical_start).unwrap()
                                    - i64::try_from(current_instruction).unwrap()
                                    // Have to jump back over the JR and the LDI
                                    - 1;
                                tv.push(Opcode::LDI(jump_back));
                                tv.push(Opcode::JRZ);

                                loop_exits.fixup_loop_exits(&mut tv);
                            } else {
                                return Err(ForthError::InvalidSyntax(
                                    "UNTIL without proper loop start like BEGIN".to_owned(),
                                ));
                            }
                        }
                        "WHILE" => {
                            if let Some(DeferredStatement::BeginLoop(_loop_def, loop_exits)) =
                                deferred_statements.last_mut()
                            {
                                loop_exits.add_exit_point(current_instruction);
                                // We fix up the jumps once we get the end of loop
                                tv.push(Opcode::LDI(0));
                                tv.push(Opcode::JRZ);
                            } else {
                                return Err(ForthError::InvalidSyntax(
                                    "WHILE without proper loop start like BEGIN".to_owned(),
                                ));
                            }
                        }
                        "REPEAT" => {
                            if let Some(DeferredStatement::BeginLoop(loop_def, loop_exits)) =
                                deferred_statements.pop()
                            {
                                let jump_back = i64::try_from(loop_def.logical_start).unwrap()
                                    - i64::try_from(current_instruction).unwrap()
                                    // Have to jump back over the JR and the LDI
                                    - 1;
                                tv.push(Opcode::LDI(jump_back));
                                tv.push(Opcode::JR);

                                loop_exits.fixup_loop_exits(&mut tv);
                            } else {
                                return Err(ForthError::InvalidSyntax(
                                    "AGAIN without proper loop start like BEGIN".to_owned(),
                                ));
                            }
                        }
                        "AGAIN" => {
                            if let Some(DeferredStatement::BeginLoop(loop_def, loop_exits)) =
                                deferred_statements.pop()
                            {
                                let jump_back = i64::try_from(loop_def.logical_start).unwrap()
                                    - i64::try_from(current_instruction).unwrap()
                                    // Have to jump back over the JR and the LDI
                                    - 1;
                                tv.push(Opcode::LDI(jump_back));
                                tv.push(Opcode::JR);

                                loop_exits.fixup_loop_exits(&mut tv);
                            } else {
                                return Err(ForthError::InvalidSyntax(
                                    "AGAIN without proper loop start like BEGIN".to_owned(),
                                ));
                            }
                        }
                        // FLAG 0 = Skip stuff inside IF, !0 = Run stuff inside IF
                        "IF" => {
                            deferred_statements.push(DeferredStatement::If(
                                DeferredIfStatement::new(current_instruction),
                            ));
                            //println!("(IF)Deferred If Stack {:?}", deferred_if_statements);
                            tv.push(Opcode::LDI(0));
                            tv.push(Opcode::JRZ);
                        }
                        "ELSE" => {
                            if let Some(DeferredStatement::If(x)) = deferred_statements.last_mut() {
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
                            if let Some(DeferredStatement::If(x)) = deferred_statements.pop() {
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
                            } else if let Some(ol) = self.intrinsic_words.get::<str>(s) {
                                tv.append(&mut ol.clone());
                            } else {
                                return Err(ForthError::UnknownToken(s.to_string()));
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

        Ok(tv)
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
