//! Typed control flow consumes solved semantic facts without repeating inference.
pub(crate) mod adapters;
pub mod body_view;
mod build;
pub mod constants;
pub(crate) mod dispatch;
pub mod drops;
pub mod instantiate;
pub mod invocation_entries;
pub mod ir;
pub(crate) mod length_methods;
pub(crate) mod primitive_methods;
pub mod promote;
pub(crate) mod provided;
pub mod regions;
pub use crate::sema::scopes;
pub mod storage;

pub use build::build;
pub use build::concrete as build_concrete;
pub(crate) use build::concrete_prepared;
pub use build::templates;
