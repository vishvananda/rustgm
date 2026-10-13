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
    let program = bodies.program();
    let pending = program
        .outlives
        .first()
        .map(|r| r.source)
        .or_else(|| program.coercions.first().map(|c| c.source))
        .or_else(|| program.region_equalities.first().map(|r| r.body))
        .or_else(|| program.body_regions.first().map(|r| r.source));
    if let Some(node) = pending {
        return Err(Error {
            node,
            message: "reference lowering requires completed region and loan proofs",
        });
    }
    context::Context::build(facts, program, entry)
}
