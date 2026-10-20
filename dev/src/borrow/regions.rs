//! Lifetime contracts retain directed obligations independently of loan liveness.
mod check;
mod membership;
pub use crate::types::region_graph as graph;
pub use crate::types::relations as constraints;
mod places;
pub mod points;
pub mod view;
pub use check::{
    Analysis, Body, Error, check, check_bodies, check_ordered, check_templates, prepare_destruction,
};

pub(crate) use check::check_one;
