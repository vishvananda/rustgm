//! Unary adapters retain the existing API over the shared actual operator walk.
use super::{
    DefId, Error, Report,
    operator_contract::{self, Operator},
};
use crate::{resolve::unit::NodeKey, sema::model::Declarations, types::TypeId};

pub(in crate::sema) struct SourceUnary {
    pub definition: DefId,
    pub method: DefId,
    pub signature: TypeId,
}
pub(in crate::sema) type SourceNot = SourceUnary;
pub(in crate::sema) fn source_not(
    report: &Report,
    facts: &mut Declarations,
    site: NodeKey,
) -> Result<SourceNot, Error> {
    unary(report, facts, site, Operator::Not)
}
pub(in crate::sema) fn source_neg(
    report: &Report,
    facts: &mut Declarations,
    site: NodeKey,
) -> Result<SourceUnary, Error> {
    unary(report, facts, site, Operator::Neg)
}
fn unary(
    report: &Report,
    facts: &mut Declarations,
    site: NodeKey,
    operator: Operator,
) -> Result<SourceUnary, Error> {
    let actual = operator_contract::query(report, facts, site, operator)?;
    Ok(SourceUnary {
        definition: actual.definition,
        method: actual.method,
        signature: actual.signature,
    })
}
#[cfg(test)]
use crate::{
    resolve::model::DefKind, support::bindings::LangItem, syntax::item::ItemKind, types::Kind,
};

#[cfg(test)]
#[path = "../../../tests/invariants/not_contracts.rs"]
mod tests;

#[cfg(test)]
#[path = "../../../tests/invariants/neg_contracts.rs"]
mod neg_tests;
