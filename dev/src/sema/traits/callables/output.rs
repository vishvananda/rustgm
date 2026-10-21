//! A callable output equation can expose existing holes while its trait proof stays pending.
use super::{Context, Declarations, TermId, infer, match_signature};
use crate::types::{infer::probe::Probe, lifetime_proof::Obligations, substitute::Bound};

pub(in crate::sema) fn symbolic(
    facts: &mut Declarations,
    source: &mut Context,
    root: TermId,
    first: u32,
) -> Result<Option<(TermId, Obligations)>, String> {
    let (mut probe, roots) = Probe::new(&facts.types, source, &[root]).map_err(error)?;
    let result = expose(facts, source, &mut probe, roots[0], first);
    source.account_work(probe.context().steps).map_err(error)?;
    result
}

fn expose(
    facts: &mut Declarations,
    source: &mut Context,
    probe: &mut Probe,
    root: TermId,
    first: u32,
) -> Result<Option<(TermId, Obligations)>, String> {
    let mut opening = Bound::open_regions(first)?;
    let root = probe
        .context()
        .map_regions(&mut facts.types, root, &mut opening)
        .map_err(error)?;
    let application = probe
        .context()
        .projection_application(&facts.types, root)
        .map_err(error)?;
    let mut proof = Obligations::new(first);
    proof.regions.end = opening
        .next_region()
        .ok_or("missing callable output region interval")?;
    proof.equalities.extend(
        opening
            .opened_regions()
            .map(|(original, free)| (free, original)),
    );
    proof
        .equalities
        .sort_unstable_by_key(|&(free, original)| (free.0, original.0));
    let matched = match match_signature(
        facts,
        probe.context(),
        &application.predicate,
        proof.regions.end,
    ) {
        Ok(matched) => matched,
        Err(
            infer::Error::Unresolved
            | infer::Error::Mismatch
            | infer::Error::Occurs
            | infer::Error::Normalization,
        ) => return Ok(None),
        Err(e) => return Err(error(e)),
    };
    proof.regions.end = matched.regions.end;
    if !probe.preserves_inputs(source).map_err(error)? {
        return Ok(None);
    }
    proof
        .equalities
        .append(&mut probe.context().region_equalities);
    let mut restoration = proof.restoration()?;
    let output = probe
        .context()
        .map_regions(&mut facts.types, matched.output, &mut restoration)
        .map_err(error)?;
    Ok(probe
        .export(&facts.types, source, output)
        .map_err(error)?
        .map(|output| (output, proof)))
}

fn error(error: infer::Error) -> String {
    error.to_string()
}
