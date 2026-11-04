//! Crate permission and actual strict owners select a declaration-only overlap mode.
use crate::{
    resolve::{
        bindings::FeatureName,
        expansion::{MacroAllowanceSession, Report},
        model::DefId,
        package::Error,
        unit::NodeKey,
    },
    sema::model::Declarations,
    support::{hash::HashSet, retained},
    syntax::{
        ast::Kind,
        attribute::{Attribute, Input},
        item::ItemKind,
        token_tree::Forest,
    },
};
#[derive(Clone, Copy, PartialEq, Eq)]
pub(in crate::sema) enum Mode {
    Stable,
    Negative,
    Strict,
}
impl Mode {
    pub(in crate::sema) fn negative(self) -> bool {
        self != Self::Stable
    }
    pub(in crate::sema) fn implicit(self) -> bool {
        self != Self::Strict
    }
}
#[derive(Default)]
pub(super) struct Policy {
    strict: HashSet<DefId>,
    pending: HashSet<NodeKey>,
}
pub(super) fn prepare(report: &Report, facts: &mut Declarations) -> Result<(), Error> {
    if report
        .package
        .requested_negative_coherence()
        .map_err(Error::unlocated)?
    {
        crate::sema::header_attributes::reserve_source_contracts(facts, 1, size_of::<Policy>())?;
        facts.generics.coherence = Some(Box::default());
    }
    Ok(())
}
pub(super) fn consume(
    report: &Report,
    facts: &mut Declarations,
    key: NodeKey,
    attribute: Attribute,
    _forest: &Forest,
    allowances: &mut MacroAllowanceSession<'_>,
) -> Result<(), Error> {
    facts.reserve_must_use_source(key, attribute.span)?;
    let error = |message| Error::at(key.file, attribute.span.start, message);
    if attribute.unsafe_wrapper || !matches!(attribute.input, Input::Empty) {
        return Err(error("strict coherence requires a bare attribute"));
    }
    if !report.package.requested_internal_attributes()
        && !allowances.allows_at(
            key.file,
            attribute.span,
            &FeatureName::new("rustc_attrs").map_err(error)?,
        )?
    {
        return Err(error("strict coherence requires rustc_attrs"));
    }
    let unit = report.package.unit(key.file);
    let Kind::Item(index) = unit.configured.ast.node(key.node).kind else {
        return Err(error("strict coherence requires an actual type or trait"));
    };
    if !matches!(
        unit.configured.ast.items[index as usize].kind,
        ItemKind::Trait { .. }
            | ItemKind::Struct { .. }
            | ItemKind::Enum { .. }
            | ItemKind::Union { .. }
            | ItemKind::ForeignType
    ) {
        return Err(error("strict coherence requires an actual type or trait"));
    }
    let policy = facts
        .generics
        .coherence
        .as_ref()
        .ok_or_else(|| error("strict coherence requires with_negative_coherence"))?;
    if policy.pending.contains(&key) {
        return Err(error("duplicate strict coherence attribute"));
    }
    let growth =
        retained::Growth::map::<NodeKey, ()>(policy.pending.len(), policy.pending.capacity(), 1);
    crate::sema::header_attributes::reserve_source_contracts(facts, growth.work + 1, growth.bytes)?;
    let policy = facts
        .generics
        .coherence
        .as_mut()
        .ok_or_else(|| error("strict coherence lost crate permission"))?;
    policy
        .pending
        .try_reserve(1)
        .map_err(|_| error("strict coherence owner allocation failed"))?;
    policy.pending.insert(key);
    Ok(())
}
pub(super) fn pending(facts: &Declarations) -> bool {
    facts
        .generics
        .coherence
        .as_ref()
        .is_some_and(|p| !p.pending.is_empty())
}
pub(super) fn bind(
    facts: &mut Declarations,
    id: DefId,
    definition: &crate::resolve::model::Definition,
) -> Result<(), Error> {
    use crate::resolve::model::DefKind;
    if !matches!(
        definition.kind,
        DefKind::Trait | DefKind::Struct | DefKind::Enum | DefKind::ForeignType
    ) {
        return Ok(());
    }
    let Some(key) = definition.node else {
        return Ok(());
    };
    let Some(policy) = facts
        .generics
        .coherence
        .as_ref()
        .filter(|p| p.pending.contains(&key))
    else {
        return Ok(());
    };
    let growth =
        retained::Growth::map::<DefId, ()>(policy.strict.len(), policy.strict.capacity(), 1);
    crate::sema::header_attributes::reserve_source_contracts(facts, growth.work + 1, growth.bytes)?;
    let policy = facts
        .generics
        .coherence
        .as_mut()
        .ok_or_else(|| Error::unlocated("strict coherence binding lost its policy"))?;
    policy
        .strict
        .try_reserve(1)
        .map_err(|_| Error::unlocated("strict coherence binding allocation failed"))?;
    policy.strict.insert(id);
    policy.pending.remove(&key);
    Ok(())
}
pub(in crate::sema) fn owner(facts: &Declarations, id: DefId) -> Mode {
    facts
        .generics
        .coherence
        .as_ref()
        .map_or(Mode::Stable, |policy| {
            if policy.strict.contains(&id) {
                Mode::Strict
            } else {
                Mode::Negative
            }
        })
}
pub(in crate::sema) fn require_transport(facts: &Declarations) -> Result<(), &'static str> {
    if facts
        .generics
        .coherence
        .as_ref()
        .is_some_and(|policy| !policy.strict.is_empty() || !policy.pending.is_empty())
    {
        return Err("strict coherence owners require authenticated metadata transport");
    }
    Ok(())
}
pub(in crate::sema) fn predicate(
    facts: &Declarations,
    predicate: crate::types::parameters::predicates::Predicate,
) -> Mode {
    predicate
        .bound
        .trait_parts()
        .and_then(|(id, _)| facts.types.trait_application(id))
        .map_or_else(
            || {
                if facts.generics.coherence.is_some() {
                    Mode::Negative
                } else {
                    Mode::Stable
                }
            },
            |app| owner(facts, DefId(app.item.0)),
        )
}
pub(in crate::sema) fn inherent(facts: &Declarations, id: DefId) -> Mode {
    let id = facts
        .associated
        .schema(id)
        .and_then(|schema| facts.types.kind(schema.self_type))
        .and_then(|kind| match kind {
            crate::types::Kind::Adt(item, _) | crate::types::Kind::Foreign(item) => {
                Some(DefId(item.0))
            }
            _ => None,
        })
        .unwrap_or(id);
    owner(facts, id)
}
#[cfg(test)]
#[path = "../../tests/invariants/negative_coherence.rs"]
mod tests;
