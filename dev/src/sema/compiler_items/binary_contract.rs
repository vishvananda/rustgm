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
    if hook.assignment() {
        return Err(Declarations::error(
            report,
            site,
            "value binary descriptor does not accept assignment hooks",
        ));
    }
    selected(report, facts, site, hook)
}
fn selected(
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
        Hook::Mul => Operator::Mul,
        Hook::Rem => Operator::Rem,
        Hook::BitXor => Operator::BitXor,
        Hook::BitAnd => Operator::BitAnd,
        Hook::Shr => Operator::Shr,
        Hook::AddAssign => Operator::AddAssign,
        Hook::SubAssign => Operator::SubAssign,
        Hook::MulAssign => Operator::MulAssign,
        Hook::DivAssign => Operator::DivAssign,
        Hook::RemAssign => Operator::RemAssign,
        Hook::BitXorAssign => Operator::BitXorAssign,
        Hook::BitAndAssign => Operator::BitAndAssign,
        Hook::BitOrAssign => Operator::BitOrAssign,
        Hook::ShlAssign => Operator::ShlAssign,
        Hook::ShrAssign => Operator::ShrAssign,
        Hook::Div => Operator::Div,
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

pub(in crate::sema) fn source_assignment(
    report: &Report,
    facts: &mut Declarations,
    site: NodeKey,
    hook: Hook,
) -> Result<SourceBinary, Error> {
    if !hook.assignment() {
        return Err(Declarations::error(
            report,
            site,
            "assignment descriptor requires an actual compound hook",
        ));
    }
    selected(report, facts, site, hook)
}
