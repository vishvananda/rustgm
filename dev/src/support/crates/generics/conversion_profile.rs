//! Version nine admits actual conversion declarations and ordinary impl methods.
use super::{Constness, FunctionBody, HeaderIndex, SourceRows, TraitFlags, Work};
use crate::support::crates::{generics::source as s, owner_catalog::DeclarationRef};

pub(super) fn trait_header(
    rows: &SourceRows,
    headers: &HeaderIndex<'_>,
    header: &s::TraitHeader,
    work: &mut Work,
) -> Result<(), &'static str> {
    work.charge(6)?;
    if header.written_parameters != 1
        || header.region_captures != 0
        || !header.region_bounds.slice(&rows.region_bounds)?.is_empty()
        || headers.scopes().complete_slots(header.scope, work)? != 2
        || headers.scopes().complete_type_slots(header.scope, work)? != 2
    {
        return Err("source From transport requires its actual Self and one type formal");
    }
    let Some(s::Type::Parameter(id)) = rows.types.get(header.self_type.0 as usize) else {
        return Err("source From transport lacks its canonical Self parameter");
    };
    let slot = rows
        .slots
        .get(id.0 as usize)
        .ok_or("source From Self slot is absent")?;
    if slot.owner != header.self_definition
        || slot.definition != Some(header.self_definition)
        || slot.ordinal != 0
        || slot.index != 0
        || !matches!(slot.kind, s::ParameterKind::Type { .. })
    {
        return Err("source From transport changed its actual canonical Self owner");
    }
    Ok(())
}

pub(super) fn required_member(
    rows: &SourceRows,
    headers: &HeaderIndex<'_>,
    function: &s::FunctionHeader,
    role: Option<DeclarationRef>,
    work: &mut Work,
) -> Result<bool, &'static str> {
    if function.parent.is_none() || function.parent != role || function.body != FunctionBody::Absent
    {
        return Ok(false);
    }
    let parent = function
        .parent
        .ok_or("source required conversion lost its trait")?;
    let Some(s::HeaderRef::Trait(index)) = headers.header(parent, work)? else {
        return Err("source required conversion has a nontrait parent");
    };
    work.charge(1)?;
    Ok(
        rows.traits[index as usize].flags.bits() & TraitFlags::CONST != 0
            && function.constness == Constness::Conditional,
    )
}

pub(super) fn method(
    rows: &SourceRows,
    headers: &HeaderIndex<'_>,
    function: &s::FunctionHeader,
    work: &mut Work,
) -> Result<bool, &'static str> {
    let Some(parent) = function.parent else {
        return Ok(false);
    };
    let Some(s::HeaderRef::Implementation(index)) = headers.header(parent, work)? else {
        return Ok(false);
    };
    let implementation = &rows.implementations[index as usize];
    work.charge(5)?;
    if implementation.self_definition != parent
        || implementation.constant
        || implementation.negative
        || implementation.unsafe_
        || implementation.region_captures != 0
        || !implementation
            .region_bounds
            .slice(&rows.region_bounds)?
            .is_empty()
        || function.constness != Constness::Ordinary
        || function.regions.captures != 0
        || function.regions.early.start != function.regions.early.end
    {
        return Ok(false);
    }
    let scopes = headers.scopes();
    if scopes.complete_slots(implementation.scope, work)?
        != scopes.complete_type_slots(implementation.scope, work)?
        || scopes
            .complete_slots(function.scope, work)?
            .checked_sub(scopes.complete_type_slots(function.scope, work)?)
            != Some(function.regions.total)
    {
        return Ok(false);
    }
    // The shared late-slot validator authenticates this exact difference as
    // this declaration's original/public late region-slot permutation. It is
    // not an early or captured generic lifetime transport permission.
    if let Some(receiver) = function.receiver {
        work.charge(2)?;
        if receiver.base != implementation.self_type
            || receiver.references > 1
            || receiver.formal as usize >= function.formals.slice(&rows.formals)?.len()
        {
            return Err("source method receiver differs from its actual implementation Self");
        }
    }
    Ok(true)
}
