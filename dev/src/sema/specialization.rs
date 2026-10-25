//! Checked local attributes and specialization structure remain separate proof owners.
mod annotation_reads;
mod annotation_rows;
mod annotation_store;
mod argument_closure;
mod forest_edges;
mod forest_order;
mod graph_pairs;
mod header_conditions;
mod proof_attempt;
mod proof_inputs;
mod registry_graph;
mod retention_work;
pub(in crate::sema) use annotation_reads::marker;
pub(in crate::sema) use annotation_rows::{bind_trait, consume, finish, observe_owner, prepare};
pub(in crate::sema) use annotation_store::LocalMarkers;
pub(in crate::sema) use registry_graph::Graph;

pub(in crate::sema) use graph_pairs::{compare, drain, remembered};
