extern crate rust_simple_stack_processor;

use rust_forth_compiler::ForthCompiler;
use rust_forth_compiler::ForthError;
use rust_forth_compiler::GasLimit;
use std::fs;

fn main() -> Result<(), ForthError> {
    println!("Hello, world! This is rust-forth-compiler");

    run()?;

    Ok(())
}

fn run() -> Result<(), ForthError> {
    let mut fc = ForthCompiler::new();

    //fc.execute_string("1 IF 1 2 ADD ELSE 3 4 ADD THEN", GasLimit::Limited(100))?;
    fc.execute_string("0 IF 1 2 ADD THEN", GasLimit::Limited(100))?;

    println!("Contents of Number Stack {:?}", fc.sm.st.number_stack);
    assert_eq!(&fc.sm.st.number_stack, &vec![3_i64]);

    fc.execute_string(
        ": RickTest 1 2 ADD 3 MUL ; RickTest",
        GasLimit::Limited(100),
    )?;

    //    assert_eq!(&fc.sm.st.number_stack, &vec![888_i64]);

    fc.execute_string(": RickTest2 4 5 ADD 6 MUL ;", GasLimit::Limited(100))?;

    fc.execute_string(
        ": RickTest3 RickTest RickTest2 7 ADD 8 MUL ;",
        GasLimit::Limited(100),
    )?;

    //    assert_eq!(&fc.sm.st.number_stack, &vec![888_i64]);

    fc.execute_string("RickTest3", GasLimit::Limited(100))?;

    assert_eq!(&fc.sm.st.number_stack, &vec![3_i64, 9, 9, 488]);

    fc.execute_string("123 321 ADD 2 MUL", GasLimit::Limited(100))?;

    assert_eq!(&fc.sm.st.number_stack, &vec![3_i64, 9, 9, 488, 888]);

    fc.execute_string("123 321 ADD 2 MUL", GasLimit::Limited(100))?;

    assert_eq!(&fc.sm.st.number_stack, &vec![3_i64, 9, 9, 488, 888, 888]);

    let startup = fs::read_to_string("init.forth")?;
    fc.execute_string(&startup, GasLimit::Limited(100))?;

    fc.execute_string(
        "predefined1 123 predefined2 456 POP Numbers MUL ADD DUP",
        GasLimit::Limited(100),
    )?;

    fc.execute_string(
        ": RickCommand 123456 DUP ADD 777 ; RickCommand RickCommand",
        GasLimit::Limited(100),
    )?;

    assert_eq!(
        &fc.sm.st.number_stack,
        &vec![3_i64, 9, 9, 488, 888, 888, 123, 1, 2, 3, 34, 34, 246912, 777, 246912, 777]
    );

    fc.sm.st.number_stack.push(123);
    fc.sm.st.number_stack.push(321);
    fc.sm.st.number_stack.push(0);
    fc.execute_string("IF ADD 2 MUL ELSE ADD 3 MUL THEN", GasLimit::Limited(100))
        .unwrap();
    let n = fc.sm.st.number_stack.pop().unwrap();

    assert_eq!(n, 888);

    /*
        let mut rf = ForthInterpreter::new();

        let startup = fs::read_to_string("init.forth")?;
        rf.execute_string(&startup)?;

        rf.execute_string("predefined1 123 predefined2 456 POP Numbers MUL ADD DUP")?;

        rf.execute_string(": RickCommand 123456 DUP ADD 777 ; RickCommand RickCommand")?;

        assert_eq!(
            rf.access_stack(),
            &vec![123_i64, 1, 2, 3, 34, 34, 246912, 777, 246912, 777]
        );

        rf.token_handlers
            .push(Box::new(ExternalCommandHandler::new()));

        rf.execute_string("1111 123456 OUT 123456 IN")?;

        assert_eq!(
            rf.access_stack(),
            &vec![123_i64, 1, 2, 3, 34, 34, 246912, 777, 246912, 777, 777]
        );

        rf.push_stack(123);
        rf.push_stack(321);
        rf.push_stack(0);
        rf.execute_string("IF ADD 2 MUL ELSE ADD 3 MUL THEN")
            .unwrap();
        let n = rf.pop_stack().unwrap();

        assert_eq!(n, 1332);
    */
    Ok(())
}
/*
pub struct ExternalCommandHandler {}

impl HandleToken for ExternalCommandHandler {
    fn handle_token(&mut self, t: &Token, st: &mut State) -> Result<Handled, ForthError> {
        if let Token::Command(s) = t {
            println!("ExternalCommandHandler: Interpreting token {}", s);
            match s.as_ref() {
                "OUT" => self.out_port(st).map(|_| Ok(Handled::Handled))?,
                "IN" => self.in_port(st).map(|_| Ok(Handled::Handled))?,
                _ => Ok(Handled::NotHandled),
            }
        } else {
            Ok(Handled::NotHandled)
        }
    }
}

impl ExternalCommandHandler {
    fn out_port(&self, st: &mut State) -> Result<(), ForthError> {
        let port = st.number_stack.pop_stack()?;
        let value = st.number_stack.pop_stack()?;

        println!("Sending {} to port {}", value, port);

        Ok(())
    }

    fn in_port(&self, st: &mut State) -> Result<(), ForthError> {
        let port = st.number_stack.pop_stack()?;
        let value = 777;

        st.number_stack.push_stack(value);

        println!("Receiving {} from port {}", value, port);

        Ok(())
    }

    pub fn new() -> ExternalCommandHandler {
        ExternalCommandHandler {}
    }
}
*/
