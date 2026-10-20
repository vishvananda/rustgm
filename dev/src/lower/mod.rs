//! Mechanical lowering consumes typed body selections and emits each demanded function once.
mod address;
pub mod alignment;
mod arithmetic;
mod casts;
mod cleanup;
mod context;
mod data;
mod function;
pub mod instances;
mod remainder;
mod virtual_dispatch;

use crate::{
    borrow::Checked,
    lowir::program,
    resolve::{model::DefId, unit::NodeKey},
    sema::model::Declarations,
};
pub struct Error {
    pub node: NodeKey,
    pub message: &'static str,
}

/// Construct typed `LowIR` from combined borrow proofs, including the entry adapter.
///
/// # Errors
/// Diagnoses unsupported demanded representations and inconsistent typed facts.
pub fn build(
    facts: &mut Declarations,
    bodies: &Checked<'_>,
    entry: DefId,
) -> Result<program::Program, Error> {
    let program = bodies.program();
    alignment::check(facts, program)?;
    context::build(facts, bodies, entry)
}
