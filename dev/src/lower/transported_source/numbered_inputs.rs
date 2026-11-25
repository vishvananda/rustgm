//! Preserve actual allocation order and actual alias membership independently.
use super::{Body, Context, Contract, Declarations, DefId, Owner, RegionId, Stage, s};
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
    let allow_gaps = stage.declaration_policies();
    if facts.body_regions.unit_of(source) != source
        && !(allow_gaps && matches!(body.owner, Owner::Closure { .. } | Owner::AnonymousConstant))
    {
        return Err("nested source numbering requires complete parent/unit transport");
    }
    let count = frontier(stage, facts, source, input)?
        .checked_sub(1)
        .ok_or("source numbering frontier precedes its domain")? as usize;
    if count < variables || (!allow_gaps && count != variables) {
        return Err("source numbering omits actual allocations from the original contract");
    }
    let environment = match body.owner {
        Owner::Function { .. } => {
            facts
                .function_header(definition)?
                .ok_or("source numbering lost its actual callable header")?
                .environment
        }
        Owner::Initializer { .. } => facts.generics.body_environment(definition),
        Owner::Closure { .. } | Owner::AnonymousConstant if allow_gaps => None,
        _ => return Err("source numbering lost its actual declaration body"),
    };
    let aliases = environment.and_then(|row| row.body_aliases());
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
    let order = allocation_order(
        stage,
        facts,
        (source, owner),
        input,
        (count, variables),
        allow_gaps,
    )?;
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

fn allocation_order(
    stage: &mut Stage,
    facts: &Declarations,
    (source, owner): (crate::resolve::unit::NodeKey, BodyRef),
    input: &Contract,
    (count, variables): (usize, usize),
    allow_gaps: bool,
) -> Result<Vec<u32>, &'static str> {
    let mut order = vec![u32::MAX; count];
    for obligation in &input.obligations {
        if let crate::body::regions::Kind::Variable { id, .. } = obligation.kind {
            join(stage, owner, id, &mut order)?;
        }
    }
    for proof in &input.proofs {
        for range in proof.proof.owned_regions() {
            for id in range {
                join(stage, owner, RegionId(id), &mut order)?;
            }
        }
    }
    for (id, region) in facts.body_regions.regions_after(source, 0) {
        stage.charge(1)?;
        if region.body != source {
            if allow_gaps {
                continue;
            }
            return Err("source numbering cannot flatten nested allocations");
        }
        let ordinal =
            id.0.checked_sub(1)
                .ok_or("source numbering has a static allocation identity")? as usize;
        if allow_gaps
            && order.get(ordinal) == Some(&u32::MAX)
            && matches!(region.origin, BodyRegionOrigin::QuantifiedProof)
            && is_proof_gap(input, id)
        {
            continue;
        }
        join(stage, owner, id, &mut order)?;
    }
    if allow_gaps && count > variables {
        for (ordinal, &slot) in order.iter().enumerate() {
            if slot == u32::MAX {
                let id = RegionId(
                    u32::try_from(ordinal + 1)
                        .map_err(|_| "source numbering allocation identity overflow")?,
                );
                let foreign_body = facts
                    .body_regions
                    .get(source, id)
                    .is_some_and(|region| region.body != source);
                if !foreign_body && !is_proof_gap(input, id) {
                    return Err(
                        "source numbering omits an allocation in its complete actual domain",
                    );
                }
            }
        }
        order.retain(|&slot| slot != u32::MAX);
        if order.len() != variables {
            return Err("source numbering omits actual allocations from the original contract");
        }
    } else if order.contains(&u32::MAX) {
        return Err("source numbering omits an allocation in its complete actual domain");
    }
    Ok(order)
}

fn is_proof_gap(input: &Contract, id: RegionId) -> bool {
    input
        .proofs
        .iter()
        .any(|proof| proof.proof.regions.contains(&id.0) && !proof.proof.owns_region(id))
}

fn frontier(
    stage: &mut Stage,
    facts: &Declarations,
    source: crate::resolve::unit::NodeKey,
    input: &Contract,
) -> Result<u32, &'static str> {
    let mut next = facts.body_regions.frontier(source)?;
    for obligation in &input.obligations {
        if let crate::body::regions::Kind::Variable { id, .. } = obligation.kind {
            stage.charge(1)?;
            next = next.max(
                id.0.checked_add(1)
                    .ok_or("source numbering frontier exceeds its identity space")?,
            );
        }
    }
    for proof in &input.proofs {
        for range in proof.proof.owned_regions() {
            stage.charge(1)?;
            next = next.max(range.end);
        }
    }
    Ok(next)
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
