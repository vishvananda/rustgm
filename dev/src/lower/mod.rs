//! Mechanical lowering consumes typed body selections and emits each demanded function once.
mod arithmetic;
mod casts;
mod context;
mod function;
mod remainder;

use crate::{
    borrow::ownership::Checked,
    lowir::program,
    resolve::{model::DefId, unit::NodeKey},
    sema::model::Declarations,
};
pub struct Error {
    pub node: NodeKey,
    pub message: &'static str,
}

/// Construct typed `LowIR` from proved owned bodies, including the entry adapter.
///
/// # Errors
/// Diagnoses unsupported demanded representations and inconsistent typed facts.
pub fn build(
    facts: &mut Declarations,
    bodies: &Checked<'_>,
    entry: DefId,
) -> Result<program::Program, Error> {
    context::Context::build(facts, bodies.program(), entry)
}
