//! Typed control flow consumes solved semantic facts without repeating inference.
pub(crate) mod adapters;
pub mod body_view;
mod build;
mod caller_identity;
mod clone_factories;
mod collection_factories;
pub(crate) mod conditional_inputs;
pub mod constants;
mod diagnostics;
pub use diagnostics::BodyOrigins;
pub(crate) mod copy_requirements;
pub(crate) mod dispatch;
pub mod drops;
pub mod instantiate;
pub mod invocation_entries;
pub mod ir;
pub(crate) mod iterator_factories;
pub(crate) mod length_methods;
pub(crate) mod primitive_methods;
pub mod promote;
pub(crate) mod provided;
mod range_factories;
pub mod regions;
pub use crate::sema::scopes;
pub mod storage;

pub use build::build;
pub use build::concrete as build_concrete;
pub(crate) use build::concrete_prepared;
pub use build::templates;

/// Prepare source-provider Copy requirements for this exact body and contract.
/// Ordinary compilation returns without source-only allocation. This selects
/// requirements; callers must still prove the contract and ordinary ownership.
/// # Errors
/// Rejects foreign body/contract identities, unvalidated roles, pending selection
/// and exhausted work. A missing Copy role remains an explicit read-time policy.
pub fn prepare_source_copy(
    facts: &mut crate::sema::model::Declarations,
    body: &ir::Body,
    contract: &mut regions::Contract,
) -> Result<(), String> {
    copy_requirements::prepare(facts, body, contract)
}
