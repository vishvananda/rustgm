//! Lifetime contracts retain directed obligations independently of loan liveness.
mod check;
mod interfaces;
mod membership;
pub use crate::types::region_graph as graph;
pub use crate::types::relations as constraints;
mod output;
pub use output::Summary as OutputSummary;
mod paired_group;
mod places;
pub(crate) use paired_group::{Lineage, Retained, check_retained_group};
pub mod points;
pub mod view;
pub use check::{
    Analysis, Body, Error, check, check_bodies, check_ordered, check_templates, prepare_destruction,
};

pub(crate) use check::paid_inline;
pub(crate) use check::{check_nested_bodies, check_one, check_original_child};
