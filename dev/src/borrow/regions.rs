//! Lifetime contracts retain directed obligations independently of loan liveness.
mod check;
pub mod constraints;
pub mod graph;
mod places;
pub use check::{Analysis, Body, Error, check, check_bodies};
