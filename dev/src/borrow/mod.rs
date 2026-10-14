//! Ownership and region proofs retain source semantics before executable erasure.
mod check;
pub use check::{
    Analysis, Checked, Environment, Error, check, inspect, verify, verify_destruction,
    verify_instances,
};
pub mod destruction;
pub mod liveness;
pub mod loans;
pub mod ownership;
mod places;
pub mod regions;
pub mod storage;
pub mod templates;
