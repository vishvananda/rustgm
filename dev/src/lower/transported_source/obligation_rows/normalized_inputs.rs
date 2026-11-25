//! Owned intervals and labelled raw debts survive fresh allocation on import.
use super::{Context, Declarations, RegionId, Sites, Stage, g, relation, s};

pub(super) fn emit(
    stage: &mut Stage,
    facts: &Declarations,
    proof: &crate::body::regions::Proof,
    context: Context,
    sites: &mut Sites,
) -> Result<s::NormalizationInput, &'static str> {
    if !super::normalization_supported(stage, &proof.proof) {
        return Err("source quantified normalization rows are not implemented");
    }
    let body = context
        .body()
        .ok_or("source normalization has no actual original owner")?;
    let mut raw = s::NormalizationInput {
        site: sites.input(stage, proof.source)?,
        sources: Vec::new(),
        owned: Vec::new(),
        equalities: Vec::new(),
        outlives: Vec::new(),
        types: Vec::new(),
        scoped: None,
    };
    for &source in &proof.sources {
        stage.reserve(4 * size_of::<s::InputSite>())?;
        raw.sources.push(sites.input(stage, source)?);
    }
    for range in proof.proof.owned_regions() {
        if range.is_empty() {
            continue;
        }
        let start = stage.owned_slot(body, RegionId(range.start))?;
        let end = stage
            .owned_slot(body, RegionId(range.end - 1))?
            .checked_add(1)
            .ok_or("source normalization allocation ordinal overflow")?;
        if end.checked_sub(start) != range.end.checked_sub(range.start) {
            return Err("source normalization lost its exact disjoint allocation interval");
        }
        if let Some(previous) = raw.owned.last_mut()
            && previous.end == start
        {
            previous.end = end;
            continue;
        }
        stage.reserve(4 * size_of::<g::Range>())?;
        raw.owned.push(g::Range { start, end });
    }
    relations(
        stage,
        facts,
        &proof.proof,
        context,
        (&mut raw.equalities, &mut raw.outlives, &mut raw.types),
    )?;
    if !proof.proof.quantifies_nothing() && !proof.proof.has_only_root_intervals() {
        raw.scoped = Some(Box::new(emit_scoped(
            stage,
            facts,
            body,
            &proof.proof,
            context,
        )?));
    }
    Ok(raw)
}

fn emit_scoped(
    stage: &mut Stage,
    facts: &Declarations,
    body: crate::support::crates::owner_catalog::BodyRef,
    proof: &crate::types::lifetime_proof::Obligations,
    context: Context,
) -> Result<s::ScopedNormalization, &'static str> {
    stage.reserve(size_of::<s::ScopedNormalization>())?;
    let mut contexts = vec![context];
    let mut scopes = Vec::new();
    for scope in proof.scopes() {
        stage.charge(1)?;
        stage.reserve(4 * size_of::<s::NormalizationScope>())?;
        let parent = *contexts
            .get(scope.parent.0 as usize)
            .ok_or("source scoped normalization parent is out of order")?;
        let child = stage.scoped_context(parent, scope.count)?;
        scopes.push(s::NormalizationScope {
            parent: scope.parent.0,
            frame: stage.binder(child)?,
            callable: scope.callable,
        });
        contexts.push(child);
    }
    let mut blocks = Vec::new();
    for (scope_id, universal_aliases, origin, block) in proof.quantified_blocks() {
        stage.charge(1)?;
        stage.reserve(4 * size_of::<s::NormalizationBlock>())?;
        let block_context = *contexts
            .get(scope_id.0 as usize)
            .ok_or("source scoped normalization block names an unknown scope")?;
        let regions = if block.regions.is_empty() {
            g::Range { start: 0, end: 0 }
        } else {
            let start = stage.owned_slot(body, RegionId(block.regions.start))?;
            let end = stage
                .owned_slot(body, RegionId(block.regions.end - 1))?
                .checked_add(1)
                .ok_or("source scoped normalization allocation ordinal overflow")?;
            if end.checked_sub(start) != block.regions.end.checked_sub(block.regions.start) {
                return Err("source scoped normalization lost its contiguous block interval");
            }
            g::Range { start, end }
        };
        let mut equalities = Vec::new();
        let mut outlives = Vec::new();
        let mut types = Vec::new();
        relations(
            stage,
            facts,
            block,
            block_context,
            (&mut equalities, &mut outlives, &mut types),
        )?;
        blocks.push(s::NormalizationBlock {
            scope: scope_id.0,
            universal_aliases,
            origin,
            regions,
            equalities,
            outlives,
            types,
        });
    }
    Ok(s::ScopedNormalization { scopes, blocks })
}

type Relations<'a> = (
    &'a mut Vec<s::RegionBound>,
    &'a mut Vec<s::RegionBound>,
    &'a mut Vec<s::TypeOutlives>,
);
fn relations(
    stage: &mut Stage,
    facts: &Declarations,
    proof: &crate::types::lifetime_proof::Obligations,
    context: Context,
    (equalities, outlives, types): Relations<'_>,
) -> Result<(), &'static str> {
    for (output, rows) in [(equalities, &proof.equalities), (outlives, &proof.outlives)] {
        for &pair in rows {
            stage.reserve(4 * size_of::<s::RegionBound>())?;
            output.push(relation(stage, pair, context)?);
        }
    }
    for &(ty, shorter) in &proof.types {
        stage.reserve(4 * size_of::<s::TypeOutlives>())?;
        types.push(s::TypeOutlives {
            ty: stage.ty(facts, ty, context)?,
            shorter: stage.region(shorter, context)?,
        });
    }
    Ok(())
}
