//! A retained declaration promise never grants const execution or permanent storage.
use super::{Declarations, DefId, Error, Header, NodeKey, Report, ledger};
use crate::{
    sema::header_attributes::{ContractKnowledge, SourceContractSite},
    syntax::{
        ast::Kind,
        attribute::Attribute,
        item::{FunctionFlags, ItemKind},
    },
};

pub(super) struct Marker {
    owner_slot: usize,
    const_owner_slot: Option<usize>,
    effect: usize,
    attribute: Attribute,
    flags: FunctionFlags,
}
#[cfg(test)]
#[path = "../../../tests/invariants/promotable_headers.rs"]
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
    if attribute.unsafe_wrapper {
        return Err(error("rustc_promotable is not an unsafe attribute"));
    }
    if report
        .package
        .library_policy()
        .and_then(crate::resolve::stability::Library::staged_inventory)
        .is_none()
    {
        return Err(error(
            "rustc_promotable requires staged_api and its completed inventory",
        ));
    }
    let unit = report.package.unit(declaration.file);
    let Kind::Item(index) = unit.configured.ast.node(declaration.node).kind else {
        return Err(error(
            "source promotable declaration requires its typed item owner",
        ));
    };
    let ItemKind::Function { flags, .. } = unit.configured.ast.items[index as usize].kind else {
        return Err(error(
            "source non-function promotable policy requires its const-owner profile",
        ));
    };
    let actual = header(facts, declaration)
        .ok_or_else(|| error("promotable declaration lacks its recorded source header"))?;
    let count = actual.owners.len();
    super::charge_source_contract_query(facts, count)?;
    let slot = actual
        .owners
        .iter()
        .position(|row| row.key == owner && row.effects.contains(&effect))
        .ok_or_else(|| error("promotable attribute belongs to another source owner"))?;
    let present = actual.promotable.is_some();
    // The pinned parser accepts all parsed forms; their syntax stays in this
    // real effect/attribute witness. Repeated markers coalesce, including unsafe fns.
    let bytes = if present { 0 } else { size_of::<Marker>() };
    ledger(facts, bytes)?;
    let headers = &mut ledger(facts, 0)?.local;
    let actual = headers
        .declarations
        .get_mut(&declaration)
        .ok_or_else(|| error("promotable source header disappeared"))?;
    if !present {
        actual.promotable = Some(Box::new(Marker {
            owner_slot: slot,
            const_owner_slot: None,
            effect,
            attribute,
            flags,
        }));
    }
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
    let Some(marker) = actual.promotable.as_deref() else {
        return Ok(());
    };
    let policy = report
        .package
        .library_policy()
        .and_then(crate::resolve::stability::Library::inventory)
        .and_then(|inventory| inventory.const_declaration(definition))
        .ok_or_else(|| {
            Error::at(
                key.file,
                marker.attribute.span.start,
                "rustc_promotable requires explicit const stability on its actual function",
            )
        })?;
    let count = actual.owners.len();
    super::charge_source_contract_query(facts, count)?;
    let slot = actual
        .owners
        .iter()
        .position(|owner| owner.key == policy.origin)
        .ok_or_else(|| {
            Error::at(
                key.file,
                marker.attribute.span.start,
                "promotable const stability belongs to another declaration",
            )
        })?;
    let actual = ledger(facts, 0)?
        .local
        .declarations
        .get_mut(&key)
        .ok_or_else(|| Error::unlocated("promotable header disappeared"))?;
    let marker = actual
        .promotable
        .as_deref_mut()
        .ok_or_else(|| Error::unlocated("promotable marker disappeared"))?;
    marker.const_owner_slot = Some(slot);
    Ok(())
}
pub(in crate::sema) fn query(
    facts: &Declarations,
    report: &Report,
    definition: DefId,
) -> Result<ContractKnowledge<SourceContractSite>, Error> {
    super::charge_source_contract_query(facts, 5)?;
    let key = report
        .resolution
        .definitions
        .get(definition.0 as usize)
        .and_then(|d| d.node)
        .ok_or_else(|| Error::unlocated("promotable query lacks its actual source declaration"))?;
    let actual = header(facts, key)
        .ok_or_else(|| Error::unlocated("promotable query lacks its completed header"))?;
    let Some(marker) = actual.promotable.as_deref() else {
        return Ok(ContractKnowledge::KnownAbsent);
    };
    let owner = actual
        .owners
        .get(marker.owner_slot)
        .ok_or_else(|| Error::unlocated("promotable owner slot changed"))?;
    let unit = report.package.unit(key.file);
    let error = |message| Error::at(key.file, marker.attribute.span.start, message);
    if !owner.effects.contains(&marker.effect) {
        return Err(error("promotable effect belongs to another owner"));
    }
    let (attribute, forest) = unit
        .retained_attributes(marker.effect..marker.effect + 1)
        .next()
        .ok_or_else(|| error("promotable attribute disappeared"))?;
    let Kind::Item(index) = unit.configured.ast.node(key.node).kind else {
        return Err(error("promotable function owner changed"));
    };
    if attribute != marker.attribute
        || attribute.name(forest, unit.source.text()) != Some("rustc_promotable")
        || !matches!(unit.configured.ast.items[index as usize].kind,ItemKind::Function{flags,..} if flags==marker.flags)
        || !facts.is_const_function(definition)
    {
        return Err(error("promotable source function or attribute changed"));
    }
    let policy = report
        .package
        .library_policy()
        .and_then(crate::resolve::stability::Library::inventory)
        .and_then(|inventory| inventory.const_declaration(definition))
        .ok_or_else(|| error("promotable const policy disappeared"))?;
    if marker
        .const_owner_slot
        .and_then(|index| actual.owners.get(index))
        .is_none_or(|row| row.key != policy.origin)
    {
        return Err(error("promotable const policy owner changed"));
    }
    Ok(ContractKnowledge::KnownPresent(SourceContractSite {
        owner: owner.key,
        declaration: key,
        span: attribute.span,
    }))
}
