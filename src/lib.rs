//! This is documentation for the rust-forth-compiler module
//!
//!

extern crate rust_simple_stack_processor;

pub use error::ForthError;
pub use forth_compiler::Token;

pub mod error;
pub mod forth_compiler;

pub enum Handled {
    Handled,
    NotHandled,
}
