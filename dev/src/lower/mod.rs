//! Mechanical lowering consumes typed body selections and emits each demanded function once.
mod address;
pub mod alignment;
mod arithmetic;
mod casts;
mod cleanup;
pub mod constant_view;
mod context;
mod data;
mod function;
pub mod instances;
mod intrinsics;
pub mod linkage;
mod remainder;
mod roots;
pub use roots::Roots;
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
    build_roots(facts, bodies, Roots::executable(entry))
}
/// Lower checked executable or entry-free library roots, including registry exports.
/// # Errors
/// Preserves representation, demand and bounded export diagnostics.
pub fn build_roots(
    facts: &mut Declarations,
    bodies: &Checked<'_>,
    roots: Roots<'_>,
) -> Result<program::Program, Error> {
    let exports = roots::exports(facts, roots.exports).map_err(|message| Error {
        node: bodies.program().bodies.first().map_or(
            NodeKey {
                file: crate::resolve::unit::FileId(0),
                node: crate::syntax::ast::NodeId::new(0),
            },
            |body| body.source,
        ),
        message,
    })?;
    build_exact_roots(facts, bodies, exports.selected(roots.entry))
}
pub(crate) fn build_exact_roots(
    facts: &mut Declarations,
    bodies: &Checked<'_>,
    roots: roots::Selected<'_>,
) -> Result<program::Program, Error> {
    let program = bodies.program();
    alignment::check(facts, program)?;
    context::build(facts, bodies, roots)
}
/// Lower an assembled proof in demand order, releasing each body and cleanup
/// plan once it is emitted. Emission order and identities are unchanged.
pub(crate) fn build_assembled(
    facts: &mut Declarations,
    assembled: crate::borrow::incremental::Assembled,
    roots: roots::Selected<'_>,
) -> Result<program::Program, Error> {
    let (program, mut destruction) = assembled.into_lowering();
    alignment::check(facts, &program)?;
    let plans = destruction
        .as_deref_mut()
        .map(crate::borrow::destruction::Proof::take_plans);
    context::build_owned(facts, program.bodies, destruction.as_deref(), plans, roots)
}
