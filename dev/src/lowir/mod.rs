//! Typed backend interchange. Text serialization is confined to the writer.
pub mod asm;
pub mod atomic;
pub mod boundary;
pub mod builder;
pub mod data;
pub mod program;
pub mod validation;
mod vararg;
pub mod vocabulary;
pub mod writer;
