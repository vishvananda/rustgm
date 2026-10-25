//! Preserve actual allocation order and actual alias membership independently.
use super::{Body, Context, Contract, Declarations, DefId, RegionId, Stage, s};
use crate::sema::model::BodyRegionOrigin;
use crate::support::crates::owner_catalog::BodyRef;

pub(super) fn emit(
    stage: &mut Stage,
    facts: &Declarations,
    body: &Body,
    definition: DefId,
    input: &Contract,
    context: Context,
    variables: usize,
) -> Result<s::BodyUnit, &'static str> {
    stage.charge(1)?;
    let source = body
        .source
        .source()
        .ok_or("source numbering requires its actual source body")?;
    let owner = context
        .body()
        .ok_or("source numbering lacks an original body")?;
    if facts.body_regions.unit_of(source) != source {
        return Err("nested source numbering requires complete parent/unit transport");
    }
    let mut frontier = facts.body_regions.frontier(source)?;
    for proof in &input.proofs {
        for range in proof.proof.owned_regions() {
            stage.charge(1)?;
            frontier = frontier.max(range.end);
        }
    }
    let count = frontier
        .checked_sub(1)
        .ok_or("source numbering frontier precedes its domain")? as usize;
    if count != variables {
        return Err("source numbering omits actual allocations from the original contract");
    }
    let header = facts
        .function_header(definition)?
        .ok_or("source numbering lost its actual callable header")?;
    let aliases = header
        .environment
        .and_then(|environment| environment.body_aliases());
    let alias_count = aliases.map_or(0, |aliases| aliases.len());
    stage.charge(count.saturating_add(alias_count))?;
    stage.reserve(
        size_of::<s::BodyUnit>().saturating_add(
            count
                .saturating_add(alias_count)
                .saturating_mul(size_of::<u32>()),
        ),
    )?;
    // Missing raw-slot sentinels are construction scratch, never region IDs.
    // Owned normalization suffixes need not appear in semantic BodyRegions.
    let mut order = vec![u32::MAX; count];
    for (id, region) in facts.body_regions.regions_after(source, 0) {
        stage.charge(1)?;
        if region.body != source {
            return Err("source numbering cannot flatten nested allocations");
        }
        // Ordinary normalization-owned reservations also use QuantifiedProof
        // allocation labels. Their actual complete owned intervals were joined
        // by obligation_rows, which independently rejects quantified proofs.
        join(stage, owner, id, &mut order)?;
    }
    for proof in &input.proofs {
        for range in proof.proof.owned_regions() {
            for id in range {
                join(stage, owner, RegionId(id), &mut order)?;
            }
        }
    }
    if order.contains(&u32::MAX) {
        return Err("source numbering omits an allocation in its complete actual domain");
    }
    let aliases = aliases
        .map(|aliases| {
            if aliases.len() != input.universals as usize {
                return Err("source numbering has an incomplete actual body alias ledger");
            }
            let mut slots = Vec::with_capacity(aliases.len());
            for &alias in aliases.ordered() {
                stage.charge(1)?;
                if facts.body_regions.get(source, alias).is_none_or(|region| {
                    region.body != source
                        || !matches!(region.origin, BodyRegionOrigin::UniversalAlias)
                }) {
                    return Err("source numbering alias is not an actual universal allocation");
                }
                slots.push(stage.owned_slot(owner, alias)?);
            }
            Ok(slots)
        })
        .transpose()?;
    Ok(s::BodyUnit {
        body: owner,
        universals: input.universals,
        allocation_order: order,
        aliases,
    })
}

fn join(
    stage: &mut Stage,
    owner: BodyRef,
    id: RegionId,
    order: &mut [u32],
) -> Result<(), &'static str> {
    stage.charge(1)?;
    let ordinal =
        id.0.checked_sub(1)
            .ok_or("source numbering has a static allocation identity")? as usize;
    let target = order
        .get_mut(ordinal)
        .ok_or("source allocation is outside its complete actual domain")?;
    let slot = stage.owned_slot(owner, id)?;
    if *target != u32::MAX && *target != slot {
        return Err("source allocation observations disagree on their actual owned slot");
    }
    *target = slot;
    Ok(())
}
