//! Maintainer integration suites for inference.
#[path = "../../argument_inference.rs"]
mod argument_inference;
#[path = "../../call_inference.rs"]
mod call_inference;
#[path = "../../constructor_inference.rs"]
mod constructor_inference;
#[path = "../../deferred_inference_equations.rs"]
mod deferred_inference_equations;
#[path = "../../dot_methods.rs"]
mod dot_methods;
#[path = "../../field_inference.rs"]
mod field_inference;
#[path = "../../generic_substitution.rs"]
mod generic_substitution;
#[path = "../../inference_batches.rs"]
mod inference_batches;
#[path = "../../inference_changes.rs"]
mod inference_changes;
#[path = "../../inference_dependencies.rs"]
mod inference_dependencies;
#[path = "../../inference_probe.rs"]
mod inference_probe;
#[path = "../../inferred_parent_proofs.rs"]
mod inferred_parent_proofs;
#[path = "../../inherited_methods.rs"]
mod inherited_methods;
#[path = "../../method_contracts.rs"]
mod method_contracts;
#[path = "../../method_selection.rs"]
mod method_selection;
#[path = "../../method_sites.rs"]
mod method_sites;
#[path = "../../harness/native_references.rs"]
pub(crate) mod native_references;
#[path = "../../opaque_inference.rs"]
mod opaque_inference;
#[path = "../../qualified_methods.rs"]
mod qualified_methods;
#[path = "../../helpers/qualify.rs"]
pub(crate) mod qualify;
#[path = "../../refinement_scaling.rs"]
mod refinement_scaling;
#[path = "../../refinement_scopes.rs"]
mod refinement_scopes;
#[path = "../../refinement_universes.rs"]
mod refinement_universes;
#[path = "../../refining_inference.rs"]
mod refining_inference;
#[path = "../../helpers/resolve.rs"]
pub(crate) mod resolve;
#[path = "../../retained_inference.rs"]
mod retained_inference;
#[path = "../../support/semantic.rs"]
pub(crate) mod semantic;
#[path = "../../shape_arguments.rs"]
mod shape_arguments;
#[path = "../../shape_constructors.rs"]
mod shape_constructors;
#[path = "../../solver_queries.rs"]
mod solver_queries;
#[path = "../../substitution_contexts.rs"]
mod substitution_contexts;
#[path = "../../type_inference.rs"]
mod type_inference;

#[path = "../../conversion_inference.rs"]
mod conversion_inference;
