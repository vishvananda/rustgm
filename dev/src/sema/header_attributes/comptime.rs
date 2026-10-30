//! Always-const source policy is independent of API, safety and pointer calls.
use super::{Declarations, DefId, Error, Header, NodeKey, Report, ledger};
use crate::{
    sema::{
        associated::OwnerKind,
        header_attributes::{ContractKnowledge, SourceContractSite},
    },
    syntax::{
        ast::Kind,
        attribute::{Attribute, Input},
        item::{FunctionFlags, ItemKind},
    },
};

pub(super) struct Marker {
    owner_slot: usize,
    effect: usize,
    attribute: Attribute,
    flags: FunctionFlags,
}
#[cfg(test)]
#[path = "../../../tests/invariants/always_const.rs"]
mod tests;
fn header(facts: &Declarations, key: NodeKey) -> Option<&Header> {
    facts
        .must_use_contracts
        .as_deref()?
        .headers
        .as_deref()?
        .local
        .declarations
        .get(&key)
}
/// An absent sparse row cannot erase an effective non-written const policy.
fn validate_absence(
    facts: &Declarations,
    report: &Report,
    definition: DefId,
    key: NodeKey,
) -> Result<(), Error> {
    if facts
        .declaration_header(definition)
        .map_err(Error::unlocated)?
        .is_none_or(|header| header.declaration.source() != Some(key))
    {
        return Err(Error::unlocated(
            "comptime absence lacks its actual completed function",
        ));
    }
    let unit = report.package.unit(key.file);
    let Kind::Item(index) = unit.configured.ast.node(key.node).kind else {
        return Err(Error::unlocated(
            "comptime absence changed its actual owner",
        ));
    };
    let ItemKind::Function { flags, .. } = unit.configured.ast.items[index as usize].kind else {
        return Err(Error::unlocated(
            "comptime absence requires its function syntax",
        ));
    };
    let const_impl = facts
        .associated
        .owner(definition)
        .and_then(|id| facts.associated.get(id))
        .is_some_and(|owner| matches!(owner.kind, OwnerKind::Impl { constant: true, .. }));
    if facts.is_const_function(definition) && !flags.constant && !const_impl {
        return Err(Error::unlocated(
            "effective comptime function lost its source marker",
        ));
    }
    Ok(())
}
fn validate_marker(
    report: &Report,
    key: NodeKey,
    header: &Header,
    marker: &Marker,
) -> Result<SourceContractSite, Error> {
    let owner = header
        .owners
        .get(marker.owner_slot)
        .ok_or_else(|| Error::unlocated("comptime owner slot changed"))?;
    let unit = report.package.unit(key.file);
    let error = |message| Error::at(key.file, marker.attribute.span.start, message);
    if !owner.effects.contains(&marker.effect) {
        return Err(error("comptime effect belongs to another owner"));
    }
    let (attribute, forest) = unit
        .retained_attributes(marker.effect..marker.effect + 1)
        .next()
        .ok_or_else(|| error("comptime effect disappeared"))?;
    let Kind::Item(index) = unit.configured.ast.node(key.node).kind else {
        return Err(error("comptime owner changed"));
    };
    if attribute != marker.attribute
        || attribute.name(forest, unit.source.text()) != Some("rustc_comptime")
        || !matches!(unit.configured.ast.items[index as usize].kind,ItemKind::Function{flags,..} if flags==marker.flags && !flags.constant)
    {
        return Err(error("comptime source function or attribute changed"));
    }
    Ok(SourceContractSite {
        owner: owner.key,
        declaration: key,
        span: attribute.span,
    })
}
pub(in crate::sema) fn consume(
    report: &Report,
    facts: &mut Declarations,
    owner: NodeKey,
    declaration: NodeKey,
    attribute: Attribute,
    effect: usize,
) -> Result<(), Error> {
    facts.reserve_must_use_source(declaration, attribute.span)?;
    let error = |message| Error::at(owner.file, attribute.span.start, message);
    if attribute.unsafe_wrapper || attribute.input != Input::Empty {
        return Err(error("rustc_comptime requires a word attribute"));
    }
    let unit = report.package.unit(declaration.file);
    let (_, forest) = unit
        .retained_attributes(effect..effect + 1)
        .next()
        .ok_or_else(|| error("comptime attribute lost its actual effect"))?;
    let name = forest
        .tree(attribute.path.start)
        .ok_or_else(|| error("comptime attribute lost its name"))?
        .span;
    if !report.package.requested_internal_attributes()
        && !report.macro_allowances().allows_at(
            declaration.file,
            name,
            &crate::resolve::bindings::FeatureName::new("rustc_attrs").map_err(error)?,
        )?
    {
        return Err(error("rustc_comptime requires its rustc_attrs request"));
    }
    let Kind::Item(index) = unit.configured.ast.node(declaration.node).kind else {
        return Err(error("rustc_comptime requires a free or inherent function"));
    };
    let ItemKind::Function { flags, .. } = unit.configured.ast.items[index as usize].kind else {
        return Err(error("rustc_comptime requires a free or inherent function"));
    };
    if flags.constant {
        return Err(error("a function cannot be both comptime and const"));
    }
    if report.package.source_core().is_none() {
        super::observe_selected_owner(report, facts, owner, declaration, true)?;
    }
    let actual = header(facts, declaration)
        .ok_or_else(|| error("comptime declaration lacks its source header"))?;
    super::charge_source_contract_query(facts, actual.owners.len())?;
    let slot = actual
        .owners
        .iter()
        .position(|row| row.key == owner && row.effects.contains(&effect))
        .ok_or_else(|| error("comptime attribute belongs to another source owner"))?;
    if actual
        .promises
        .as_deref()
        .is_some_and(|promises| promises.comptime.is_some())
    {
        return Err(error("duplicate rustc_comptime attribute"));
    }
    let bytes = if actual.promises.is_some() {
        0
    } else {
        size_of::<super::Promises>()
    };
    let headers = &mut ledger(facts, bytes)?.local;
    headers.has_comptime = true;
    let actual = headers
        .declarations
        .get_mut(&declaration)
        .ok_or_else(|| error("comptime source header disappeared"))?;
    actual
        .promises
        .get_or_insert_with(Default::default)
        .comptime = Some(Marker {
        owner_slot: slot,
        effect,
        attribute,
        flags,
    });
    Ok(())
}
pub(super) fn bind(
    report: &Report,
    facts: &Declarations,
    definition: DefId,
    key: NodeKey,
) -> Result<(), Error> {
    let Some(marker) = header(facts, key)
        .and_then(|row| row.promises.as_deref())
        .and_then(|row| row.comptime.as_ref())
    else {
        return Ok(());
    };
    super::charge_source_contract_query(facts, 2)?;
    if report.resolution.foreign_owners.contains_key(&definition)
        || facts
            .associated
            .owner(definition)
            .and_then(|id| facts.associated.get(id))
            .is_some_and(|owner| {
                !matches!(
                    owner.kind,
                    OwnerKind::Impl {
                        trait_type: None,
                        ..
                    }
                )
            })
    {
        return Err(Error::at(
            key.file,
            marker.attribute.span.start,
            "rustc_comptime requires a free or inherent function, not a trait or foreign member",
        ));
    }
    Ok(())
}
impl Declarations {
    pub(in crate::sema) fn validate_comptime_parameters(
        &self,
        report: &Report,
        definition: DefId,
    ) -> Result<(), Error> {
        use crate::types::parameters::{clauses::Clause, predicates::Bound};
        for predicate in self
            .generics
            .predicates
            .get(&definition)
            .into_iter()
            .flatten()
        {
            super::charge_source_contract_query(self, 1)?;
            let bound = match predicate.value {
                Clause::Predicate(predicate) => Some(predicate.bound),
                Clause::ForAll(id) => Some(
                    self.types
                        .quantified(id)
                        .ok_or_else(|| Error::unlocated("comptime quantified declaration changed"))?
                        .predicate
                        .bound,
                ),
                Clause::UnstableFeature(_) => None,
            };
            if matches!(bound, Some(Bound::MaybeConstTrait(_))) {
                return Err(Error::at(
                    predicate.source.file,
                    report
                        .package
                        .unit(predicate.source.file)
                        .configured
                        .ast
                        .node(predicate.source.node)
                        .span
                        .start,
                    "conditional const bounds are not allowed on a comptime function",
                ));
            }
        }
        Ok(())
    }
    /// Existing effective constness also survives loss of optional marker data.
    pub(in crate::sema) fn comptime_calls_need_checking(&self) -> bool {
        !self.constant_functions.is_empty()
            || self
                .must_use_contracts
                .as_deref()
                .and_then(|owner| owner.headers.as_deref())
                .is_some_and(|ledger| ledger.local.has_comptime)
    }
    /// Classification from an already bound declaration; never a call permission.
    pub(crate) fn is_comptime_function(&self, definition: DefId) -> bool {
        self.must_use_contracts
            .as_deref()
            .and_then(|owner| owner.headers.as_deref())
            .filter(|ledger| ledger.local.comptime_complete)
            .and_then(|ledger| {
                ledger
                    .local
                    .definitions
                    .get(&definition)
                    .and_then(|key| ledger.local.declarations.get(key))
            })
            .and_then(|header| header.promises.as_deref())
            .is_some_and(|promises| promises.comptime.is_some())
    }
    pub(crate) fn source_comptime_contract(
        &self,
        report: &Report,
        definition: DefId,
    ) -> Result<ContractKnowledge<SourceContractSite>, Error> {
        self.require_lint_context(report)?;
        let actual = report
            .resolution
            .definitions
            .get(definition.0 as usize)
            .filter(|row| row.kind == crate::resolve::model::DefKind::Function)
            .ok_or_else(|| Error::unlocated("comptime query requires an actual function"))?;
        let Some(key) = actual.node else {
            // Canonical factory methods have no source header to borrow. Their
            // checked registry/owner cannot carry an authored comptime marker.
            if super::super::bootstrap::owner(report, definition)? {
                return Ok(ContractKnowledge::KnownAbsent);
            }
            // Every admitted current wire profile encodes ordinary constness;
            // the producer fences Always before publishing any function row.
            let header = self
                .function_header(definition)
                .map_err(Error::unlocated)?
                .ok_or_else(|| {
                    Error::unlocated("foreign comptime query requires its actual callable")
                })?;
            if header.declaration.source().is_some() {
                return Err(Error::unlocated(
                    "foreign comptime query changed its actual declaration origin",
                ));
            }
            return Ok(ContractKnowledge::KnownAbsent);
        };
        let local = self
            .must_use_contracts
            .as_deref()
            .and_then(|owner| owner.headers.as_deref())
            .map(|ledger| &ledger.local);
        if local.is_some() {
            super::charge_source_contract_query(self, 8)?;
        }
        if local.is_some_and(|headers| !headers.comptime_complete) {
            return Err(Error::unlocated(
                "comptime source attribute coverage is pending",
            ));
        }
        let Some(header) = header(self, key) else {
            return if report.package.source_core().is_none() {
                validate_absence(self, report, definition, key)?;
                Ok(ContractKnowledge::KnownAbsent)
            } else {
                Err(Error::unlocated(
                    "comptime query lost its actual source header",
                ))
            };
        };
        let headers = &self
            .must_use_contracts
            .as_deref()
            .and_then(|owner| owner.headers.as_deref())
            .ok_or_else(|| Error::unlocated("comptime query lost its ledger"))?
            .local;
        if !headers.comptime_complete || headers.definitions.get(&definition) != Some(&key) {
            return Err(Error::unlocated(
                "comptime declaration identity is pending or changed",
            ));
        }
        super::validate_header(report, self, definition, key, header)?;
        let Some(marker) = header
            .promises
            .as_deref()
            .and_then(|row| row.comptime.as_ref())
        else {
            validate_absence(self, report, definition, key)?;
            return Ok(ContractKnowledge::KnownAbsent);
        };
        bind(report, self, definition, key)?;
        Ok(ContractKnowledge::KnownPresent(validate_marker(
            report, key, header, marker,
        )?))
    }
}
