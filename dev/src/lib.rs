//! `RustGM`'s maintainer frontend. Text formats are views of typed phases.
pub mod abi;
pub mod body;
pub mod borrow;
pub mod expand;
pub mod lex;
pub mod literal;
pub mod lower;
pub mod lowir;
pub mod pipeline;
pub mod resolve;
#[cfg(test)]
#[path = "../tests/helpers/qualify.rs"]
mod rustc;
pub mod sema;
pub mod support;
pub mod syntax;
pub mod types;
