//! Ownership and region proofs retain source semantics before executable erasure.
mod active;
mod check;
pub(crate) mod copy_view;
pub use check::{
    Analysis, Checked, Environment, Error, check, inspect, verify, verify_destruction,
    verify_instances,
};
pub mod destruction;
pub mod liveness;
pub mod loans;
mod order;
pub mod ownership;
mod permissions;
mod places;
pub mod regions;
pub mod storage;
pub mod templates;

pub(crate) use check::incremental;
#[cfg(test)]
pub(crate) use check::structure;
