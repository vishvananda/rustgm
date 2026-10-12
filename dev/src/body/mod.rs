//! Typed control flow consumes solved semantic facts without repeating inference.
pub mod body_view;
mod build;
pub mod ir;
pub mod scalar_check;

pub use build::build;
