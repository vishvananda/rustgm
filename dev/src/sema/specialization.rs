//! Checked local attributes and specialization structure remain separate proof owners.
mod annotation_reads;
mod annotation_rows;
mod annotation_store;
mod argument_closure;
mod forest_edges;
mod forest_order;
mod graph_closure;
mod graph_pairs;
mod header_conditions;
mod member_paths;
mod minimal_admission;
mod predicate_basis;
mod proof_attempt;
mod proof_inputs;
mod registry_graph;
mod retention_work;
mod selected_member;
pub(in crate::sema) use annotation_reads::marker;
pub(in crate::sema) use annotation_rows::{bind_trait, consume, finish, observe_owner, prepare};
pub(in crate::sema) use annotation_store::LocalMarkers;
pub(in crate::sema) use registry_graph::Graph;

pub(in crate::sema) use graph_pairs::{compare, drain, remembered};

pub(in crate::sema) use graph_closure::{prepare as prepare_graph, seal};

pub(in crate::sema) use member_paths::{conformance_order, inherited_body, prefer};

pub(in crate::sema) use selected_member::{SelectedMethod, resolve as selected_method};

pub(in crate::sema) use header_conditions::Conditions as DefiningConditions;
