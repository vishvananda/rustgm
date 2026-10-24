//! Public late binders retain the exact original region-slot permutation.
use super::{HeaderIndex, Work, s};

pub(super) fn validate(headers: &HeaderIndex<'_>, work: &mut Work) -> Result<(), &'static str> {
    let rows = headers.rows();
    work.charge(rows.region_slots.len().saturating_add(rows.slots.len()))?;
    let mut covered = vec![false; rows.region_slots.len()];
    let mut stamps = vec![0_u32; rows.slots.len()];
    for (ordinal, header) in rows.functions.iter().enumerate() {
        work.charge(1)?;
        if header.regions.captures != 0
            || !header.regions.early.slice(&rows.region_slots)?.is_empty()
        {
            return Err("source early/captured region schemes require a complete wider profile");
        }
        let late = header.regions.late.slice(&rows.region_slots)?;
        let frame = &rows.binders[header.callable.binder.0 as usize];
        let total = headers.scopes().complete_slots(header.scope, work)?;
        let types = headers.scopes().complete_type_slots(header.scope, work)?;
        if late.len() != header.regions.total as usize
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
                || slot.index >= header.regions.total
                || std::mem::replace(
                    &mut covered[header.regions.late.start as usize + offset],
                    true,
                )
                || std::mem::replace(&mut stamps[reference.0 as usize], stamp) == stamp
            {
                return Err("source late opening has foreign, repeated or nonregional formals");
            }
        }
    }
    work.charge(covered.len())?;
    if covered.contains(&false) {
        return Err("source late region-slot table has incomplete actual header ownership");
    }
    Ok(())
}
