//! A range member descriptor confers no implementation, API, const or body permission.
use super::{Error, Report, roles::range_members::ROLES};
use crate::{
    resolve::{model::DefId, unit::NodeKey},
    sema::model::Declarations,
    types::TypeId,
};
#[derive(Clone, Copy)]
pub(in crate::sema) enum RangeHelper {
    Minimum,
    Maximum,
    Predecessor,
}
pub(in crate::sema) enum SourceRangeMember {
    Bound {
        parent: DefId,
        member: DefId,
        self_type: TypeId,
    },
    Predecessor {
        parent: DefId,
        member: DefId,
        signature: TypeId,
    },
}
fn range_contract(facts: &Declarations) -> Option<&super::roles::range_members::Contracts> {
    facts
        .source_bindings
        .as_deref()?
        .traits
        .as_deref()?
        .range
        .as_deref()
}

pub(in crate::sema) fn source_range_headers(
    facts: &Declarations,
) -> Option<(
    crate::resolve::bindings::Declaration,
    [crate::resolve::bindings::Declaration; 3],
)> {
    let contract = range_contract(facts)?;
    Some((contract.parent, contract.members))
}

pub(in crate::sema) fn require_range_transport(
    report: &Report,
    facts: &Declarations,
) -> Result<(), &'static str> {
    range_contract(facts)
        .ok_or("source range transport requires all three completed roles")?
        .current(report, facts)
        .map_err(|_| "source range transport changed its completed declaration contract")
}

pub(in crate::sema) fn source_range_member(
    report: &Report,
    facts: &mut Declarations,
    site: NodeKey,
    helper: RangeHelper,
) -> Result<SourceRangeMember, Error> {
    facts.body_work.canonical_steps = facts.body_work.canonical_steps.saturating_add(1);
    crate::sema::header_attributes::charge_source_contract_query(facts, 4)?;
    let catalog = facts.source_bindings.as_deref().ok_or_else(|| {
        Declarations::error(
            report,
            site,
            "source range identity requires its actual source catalog",
        )
    })?;
    catalog
        .require_provider_report(report)
        .map_err(|e| Declarations::error(report, site, e))?;
    let complete = catalog.traits.as_deref().ok_or_else(|| {
        Declarations::error(
            report,
            site,
            "source range family declaration contract is pending",
        )
    })?;
    let contracts = complete.range.as_deref().ok_or_else(|| {
        Declarations::error(
            report,
            site,
            "source range family declaration contract is pending or requires transport",
        )
    })?;
    for role in ROLES {
        if catalog
            .validated_language(role)
            .map_err(|e| Declarations::error(report, site, e))?
            .is_none()
        {
            return Err(Declarations::error(
                report,
                site,
                "source range helper requires its actual completed role",
            ));
        }
    }
    contracts.current(report, facts)?;
    let index = match helper {
        RangeHelper::Minimum => 0,
        RangeHelper::Maximum => 1,
        RangeHelper::Predecessor => 2,
    };
    let parent = contracts.parent.definition;
    let member = contracts.members[index].definition;
    Ok(if index < 2 {
        SourceRangeMember::Bound {
            parent,
            member,
            self_type: contracts.self_type,
        }
    } else {
        SourceRangeMember::Predecessor {
            parent,
            member,
            signature: contracts.signature,
        }
    })
}
#[cfg(test)]
#[path = "../../../tests/invariants/ranged_headers.rs"]
mod tests;
