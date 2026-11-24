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
    if !proof.proof.quantifies_nothing()
        && let Some(scope) = proof.proof.single_normalization_scope()
    {
        stage.reserve(size_of::<s::ScopedNormalization>())?;
        let child = stage.scoped_context(context, scope.regions)?;
        let mut scoped = s::ScopedNormalization {
            frame: stage.binder(child)?,
            callable: scope.callable,
            origin: scope.origin,
            equalities: Vec::new(),
            outlives: Vec::new(),
            types: Vec::new(),
        };
        relations(
            stage,
            facts,
            scope.block,
            child,
            (
                &mut scoped.equalities,
                &mut scoped.outlives,
                &mut scoped.types,
            ),
        )?;
        raw.scoped = Some(Box::new(scoped));
    }
    Ok(raw)
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
