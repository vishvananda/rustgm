//! Type checking consumes resolved syntax identities and publishes typed facts once.
pub mod bodies;
mod constants;
pub use constants::values;
pub mod declarations;
pub mod entry;
pub mod layout_view;
pub mod lifetimes;
pub mod members;
pub mod model;
pub mod pattern_view;
pub mod patterns;
mod regions;
mod representation;
mod scalar;
pub mod type_view;

pub mod text;
