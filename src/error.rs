extern crate rust_simple_stack_processor;

use rust_simple_stack_processor::StackMachineError;

/// This Enum lists the errors that the Forth Interpreter might return
#[derive(Debug)]
pub enum ForthError {
    UnknownError,
    UnknownToken(String),
    NumberStackUnderflow,
    LoopStackUnderflow,
    ScratchStackUnderflow,
    InvalidSyntax(String),
    MissingSemicolonAfterColon,
    MissingCommandAfterColon,
    SemicolonBeforeColon,
    UnhandledTrap,
    RanOutOfGas,
    InternalNumericOverflow,
}

/// Convert StackMachineError to a ForthError so our Interpreter functions can
/// return a single Error type.
impl From<StackMachineError> for ForthError {
    fn from(err: StackMachineError) -> ForthError {
        match err {
            StackMachineError::NumberStackUnderflow => ForthError::NumberStackUnderflow,
            StackMachineError::LoopStackUnderflow => ForthError::LoopStackUnderflow,
            StackMachineError::ScratchStackUnderflow => ForthError::ScratchStackUnderflow,
            StackMachineError::UnkownError => ForthError::UnknownError,
            StackMachineError::UnhandledTrap => ForthError::UnhandledTrap,
            StackMachineError::RanOutOfGas => ForthError::RanOutOfGas,
            StackMachineError::NumericOverflow => ForthError::InternalNumericOverflow,
        }
    }
}

/// Helper to convert ForthError codes to numeric codes for exit()
impl From<ForthError> for i32 {
    fn from(err: ForthError) -> Self {
        match err {
            ForthError::UnknownError => 2,
            ForthError::UnknownToken(_) => 3,
            ForthError::NumberStackUnderflow => 4,
            ForthError::LoopStackUnderflow => 5,
            ForthError::ScratchStackUnderflow => 13,
            ForthError::InvalidSyntax(_) => 6,
            ForthError::MissingSemicolonAfterColon => 7,
            ForthError::MissingCommandAfterColon => 8,
            ForthError::SemicolonBeforeColon => 9,
            ForthError::UnhandledTrap => 10,
            ForthError::RanOutOfGas => 11,
            ForthError::InternalNumericOverflow => 12,
        }
    }
}
