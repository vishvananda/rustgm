//! Checked local attributes and specialization structure remain separate proof owners.
mod annotation_reads;
mod annotation_rows;
mod annotation_store;
mod argument_closure;
mod forest_edges;
mod forest_order;
mod graph_closure;
#[path = "specialization/overlap_attempts.rs"]
mod graph_pairs;
#[path = "specialization/callee_conditions.rs"]
mod header_conditions;
#[path = "specialization/inherited_paths.rs"]
mod member_paths;
mod minimal_admission;
mod obligation_chain;
mod predicate_basis;
#[path = "specialization/implication_steps.rs"]
mod proof_attempt;
#[path = "specialization/directional_inputs.rs"]
mod proof_inputs;
mod registry_graph;
mod retention_work;
#[path = "specialization/ancestor_target.rs"]
mod selected_member;
pub(in crate::sema) use annotation_reads::marker;
pub(in crate::sema) use annotation_rows::{bind_trait, consume, finish, observe_owner, prepare};
pub(in crate::sema) use annotation_store::LocalMarkers;
pub(in crate::sema) use registry_graph::Graph;

pub(in crate::sema) use graph_pairs::{compare, remembered, requested};

pub(in crate::sema) use graph_closure::{prepare as prepare_graph, reject_execution, seal};

pub(in crate::sema) use member_paths::{conformance_order, inherited_body, prefer};

pub(in crate::sema) use obligation_chain::{ProjectionRequirements, ProofInputs};
pub(in crate::sema) use selected_member::{SelectedMethod, resolve as selected_method};

pub(in crate::sema) use header_conditions::Conditions as DefiningConditions;

#[cfg(test)]
#[path = "../../tests/invariants/marker_contracts.rs"]
mod component_tests;
