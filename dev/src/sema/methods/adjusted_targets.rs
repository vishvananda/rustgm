//! Relevant structural Self shapes are collected once, before isolated receiver probes.
use super::{
    buckets::Bucket,
    names::Use,
    selection::{self, Found},
};
use crate::{
    resolve::{expansion::Report, names::NameId},
    sema::model::Declarations,
    types::{
        infer::{Constructor, Context, TermId},
        subject_head::Head,
    },
};

pub(super) fn collect(
    report: &Report,
    facts: &mut Declarations,
    context: &mut Context,
    (mut term, base): (TermId, TermId),
    name: NameId,
    site: Use,
    found: &mut Found,
) -> Result<(), &'static str> {
    if !facts.methods.structural_name(name) {
        return Ok(());
    }
    let array_base = matches!(
        context
            .head(&facts.types, base)
            .map_err(selection::inference)?,
        Some(Constructor::Array(_) | Constructor::ArrayTerm(_) | Constructor::ArrayInfer(_))
    );
    let retain_references = array_base
        && [false, true].into_iter().any(|mutable| {
            facts
                .methods
                .has_inherent(Bucket::Structural(Head::Reference(mutable), name))
        });
    let mut depth = 0;
    let mut reference_steps = Vec::new();
    loop {
        extend(report, facts, context, term, name, site, found)?;
        let head = context
            .head(&facts.types, term)
            .map_err(selection::inference)?;
        if let Some(reference @ Constructor::Reference { .. }) = head {
            if retain_references {
                reference_steps.push(reference);
            }
            term = context
                .child(&facts.types, term, 0)
                .map_err(selection::inference)?;
            depth += 1;
            if depth > 1_000_000 {
                return Err("structural receiver depth budget exceeded");
            }
        } else {
            if matches!(
                head,
                Some(
                    Constructor::Array(_) | Constructor::ArrayTerm(_) | Constructor::ArrayInfer(_)
                )
            ) {
                let element = context
                    .child(&facts.types, term, 0)
                    .map_err(selection::inference)?;
                let slice = context
                    .construct(Constructor::Slice, &[element])
                    .map_err(selection::inference)?;
                unsized_candidates(
                    report,
                    facts,
                    context,
                    (slice, &reference_steps),
                    name,
                    site,
                    found,
                )?;
            }
            break;
        }
    }
    found.members.sort_unstable_by_key(|member| member.0);
    found.members.dedup();
    Ok(())
}

/// Autoref adjusts a receiver; it does not introduce an inherent Self owner.
/// Rebuild actual reference prefixes once, retaining their order and regions.
/// These are full-Self exclusion probes, not permission for a nested unsize.
fn unsized_candidates(
    report: &Report,
    facts: &mut Declarations,
    context: &mut Context,
    (term, reference_steps): (TermId, &[Constructor]),
    name: NameId,
    site: Use,
    found: &mut Found,
) -> Result<(), &'static str> {
    extend(report, facts, context, term, name, site, found)?;
    let present = [false, true].map(|mutable| {
        facts
            .methods
            .has_inherent(Bucket::Structural(Head::Reference(mutable), name))
    });
    if !present.into_iter().any(|present| present) {
        return Ok(());
    }
    let mut prefix = term;
    for &reference in reference_steps.iter().rev() {
        let Constructor::Reference { mutable, .. } = reference else {
            return Err("invalid source receiver reference prefix");
        };
        prefix = context
            .construct(reference, &[prefix])
            .map_err(selection::inference)?;
        if present[usize::from(mutable)] {
            extend(report, facts, context, prefix, name, site, found)?;
        }
    }
    Ok(())
}

fn extend(
    report: &Report,
    facts: &mut Declarations,
    context: &mut Context,
    term: TermId,
    name: NameId,
    site: Use,
    found: &mut Found,
) -> Result<(), &'static str> {
    let Some(head) = context
        .head(&facts.types, term)
        .map_err(selection::inference)?
    else {
        return Ok(());
    };
    let Some(bucket @ Bucket::Structural(..)) = Bucket::lookup(Head::inferred(head), name) else {
        return Ok(());
    };
    if !facts.methods.has_inherent(bucket) {
        return Ok(());
    }
    let candidates = selection::candidates(report, facts, context, term, bucket, site)?;
    found.filtered |= candidates.filtered;
    found.members.extend(candidates.members);
    Ok(())
}
