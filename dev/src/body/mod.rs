//! Typed control flow consumes solved semantic facts without repeating inference.
pub mod body_view;
mod build;
pub mod ir;
pub mod promote;
pub mod regions;
pub mod scopes;
pub mod storage;

pub use build::build;
