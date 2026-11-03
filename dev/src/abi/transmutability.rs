//! Finite layout languages describe all values; conditional answers retain typed obligations.
mod automata;
mod budget;
mod interpretation;
mod planning;
mod propositions;
mod relation;
mod transitions;
mod typed_layouts;
pub use automata::{ByteRange, Language, LayoutId, Reference};
pub use budget::{Budget, Limits};
pub use propositions::{Condition, ConditionId, Obligation, Proof};
pub use relation::decide;
pub use typed_layouts::describe;

/// These are decoded source obligations, separate from any permission to execute a call.
#[derive(Clone, Copy, Debug, Default)]
#[expect(
    clippy::struct_excessive_bools,
    reason = "Four independent assumptions mirror the authenticated source options protocol"
)]
pub struct Assumptions {
    pub alignment: bool,
    pub lifetimes: bool,
    pub safety: bool,
    pub validity: bool,
}

#[cfg(test)]
#[path = "../../tests/invariants/transmutability_algebra.rs"]
mod tests;
