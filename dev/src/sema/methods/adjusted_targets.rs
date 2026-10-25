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
        RegionId,
        infer::{Constructor, Context, TermId},
        subject_head::Head,
    },
};

pub(super) fn collect(
    report: &Report,
    facts: &mut Declarations,
    context: &mut Context,
    mut term: TermId,
    name: NameId,
    site: Use,
    found: &mut Found,
) -> Result<(), &'static str> {
    if !facts.methods.structural_name(name) {
        return Ok(());
    }
    let mut depth = 0;
    loop {
        variants(report, facts, context, term, name, site, found)?;
        let head = context
            .head(&facts.types, term)
            .map_err(selection::inference)?;
        if matches!(head, Some(Constructor::Reference { .. })) {
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
                variants(report, facts, context, slice, name, site, found)?;
            }
            break;
        }
    }
    found.members.sort_unstable_by_key(|member| member.0);
    found.members.dedup();
    Ok(())
}

/// Build borrow shapes only when an actual reference-owner bucket can use them.
fn variants(
    report: &Report,
    facts: &mut Declarations,
    context: &mut Context,
    term: TermId,
    name: NameId,
    site: Use,
    found: &mut Found,
) -> Result<(), &'static str> {
    extend(report, facts, context, term, name, site, found)?;
    for mutable in [false, true] {
        if !facts
            .methods
            .has_inherent(Bucket::Structural(Head::Reference(mutable), name))
        {
            continue;
        }
        let reference = context
            .construct(
                Constructor::Reference {
                    mutable,
                    region: RegionId::STATIC,
                },
                &[term],
            )
            .map_err(selection::inference)?;
        extend(report, facts, context, reference, name, site, found)?;
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
