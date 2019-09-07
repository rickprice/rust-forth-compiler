extern crate rust_simple_stack_processor;

use rust_simple_stack_processor::StackMachineError;

/// This Enum lists the errors that the Forth Interpreter might return
#[derive(Debug)]
pub enum ForthError {
    UnknownError,
    UnknownToken(String),
    PopOfEmptyStack,
    InvalidSyntax(String),
    MissingSemicolonAfterColon,
    MissingCommandAfterColon,
    SemicolonBeforeColon,
    UnhandledTrap,
    RanOutOfGas,
}

/// Convert StackMachineError to a ForthError so our Interpreter functions can
/// return a single Error type.
impl From<StackMachineError> for ForthError {
    fn from(err: StackMachineError) -> ForthError {
        match err {
            StackMachineError::NumberStackUnderflow => ForthError::PopOfEmptyStack,
            StackMachineError::UnkownError => ForthError::UnknownError,
            StackMachineError::UnhandledTrap => ForthError::UnhandledTrap,
            StackMachineError::RanOutOfGas => ForthError::RanOutOfGas,
        }
    }
}

/// Helper to convert ForthError codes to numeric codes for exit()
impl From<ForthError> for i32 {
    fn from(err: ForthError) -> Self {
        match err {
            ForthError::UnknownError => 2,
            ForthError::UnknownToken(_) => 3,
            ForthError::PopOfEmptyStack => 4,
            ForthError::InvalidSyntax(_) => 5,
            ForthError::MissingSemicolonAfterColon => 6,
            ForthError::MissingCommandAfterColon => 7,
            ForthError::SemicolonBeforeColon => 8,
            ForthError::UnhandledTrap => 9,
            ForthError::RanOutOfGas => 10,
        }
    }
}
