//! A binary descriptor borrows real identities; it grants no executable permission.
use super::{
    DefId, Error, Report,
    operator_contract::{self, Operator},
};
use crate::{
    resolve::unit::NodeKey, sema::model::Declarations, support::language_hooks::Hook, types::TypeId,
};
pub(in crate::sema) struct SourceBinary {
    pub definition: DefId,
    pub method: DefId,
    pub signature: TypeId,
    pub rhs: TypeId,
}
pub(in crate::sema) fn source_binary(
    report: &Report,
    facts: &mut Declarations,
    site: NodeKey,
    hook: Hook,
) -> Result<SourceBinary, Error> {
    let operator = requested_operator(report, site, hook)?;
    let actual = operator_contract::query(report, facts, site, operator)?;
    Ok(SourceBinary {
        definition: actual.definition,
        method: actual.method,
        signature: actual.signature,
        rhs: actual.rhs.ok_or_else(|| {
            Declarations::error(report, site, "source binary descriptor lost its Rhs formal")
        })?,
    })
}
pub(in crate::sema) fn pending_default(
    report: &Report,
    facts: &Declarations,
    site: NodeKey,
    hook: Hook,
) -> Result<Option<DefId>, Error> {
    let operator = requested_operator(report, site, hook)?;
    operator_contract::pending_default(report, facts, site, operator)
}
fn requested_operator(report: &Report, site: NodeKey, hook: Hook) -> Result<Operator, Error> {
    let operator = match hook {
        Hook::Shl => Operator::Shl,
        Hook::BitOr => Operator::BitOr,
        Hook::Add => Operator::Add,
        Hook::Sub => Operator::Sub,
        _ => {
            return Err(Declarations::error(
                report,
                site,
                "unsupported source binary operator",
            ));
        }
    };
    Ok(operator)
}

#[cfg(test)]
#[path = "../../../tests/invariants/binary_authority.rs"]
mod tests;
