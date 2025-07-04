use rust_simple_stack_processor::{GasLimit, StackMachineError};
use thiserror::Error;

/// This Enum lists the errors that the Forth Interpreter might return
#[derive(Debug, Error)]
pub enum ForthError {
    #[error("Unknown error occurred")]
    UnknownError,
    #[error("Unknown token: {0}")]
    UnknownToken(String),
    #[error("Number stack underflow")]
    NumberStackUnderflow,
    #[error("Loop stack underflow")]
    LoopStackUnderflow,
    #[error("Scratch stack underflow")]
    ScratchStackUnderflow,
    #[error("Invalid cell operation")]
    InvalidCellOperation,
    #[error("Invalid syntax: {0}")]
    InvalidSyntax(String),
    #[error("Missing semicolon after colon")]
    MissingSemicolonAfterColon,
    #[error("Missing command after colon")]
    MissingCommandAfterColon,
    #[error("Semicolon before colon")]
    SemicolonBeforeColon,
    #[error("Unhandled trap ID: {unhandled_trap_id}")]
    UnhandledTrap { unhandled_trap_id: i64 },
    #[error("Ran out of gas: used {gas_used}, limit {gas_limit:?}")]
    RanOutOfGas { gas_used: u64, gas_limit: GasLimit },
    #[error("Internal numeric overflow")]
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
            StackMachineError::InvalidCellOperation => ForthError::InvalidCellOperation,
            StackMachineError::UnknownError => ForthError::UnknownError,
            StackMachineError::UnhandledTrap { unhandled_trap_id } => {
                ForthError::UnhandledTrap { unhandled_trap_id }
            }
            StackMachineError::RanOutOfGas {
                gas_used,
                gas_limit,
            } => ForthError::RanOutOfGas {
                gas_used,
                gas_limit,
            },
            StackMachineError::NumericOverflow { failing_opcode: _ } => {
                ForthError::InternalNumericOverflow
            }
            StackMachineError::DivisionByZero { failing_opcode: _ } => {
                ForthError::InternalNumericOverflow
            }
            StackMachineError::TryFromIntError(_) => ForthError::InternalNumericOverflow,
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
            ForthError::InvalidCellOperation => 14,
            ForthError::InvalidSyntax(_) => 6,
            ForthError::MissingSemicolonAfterColon => 7,
            ForthError::MissingCommandAfterColon => 8,
            ForthError::SemicolonBeforeColon => 9,
            ForthError::UnhandledTrap {
                unhandled_trap_id: _,
            } => 10,
            ForthError::RanOutOfGas {
                gas_used: _,
                gas_limit: _,
            } => 11,
            ForthError::InternalNumericOverflow => 12,
        }
    }
}
