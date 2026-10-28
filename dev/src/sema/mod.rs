//! Type checking consumes resolved syntax identities and publishes typed facts once.
pub mod alias_requirements;
mod annotations;
pub mod associated;
pub mod bodies;
pub mod closures;
mod constants;
pub mod crates;
pub mod numbered_regions;
pub use constants::values;
mod api_exposure;
mod arguments;
pub(crate) mod callee_premises;
pub(crate) mod cloning;
pub(crate) mod compiler_items;
mod coverage_annotations;
pub(crate) mod declaration_inputs;
pub mod declarations;
mod deprecation;
pub mod destructors;
mod diagnostic_contracts;
pub(crate) mod dispatch;
pub mod dyn_compat;
pub mod entry;
pub mod foreign;
pub(crate) mod function_context;
pub(crate) mod function_headers;
pub mod generics;
pub mod inline_constants;
pub(crate) mod invocations;
pub(crate) mod iteration;
pub mod layout_view;
pub mod lifetimes;
mod marker_traits;
pub mod members;
pub mod methods;
pub mod model;
pub mod nested_outputs;
mod nominals;
pub mod opaque;
pub mod pattern_view;
pub mod patterns;
pub(crate) mod primitive_constants;
pub(crate) mod primitive_methods;
pub(crate) mod projections;
mod regions;
mod representation;
mod safety;
mod scalar;
pub mod scopes;
pub mod semantic_view;
mod specialization;
pub(crate) mod stored_fields;
mod traits;
#[cfg(test)]
pub(crate) use traits::selection::copy_trace;
pub mod type_view;
pub mod unreduced;
pub(crate) mod vtable_plans;

pub(crate) mod target_features;
pub mod text;

#[path = "annotated_headers.rs"]
pub mod header_attributes;
pub mod must_use;

pub mod inlining;
pub mod non_exhaustive;
pub mod parentheses;

pub(crate) mod try_plans;
