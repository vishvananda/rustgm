//! Type checking consumes resolved syntax identities and publishes typed facts once.
mod annotations;
pub mod associated;
pub mod bodies;
mod constants;
pub use constants::values;
mod arguments;
pub mod declarations;
pub mod destructors;
pub mod entry;
pub mod generics;
pub mod layout_view;
pub mod lifetimes;
pub mod members;
pub mod methods;
pub mod model;
mod nominals;
pub mod pattern_view;
pub mod patterns;
mod regions;
mod representation;
mod scalar;
pub mod scopes;
pub mod type_view;

pub mod text;
