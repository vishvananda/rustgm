//! Public late binders retain the exact original region-slot permutation.
use super::{HeaderIndex, Work, s};

pub(super) fn validate(
    headers: &HeaderIndex<'_>,
    profile: s::Profile,
    work: &mut Work,
) -> Result<(), &'static str> {
    let rows = headers.rows();
    let wide = profile.declaration_policies();
    work.charge(rows.region_slots.len().saturating_add(rows.slots.len()))?;
    let mut covered = vec![false; rows.region_slots.len()];
    let mut stamps = vec![0_u32; rows.slots.len()];
    for (ordinal, header) in rows.functions.iter().enumerate() {
        work.charge(1)?;
        let captures = header.regions.captures;
        let early = header.regions.early.slice(&rows.region_slots)?;
        if captures != 0 && !associated_captures(headers, header, work)?
            || (!wide && !early.is_empty())
        {
            return Err("source early/captured region schemes require a complete wider profile");
        }
        let late = header.regions.late.slice(&rows.region_slots)?;
        let frame = &rows.binders[header.callable.binder.0 as usize];
        let total = headers.scopes().complete_slots(header.scope, work)?;
        let types = headers.scopes().complete_type_slots(header.scope, work)?;
        let used = early
            .len()
            .checked_add(late.len())
            .and_then(|own| own.checked_add(captures as usize));
        let valid_counts = if wide {
            used.is_some_and(|used| used <= header.regions.total as usize)
        } else {
            used == Some(header.regions.total as usize)
        };
        if !valid_counts
            || frame.count != header.regions.total
            || total.checked_sub(types) != Some(header.regions.total)
        {
            return Err("source public binder omitted actual late original region formals");
        }
        let stamp = u32::try_from(ordinal)
            .ok()
            .and_then(|ordinal| ordinal.checked_add(1))
            .ok_or("source region scheme owner stamp overflow")?;
        mark_slots(
            rows,
            header,
            (header.regions.early.start, early, true),
            (stamp, &mut covered, &mut stamps),
            work,
        )?;
        mark_slots(
            rows,
            header,
            (header.regions.late.start, late, false),
            (stamp, &mut covered, &mut stamps),
            work,
        )?;
    }
    work.charge(covered.len())?;
    if covered.contains(&false) {
        return Err("source late region-slot table has incomplete actual header ownership");
    }
    Ok(())
}

fn mark_slots(
    rows: &super::SourceRows,
    header: &s::FunctionHeader,
    (start, slots, require_named): (u32, &[s::SlotRef], bool),
    (stamp, covered, stamps): (u32, &mut [bool], &mut [u32]),
    work: &mut Work,
) -> Result<(), &'static str> {
    for (offset, reference) in slots.iter().enumerate() {
        work.charge(1)?;
        let slot = rows
            .slots
            .get(reference.0 as usize)
            .ok_or("source late region slot is unknown")?;
        if slot.owner != header.declaration
            || slot.kind != s::ParameterKind::Region
            || slot.index < header.regions.captures
            || slot.index >= header.regions.total
            || (require_named && slot.definition.is_none())
        {
            return Err("source late opening has foreign, repeated or nonregional formals");
        }
        let canonical = stamps
            .get_mut(slot.index as usize)
            .ok_or("source late canonical region index exceeds its checked slot budget")?;
        if std::mem::replace(&mut covered[start as usize + offset], true)
            || std::mem::replace(canonical, stamp) == stamp
        {
            return Err("source late opening repeats an actual canonical region index");
        }
    }
    Ok(())
}

/// The method's leading slots match its actual trait or implementation.
/// Its own late openings may name only slots after those captures.
fn associated_captures(
    headers: &HeaderIndex<'_>,
    header: &s::FunctionHeader,
    work: &mut Work,
) -> Result<bool, &'static str> {
    let Some(parent) = header.parent else {
        return Ok(false);
    };
    let rows = headers.rows();
    Ok(match headers.header(parent, work)? {
        Some(s::HeaderRef::Implementation(index)) => {
            let owner = &rows.implementations[index as usize];
            owner.self_definition == parent && owner.region_captures == header.regions.captures
        }
        Some(s::HeaderRef::Trait(index)) => {
            let owner = &rows.traits[index as usize];
            owner.declaration == parent && owner.region_captures == header.regions.captures
        }
        _ => false,
    })
}
