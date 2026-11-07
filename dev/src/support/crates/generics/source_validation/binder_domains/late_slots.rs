//! Public late binders retain the exact original region-slot permutation.
use super::{HeaderIndex, Work, s};

pub(super) fn validate(headers: &HeaderIndex<'_>, work: &mut Work) -> Result<(), &'static str> {
    let rows = headers.rows();
    work.charge(rows.region_slots.len().saturating_add(rows.slots.len()))?;
    let mut covered = vec![false; rows.region_slots.len()];
    let mut stamps = vec![0_u32; rows.slots.len()];
    for (ordinal, header) in rows.functions.iter().enumerate() {
        work.charge(1)?;
        let captures = header.regions.captures;
        if captures != 0 && !inherent_captures(headers, header, work)?
            || !header.regions.early.slice(&rows.region_slots)?.is_empty()
        {
            return Err("source early/captured region schemes require a complete wider profile");
        }
        let late = header.regions.late.slice(&rows.region_slots)?;
        let frame = &rows.binders[header.callable.binder.0 as usize];
        let total = headers.scopes().complete_slots(header.scope, work)?;
        let types = headers.scopes().complete_type_slots(header.scope, work)?;
        if late.len().checked_add(captures as usize) != Some(header.regions.total as usize)
            || frame.count != header.regions.total
            || total.checked_sub(types) != Some(header.regions.total)
        {
            return Err("source public binder omitted actual late original region formals");
        }
        let stamp = u32::try_from(ordinal)
            .ok()
            .and_then(|ordinal| ordinal.checked_add(1))
            .ok_or("source region scheme owner stamp overflow")?;
        for (offset, reference) in late.iter().enumerate() {
            work.charge(1)?;
            let slot = rows
                .slots
                .get(reference.0 as usize)
                .ok_or("source late region slot is unknown")?;
            if slot.owner != header.declaration
                || slot.kind != s::ParameterKind::Region
                || slot.index < captures
                || slot.index >= header.regions.total
            {
                return Err("source late opening has foreign, repeated or nonregional formals");
            }
            let canonical = stamps
                .get_mut(slot.index as usize)
                .ok_or("source late canonical region index exceeds its checked slot budget")?;
            if std::mem::replace(
                &mut covered[header.regions.late.start as usize + offset],
                true,
            ) || std::mem::replace(canonical, stamp) == stamp
            {
                return Err("source late opening repeats an actual canonical region index");
            }
        }
    }
    work.charge(covered.len())?;
    if covered.contains(&false) {
        return Err("source late region-slot table has incomplete actual header ownership");
    }
    Ok(())
}

/// Version thirteen: a method's leading captured slots are exactly its
/// inherent impl's lifetimes. Its own late slots follow them; the impl owns
/// the captured slots, so no late opening may name one.
fn inherent_captures(
    headers: &HeaderIndex<'_>,
    header: &s::FunctionHeader,
    work: &mut Work,
) -> Result<bool, &'static str> {
    let Some(parent) = header.parent else {
        return Ok(false);
    };
    let Some(s::HeaderRef::Implementation(index)) = headers.header(parent, work)? else {
        return Ok(false);
    };
    let implementation = &headers.rows().implementations[index as usize];
    Ok(implementation.self_definition == parent
        && implementation.trait_application.is_none()
        && implementation.region_captures == header.regions.captures)
}
