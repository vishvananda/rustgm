//! Raw numbering structure is separate from canonical owner/environment binding.
use super::{HeaderIndex, HeaderRef, SourceRows, Work};
use crate::support::crates::{MAX_BYTES, generics::source as s, owner_catalog::BodyRef};

/// Borrowed complete raw units. Later semantic staging must independently bind
/// their actual catalog, arena, body, signatures and required alias presence.
pub struct NumberingUnits<'rows> {
    rows: &'rows SourceRows,
    peak: usize,
}
impl<'rows> NumberingUnits<'rows> {
    /// Build one dense inverse of an already checked allocation permutation.
    /// `extra_bytes` accounts for additional caller-owned retained/scratch data
    /// beyond this checked view's existing combined peak. The result contains
    /// only zero-based ordinals, never canonical regions or imported authority.
    /// # Errors
    /// Rejects unknown bodies and charges attempted work/storage before allocation.
    pub fn allocation_ordinals(
        &self,
        body: BodyRef,
        extra_bytes: usize,
        work: &mut Work,
    ) -> Result<super::AllocationOrdinals<'rows>, &'static str> {
        let unit = self.unit(body, work)?;
        super::allocation_ordinals::build(unit, self.peak, extra_bytes, work)
    }
    /// # Errors
    /// Rejects an unknown body and charges failed queries too.
    pub fn unit(&self, body: BodyRef, work: &mut Work) -> Result<&'rows s::BodyUnit, &'static str> {
        work.charge(1)?;
        self.rows
            .body_units
            .get(body.0 as usize)
            .ok_or("source numbering unit has an unknown actual body")
    }
    #[must_use]
    pub const fn peak_bytes(&self) -> usize {
        self.peak
    }
}

pub(super) fn validate<'rows>(
    headers: &HeaderIndex<'rows>,
    work: &mut Work,
) -> Result<NumberingUnits<'rows>, &'static str> {
    work.charge(1)?;
    let rows = headers.rows();
    if rows.body_units.len() != rows.originals.len() {
        return Err("source numbering units omit part of the complete original family");
    }
    let mut peak = headers.peak_bytes();
    for (index, unit) in rows.body_units.iter().enumerate() {
        work.charge(1)?;
        let original = &rows.originals[index];
        let input = &rows.body_inputs[index];
        let Some(HeaderRef::Function(function)) = headers.header(original.declaration, work)?
        else {
            return Err("source numbering unit lacks its actual function header");
        };
        let function = &rows.functions[function as usize];
        let frame = rows
            .binders
            .get(function.original.binder.0 as usize)
            .ok_or("source numbering unit lacks its actual original binder")?;
        if unit.body != original.body
            || unit.body.0 as usize != index
            || function.body != s::FunctionBody::Original(unit.body)
            || input.frame != function.original.binder
            || unit.universals != function.regions.total
            || frame.count != unit.universals
            || unit.allocation_order.len() != original.variables.len()
            || unit.allocation_order.len() >= 0x7fff_ffff
        {
            return Err("source numbering unit changed its complete body/signature domain");
        }
        // The scratch is local to one unit. All lengths and attempted work are
        // charged before allocation, including a rejected oversized alias row.
        let aliases = unit.aliases.as_ref().map_or(0, Vec::len);
        work.charge(original.variables.len().saturating_add(aliases))?;
        let scratch = original
            .variables
            .len()
            .saturating_mul(size_of::<u8>() + size_of::<Option<u32>>())
            .saturating_add(aliases);
        let next = headers
            .retained_bytes()
            .saturating_add(size_of::<NumberingUnits<'_>>())
            .saturating_add(scratch);
        if next > MAX_BYTES {
            return Err("source numbering validation exceeds combined storage budget");
        }
        peak = peak.max(next);
        validate_unit(rows, original, input, unit, function.original.binder, work)?;
    }
    Ok(NumberingUnits { rows, peak })
}

fn validate_unit(
    rows: &SourceRows,
    original: &s::Original,
    input: &s::BodyInputs,
    unit: &s::BodyUnit,
    binder: crate::support::crates::generics::BinderRef,
    work: &mut Work,
) -> Result<(), &'static str> {
    let mut seen = vec![0_u8; original.variables.len()];
    for &slot in &unit.allocation_order {
        work.charge(1)?;
        let used = seen
            .get_mut(slot as usize)
            .ok_or("source allocation order names an unknown owned variable")?;
        if std::mem::replace(used, 1) != 0 {
            return Err("source allocation order is not an exact variable permutation");
        }
    }
    let Some(aliases) = &unit.aliases else {
        // Absence remains an unpaid claim. Only the later actual defining
        // producer can determine whether this body requires an alias ledger.
        return Ok(());
    };
    if aliases.len() != unit.universals as usize {
        return Err("source universal aliases omit declaration slots, including unused slots");
    }
    let mut slots = vec![None; original.variables.len()];
    for (index, &slot) in aliases.iter().enumerate() {
        work.charge(1)?;
        let variable = original
            .variables
            .get(slot as usize)
            .ok_or("source universal alias is outside its actual allocation domain")?;
        if variable.allocation
            != (s::RegionAllocation::Inference {
                needs_provenance: false,
            })
            || slots[slot as usize]
                .replace(u32::try_from(index).map_err(|_| "source alias ordinal overflow")?)
                .is_some()
        {
            return Err("source universal alias is duplicated or has another allocation role");
        }
    }
    let mut matched = vec![false; aliases.len()];
    for obligation in &input.obligations {
        work.charge(1)?;
        match obligation.kind {
            s::InputKind::Variable {
                slot,
                needs_provenance,
            } => {
                let Some(Some(_)) = slots.get(slot as usize) else {
                    continue;
                };
                if needs_provenance || seen[slot as usize] & 2 != 0 {
                    return Err("source alias lost its exact ordinary variable input");
                }
                seen[slot as usize] |= 2;
            }
            s::InputKind::Equality(pair) => {
                alias_relation(rows, unit.body, binder, pair, &slots, &mut matched)?;
                alias_relation(
                    rows,
                    unit.body,
                    binder,
                    s::RegionBound {
                        longer: pair.shorter,
                        shorter: pair.longer,
                    },
                    &slots,
                    &mut matched,
                )?;
            }
            _ => {}
        }
    }
    work.charge(aliases.len())?;
    if aliases.iter().any(|&slot| seen[slot as usize] & 2 == 0) || matched.contains(&false) {
        return Err("source alias lacks its exact variable/declaration-bound relation");
    }
    Ok(())
}

fn alias_relation(
    rows: &SourceRows,
    body: BodyRef,
    binder: crate::support::crates::generics::BinderRef,
    pair: s::RegionBound,
    slots: &[Option<u32>],
    matched: &mut [bool],
) -> Result<(), &'static str> {
    let longer = rows
        .regions
        .get(pair.longer.0 as usize)
        .ok_or("source alias relation has an unknown region")?;
    let shorter = rows
        .regions
        .get(pair.shorter.0 as usize)
        .ok_or("source alias relation has an unknown region")?;
    if let (
        &s::Region::Variable { body: actual, slot },
        &s::Region::Bound {
            binder: frame,
            depth: 0,
            slot: bound,
        },
    ) = (longer, shorter)
        && actual == body
        && frame == binder
        && let Some(Some(expected)) = slots.get(slot as usize)
        && *expected == bound
    {
        matched[*expected as usize] = true;
    }
    Ok(())
}
