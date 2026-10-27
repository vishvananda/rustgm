//! Target requirements are source-owned callable policy, not CPU/backend capability.
mod cpu_origins;
mod feature_sets;
mod parsing;
use crate::{
    resolve::{
        expansion::Report,
        model::{DefId, DefKind},
        package::Error,
        unit::NodeKey,
    },
    sema::{
        header_attributes::{self, SourceImage},
        model::Declarations,
    },
    support::{hash::HashMap, retained::Growth},
};
pub(crate) use feature_sets::FeatureSet;
pub(super) type Sources = HashMap<NodeKey, Contract>;
pub(super) type Functions = HashMap<DefId, Contract>;
struct Site {
    owner: NodeKey,
    effect: usize,
    capture: u32,
    attribute: crate::syntax::attribute::Attribute,
    input: Vec<(u32, crate::syntax::token_tree::Tree)>,
}
pub(super) struct Contract {
    source: NodeKey,
    image: SourceImage,
    sites: Vec<Site>,
    features: FeatureSet,
    declared_unsafe: bool,
}
pub(super) use cpu_origins::{body_unsafe, validate_method};
pub(super) use parsing::collect;
fn pay(facts: &mut Declarations, work: usize, bytes: usize) -> Result<(), Error> {
    header_attributes::reserve_source_contracts(facts, work, bytes)
}
pub(super) fn publish(
    facts: &mut Declarations,
    sources: &mut Sources,
    key: NodeKey,
    definition: DefId,
) -> Result<(), Error> {
    let Some(row) = sources.remove(&key) else {
        return Ok(());
    };
    let rows = &facts.target_feature_functions;
    if rows.contains_key(&definition) {
        return Err(Error::unlocated(
            "duplicate target-feature function completion",
        ));
    }
    let growth = Growth::map::<DefId, Contract>(rows.len(), rows.capacity(), 1);
    pay(facts, 1 + growth.work, growth.bytes)?;
    facts
        .target_feature_functions
        .try_reserve(1)
        .map_err(|_| Error::unlocated("target-feature function allocation failed"))?;
    facts.target_feature_functions.insert(definition, row);
    Ok(())
}
/// Unknown imported policy cannot authorize contextual safety weakening.
pub(super) enum Policy<'a> {
    Local(Option<&'a Contract>),
    ImportedUnknown,
}
pub(super) fn query<'a>(
    report: &Report,
    facts: &'a Declarations,
    definition: DefId,
) -> Result<Policy<'a>, Error> {
    let actual = report
        .resolution
        .definitions
        .get(definition.0 as usize)
        .ok_or_else(|| Error::unlocated("unknown target-feature declaration"))?;
    if actual.kind != DefKind::Function {
        return Err(Error::unlocated(
            "target-feature policy requires a function",
        ));
    }
    let Some(header) = facts
        .declaration_header(definition)
        .map_err(Error::unlocated)?
    else {
        return Err(Error::unlocated(
            "target-feature policy lacks an actual function header",
        ));
    };
    let Some(source) = header.declaration.source() else {
        if facts.target_feature_functions.contains_key(&definition) {
            return Err(Error::unlocated(
                "local target-feature row changed to imported origin",
            ));
        }
        return Ok(Policy::ImportedUnknown);
    };
    if actual.node != Some(source) {
        return Err(Error::unlocated("target-feature declaration owner changed"));
    }
    if let Some(row) = facts.target_feature_functions.get(&definition) {
        row.current(report, facts, source)?;
        return Ok(Policy::Local(Some(row)));
    }
    if report.package.source_core().is_some() {
        let view = facts.header_attribute_contracts(report, definition)?;
        if matches!(view.must_use, header_attributes::ContractKnowledge::Unknown) {
            return Err(Error::unlocated(
                "target-feature absence lacks its completed source header",
            ));
        }
    }
    Ok(Policy::Local(None))
}
impl Contract {
    pub(super) const fn features(&self) -> FeatureSet {
        self.features
    }
    pub(super) const fn declared_unsafe(&self) -> bool {
        self.declared_unsafe
    }
}

#[cfg(test)]
#[path = "../../tests/invariants/cpu_policy.rs"]
mod tests;

/// Reached publication/execution replays the sparse retained rows, not source parsing.
pub(crate) fn authenticate(report: &Report, facts: &Declarations) -> Result<(), Error> {
    if facts.target_feature_functions.is_empty() {
        return Ok(());
    }
    header_attributes::charge_source_contract_query(
        facts,
        facts.target_feature_functions.capacity(),
    )?;
    for &definition in facts.target_feature_functions.keys() {
        if !matches!(query(report, facts, definition)?, Policy::Local(Some(_))) {
            return Err(Error::unlocated(
                "target-feature completion changed its declaration origin",
            ));
        }
    }
    Ok(())
}
