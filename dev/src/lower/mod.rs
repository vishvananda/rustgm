//! Mechanical lowering consumes typed body selections and emits each demanded function once.
mod arithmetic;
mod casts;
mod context;
mod function;
mod remainder;

use crate::{
    body::scalar_check::Checked,
    lowir::program,
    resolve::{model::DefId, unit::NodeKey},
    types::Store,
};
pub struct Error {
    pub node: NodeKey,
    pub message: &'static str,
}

/// Construct typed scalar `LowIR`, including the executable entry adapter.
///
/// # Errors
/// Diagnoses unsupported demanded representations and inconsistent typed facts.
pub fn build(types: &Store, bodies: &Checked<'_>, entry: DefId) -> Result<program::Program, Error> {
    context::Context::build(types, bodies.program(), entry)
}
