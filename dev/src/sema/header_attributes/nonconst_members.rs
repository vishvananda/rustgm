//! A trait member's runtime restriction is separate from impl constness.
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
/// Every completed local trait function seals both presence and absence.
pub(super) enum Choice {
    Unmarked,
    Marked(Marker),
}
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
pub(super) fn is_trait_member(facts: &Declarations, definition: DefId) -> bool {
    facts
        .associated
        .owner(definition)
        .and_then(|id| facts.associated.get(id))
        .is_some_and(|owner| matches!(owner.kind, OwnerKind::Trait { .. }))
}
/// Reuse the resolver's exact member scope and its existing source owner.
fn validate_owner(report: &Report, facts: &Declarations, definition: DefId) -> Result<(), Error> {
    let owner = facts.associated.owner(definition).and_then(|id| facts.associated.get(id))
        .ok_or_else(|| Error::unlocated("non-const member lost its associated owner"))?;
    let OwnerKind::Trait { definition: parent, constant, .. } = owner.kind else {
        return Err(Error::unlocated("non-const member requires its actual trait owner"));
    };
    let member = report.resolution.definitions.get(definition.0 as usize)
        .filter(|row| row.kind == crate::resolve::model::DefKind::Function)
        .ok_or_else(|| Error::unlocated("non-const member lost its function declaration"))?;
    let parent = report.resolution.definitions.get(parent.0 as usize)
        .filter(|row| row.kind == crate::resolve::model::DefKind::Trait)
        .ok_or_else(|| Error::unlocated("non-const member lost its trait declaration"))?;
    let source = owner.source.ok_or_else(|| Error::unlocated("non-const member has no actual source owner"))?;
    let unit = report.package.unit(source.file);
    if parent.node != Some(source) || parent.members != Some(member.scope)
        || unit.configured.active.get(source.node.get() as usize) != Some(&true)
        || !matches!(unit.configured.ast.node(source.node).kind,
            Kind::Item(index) if matches!(unit.configured.ast.items[index as usize].kind,
                ItemKind::Trait { constant: actual, .. } if actual == constant))
    {
        return Err(Error::unlocated("non-const member changed its actual trait scope or source header"));
    }
    Ok(())
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
        return Err(error(
            "rustc_non_const_trait_method requires a word attribute",
        ));
    }
    let unit = report.package.unit(declaration.file);
    let (_, forest) = unit
        .retained_attributes(effect..effect + 1)
        .next()
        .ok_or_else(|| error("non-const member attribute lost its actual effect"))?;
    let name = forest
        .tree(attribute.path.start)
        .ok_or_else(|| error("non-const member attribute lost its name"))?
        .span;
    if !report.package.requested_internal_attributes()
        && !report.macro_allowances().allows_at(
            declaration.file,
            name,
            &crate::resolve::bindings::FeatureName::new("rustc_attrs").map_err(error)?,
        )?
    {
        return Err(error(
            "rustc_non_const_trait_method requires its rustc_attrs request",
        ));
    }
    let Kind::Item(index) = unit.configured.ast.node(declaration.node).kind else {
        return Err(error(
            "rustc_non_const_trait_method requires a trait function",
        ));
    };
    let ItemKind::Function { flags, .. } = unit.configured.ast.items[index as usize].kind else {
        return Err(error(
            "rustc_non_const_trait_method requires a trait function",
        ));
    };
    if report.package.source_core().is_none() {
        super::observe_selected_owner(report, facts, owner, declaration, true)?;
    }
    let actual = header(facts, declaration)
        .ok_or_else(|| error("non-const member lacks its source header"))?;
    super::charge_source_contract_query(facts, actual.owners.len())?;
    let slot = actual
        .owners
        .iter()
        .position(|row| row.key == owner && row.effects.contains(&effect))
        .ok_or_else(|| error("non-const member belongs to another source owner"))?;
    if actual
        .promises
        .as_deref()
        .is_some_and(|promises| promises.member.is_some())
    {
        return Err(error("duplicate rustc_non_const_trait_method attribute"));
    }
    let bytes = if actual.promises.is_some() {
        0
    } else {
        size_of::<super::Promises>()
    };
    let headers = &mut ledger(facts, bytes)?.local;
    let actual = headers.declarations.get_mut(&declaration)
        .ok_or_else(|| error("non-const member header disappeared"))?;
    actual.promises.get_or_insert_with(Default::default).member = Some(Choice::Marked(Marker {
        owner_slot: slot,
        effect,
        attribute,
        flags,
    }));
    headers.has_special_calls = true;
    Ok(())
}
pub(super) fn bind(
    report: &Report,
    facts: &mut Declarations,
    definition: DefId,
    key: NodeKey,
) -> Result<(), Error> {
    let Some(actual) = header(facts, key) else {
        return Ok(());
    };
    let marked = actual
        .promises
        .as_deref()
        .and_then(|row| row.member.as_ref())
        .is_some();
    if !is_trait_member(facts, definition)
        || report.resolution.definitions[definition.0 as usize].kind
            != crate::resolve::model::DefKind::Function
    {
        if marked {
            return Err(Error::at(
                key.file,
                0,
                "rustc_non_const_trait_method requires a trait function",
            ));
        }
        return Ok(());
    }
    super::charge_source_contract_query(facts, 12)?;
    validate_owner(report, facts, definition)?;
    if marked {
        return Ok(());
    }
    let bytes = if actual.promises.is_some() {
        0
    } else {
        size_of::<super::Promises>()
    };
    let actual = ledger(facts, bytes)?
        .local
        .declarations
        .get_mut(&key)
        .ok_or_else(|| Error::unlocated("non-const member completion lost its header"))?;
    actual.promises.get_or_insert_with(Default::default).member = Some(Choice::Unmarked);
    Ok(())
}
fn validate_marker(
    report: &Report,
    key: NodeKey,
    header: &Header,
    marker: &Marker,
) -> Result<SourceContractSite, Error> {
    let error = |message| Error::at(key.file, marker.attribute.span.start, message);
    let owner = header
        .owners
        .get(marker.owner_slot)
        .ok_or_else(|| error("non-const member owner slot changed"))?;
    if !owner.effects.contains(&marker.effect) {
        return Err(error("non-const member effect belongs to another owner"));
    }
    let unit = report.package.unit(key.file);
    let (attribute, forest) = unit
        .retained_attributes(marker.effect..marker.effect + 1)
        .next()
        .ok_or_else(|| error("non-const member effect disappeared"))?;
    let Kind::Item(index) = unit.configured.ast.node(key.node).kind else {
        return Err(error("non-const member owner changed"));
    };
    if attribute != marker.attribute
        || attribute.name(forest, unit.source.text()) != Some("rustc_non_const_trait_method")
        || !matches!(unit.configured.ast.items[index as usize].kind, ItemKind::Function{flags,..} if flags == marker.flags)
    {
        return Err(error(
            "non-const member source function or attribute changed",
        ));
    }
    Ok(SourceContractSite {
        owner: owner.key,
        declaration: key,
        span: attribute.span,
    })
}
impl Declarations {
    /// Indexed completed classification, never a const-call or body permission.
    pub(in crate::sema) fn nonconst_trait_member(
        &self,
        definition: DefId,
    ) -> Result<bool, &'static str> {
        if !is_trait_member(self, definition) {
            return Ok(false);
        }
        let local = &self
            .must_use_contracts
            .as_deref()
            .and_then(|row| row.headers.as_deref())
            .ok_or("trait member constness lacks its completed source ledger")?
            .local;
        let function = self
            .function_indices
            .get(&definition)
            .and_then(|&index| self.functions.get(index))
            .filter(|function| function.definition == definition)
            .ok_or("trait member constness lacks its actual function")?;
        let actual = local
            .definitions
            .get(&definition)
            .filter(|key| **key == function.node)
            .and_then(|key| local.declarations.get(key))
            .filter(|_| local.comptime_complete)
            .ok_or("trait member constness lacks its completed source header")?;
        match actual
            .promises
            .as_deref()
            .and_then(|row| row.member.as_ref())
        {
            Some(Choice::Marked(_)) => Ok(true),
            Some(Choice::Unmarked) => Ok(false),
            None => Err("trait member constness lost its completed source choice"),
        }
    }
    pub(crate) fn source_non_const_trait_method_contract(
        &self,
        report: &Report,
        definition: DefId,
    ) -> Result<ContractKnowledge<SourceContractSite>, Error> {
        self.require_lint_context(report)?;
        if !is_trait_member(self, definition) {
            return Err(Error::unlocated(
                "non-const member query requires an actual trait function",
            ));
        }
        let Some(key) = report
            .resolution
            .definitions
            .get(definition.0 as usize)
            .and_then(|row| row.node)
        else {
            return Ok(ContractKnowledge::Unknown);
        };
        super::charge_source_contract_query(self, 20)?;
        validate_owner(report, self, definition)?;
        self.nonconst_trait_member(definition)
            .map_err(Error::unlocated)?;
        let actual = header(self, key)
            .ok_or_else(|| Error::unlocated("non-const member query lost its header"))?;
        super::validate_header(report, self, definition, key, actual)?;
        match actual
            .promises
            .as_deref()
            .and_then(|row| row.member.as_ref())
        {
            Some(Choice::Unmarked) => Ok(ContractKnowledge::KnownAbsent),
            Some(Choice::Marked(marker)) => Ok(ContractKnowledge::KnownPresent(validate_marker(
                report, key, actual, marker,
            )?)),
            None => Err(Error::unlocated(
                "non-const member query lost its completed choice",
            )),
        }
    }
}

#[cfg(test)]
#[path = "../../../tests/invariants/nonconst_eligibility.rs"]
mod tests;
