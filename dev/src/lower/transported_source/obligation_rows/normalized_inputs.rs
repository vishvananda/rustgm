//! Owned intervals and labelled raw debts survive fresh allocation on import.
use super::{Context, Declarations, RegionId, Sites, Stage, g, relation, s};

pub(super) fn emit(
    stage: &mut Stage,
    facts: &Declarations,
    proof: &crate::body::regions::Proof,
    context: Context,
    sites: &mut Sites,
) -> Result<s::NormalizationInput, &'static str> {
    if proof.proof.has_quantified() {
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
    };
    for &source in &proof.sources {
        stage.reserve(4 * size_of::<s::InputSite>())?;
        raw.sources.push(sites.input(stage, source)?);
    }
    for range in proof.proof.owned_regions() {
        stage.reserve(4 * size_of::<g::Range>())?;
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
        raw.owned.push(g::Range { start, end });
    }
    for &pair in &proof.proof.equalities {
        stage.reserve(4 * size_of::<s::RegionBound>())?;
        raw.equalities.push(relation(stage, pair, context)?);
    }
    for &pair in &proof.proof.outlives {
        stage.reserve(4 * size_of::<s::RegionBound>())?;
        raw.outlives.push(relation(stage, pair, context)?);
    }
    for &(ty, shorter) in &proof.proof.types {
        stage.reserve(4 * size_of::<s::TypeOutlives>())?;
        raw.types.push(s::TypeOutlives {
            ty: stage.ty(facts, ty, context)?,
            shorter: stage.region(shorter, context)?,
        });
    }
    Ok(raw)
}
