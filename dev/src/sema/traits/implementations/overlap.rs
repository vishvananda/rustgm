//! Structural impl disjointness uses simultaneous existential type/const equations.
mod deep_reject;
mod disjointness;
pub(in crate::sema) use disjointness::absence_closed;
#[cfg(test)]
#[path = "../../../../tests/invariants/overlap_headers.rs"]
mod head_tests;
mod inputs;
mod iteration;
#[cfg(test)]
#[path = "../../../../tests/invariants/rejected_overlap_pairs.rs"]
mod rejection_tests;
use crate::{
    resolve::model::DefId,
    sema::{model::Declarations, traits::implementations::Perspective},
    types::{
        infer::{self, Context},
        parameters::predicates::Predicate,
    },
};
pub(in crate::sema) use iteration::possible as iteration;

/// Whether two impl headers may overlap, and the steps spent deciding it. Like
/// rustc's `overlapping_impls`, a pair whose trait references differ at a rigid
/// constructor is answered by that comparison, before either header is instantiated.
pub(in crate::sema) fn possible(
    facts: &mut Declarations,
    left: (Predicate, DefId),
    right: (Predicate, DefId),
) -> Result<(bool, usize), String> {
    let (unifiable, compared) = deep_reject::may_unify(&facts.types, left.0, right.0)?;
    #[cfg(test)]
    audit(facts, left, right, unifiable)?;
    if !unifiable {
        return Ok((false, compared));
    }
    let (overlap, steps) = instantiated(facts, left, right)?;
    Ok((overlap == Overlap::Possible, steps.saturating_add(compared)))
}

/// The complete answer for two instantiated impl headers.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Overlap {
    /// Their simultaneous shape equation fails.
    Disjoint,
    /// Their shapes unify, but sizing or a condition excludes every intersection.
    Excluded,
    Possible,
}

fn instantiated(
    facts: &mut Declarations,
    left: (Predicate, DefId),
    right: (Predicate, DefId),
) -> Result<(Overlap, usize), String> {
    let mut context = Context::default();
    let mut inputs = inputs::Inputs::default();
    let (a, mut left_inputs) = inputs.head(facts, &mut context, left)?;
    let (b, mut right_inputs) = inputs.head(facts, &mut context, right)?;
    let mode = crate::sema::coherence_policy::predicate(facts, left.0);
    let perspective = Perspective::of(facts, [left.1, right.1])?;
    let overlap = match context.equate_predicate_shapes(&facts.types, &a, &b) {
        Ok(()) => {
            inputs.conditions(facts, &mut context, left.1, &mut left_inputs)?;
            inputs.conditions(facts, &mut context, right.1, &mut right_inputs)?;
            if disjointness::has_unsized(facts, &mut context, &inputs)?
                || inputs.excluded(
                    facts,
                    &mut context,
                    &[&left_inputs, &right_inputs],
                    mode,
                    perspective,
                )?
            {
                Overlap::Excluded
            } else {
                Overlap::Possible
            }
        }
        Err(infer::Error::Mismatch | infer::Error::Occurs) => Overlap::Disjoint,
        Err(error) => return Err(error.to_string()),
    };
    Ok((overlap, context.steps))
}

#[cfg(test)]
thread_local! {
    /// Test-only: when set, every pair the reject answers is instantiated too.
    /// Counts the pairs compared, the pairs rejected and the rejected pairs
    /// whose complete shape equation still succeeds, which must stay zero.
    static AUDIT: std::cell::Cell<Option<[usize; 3]>> = const { std::cell::Cell::new(None) };
}
#[cfg(test)]
fn audit(
    facts: &mut Declarations,
    left: (Predicate, DefId),
    right: (Predicate, DefId),
    unifiable: bool,
) -> Result<(), String> {
    let Some([pairs, rejected, unifying]) = AUDIT.with(std::cell::Cell::get) else {
        return Ok(());
    };
    let unifies = !unifiable && instantiated(facts, left, right)?.0 != Overlap::Disjoint;
    AUDIT.with(|cell| {
        cell.set(Some([
            pairs + 1,
            rejected + usize::from(!unifiable),
            unifying + usize::from(unifies),
        ]));
    });
    Ok(())
}

/// The same complete owner conditions also govern inherent method ambiguity.
pub(in crate::sema) fn inherent(
    facts: &mut Declarations,
    left: DefId,
    right: DefId,
) -> Result<(bool, usize), String> {
    let mut context = Context::default();
    let mut inputs = inputs::Inputs::default();
    let (a, mut first) = inputs.target(facts, &mut context, left)?;
    let (b, mut second) = inputs.target(facts, &mut context, right)?;
    let mode = crate::sema::coherence_policy::inherent(facts, left);
    let perspective = Perspective::of(facts, [left, right])?;
    let possible = match context.equate_shape(&facts.types, a, b) {
        Ok(()) => {
            inputs.conditions(facts, &mut context, left, &mut first)?;
            inputs.conditions(facts, &mut context, right, &mut second)?;
            !disjointness::has_unsized(facts, &mut context, &inputs)?
                && !inputs.excluded(facts, &mut context, &[&first, &second], mode, perspective)?
        }
        Err(infer::Error::Mismatch | infer::Error::Occurs) => false,
        Err(error) => return Err(error.to_string()),
    };
    Ok((possible, context.steps))
}

/// Source reference impls compete with the canonical blanket only if its exact
/// existential referent condition remains possible. No evidence from this
/// declaration-only query escapes into a caller or executable factory.
pub(in crate::sema) fn reference_comparison(
    facts: &mut Declarations,
    header: (Predicate, DefId),
) -> Result<(bool, usize), String> {
    use crate::sema::{
        primitive_methods::{self, Candidate},
        traits::selection,
    };
    let Some((application, _)) = header.0.bound.trait_parts() else {
        return Ok((false, 0));
    };
    let item = facts
        .types
        .trait_application(application)
        .ok_or("missing coherence trait application")?
        .item;
    if facts.associated.comparison(DefId(item.0)).is_none()
        || !matches!(
            facts.types.kind(header.0.ty),
            Some(crate::types::Kind::Reference { .. })
        )
    {
        return Ok((false, 0));
    }
    let mut context = Context::default();
    let mut inputs = inputs::Inputs::default();
    let (goal, bindings) = inputs.instantiate(facts, &mut context, header)?;
    let candidate = primitive_methods::candidate(facts, &mut context, &goal, false)
        .map_err(|e| e.to_string())?;
    let Candidate::Conditional { prerequisite, .. } = candidate else {
        return Ok((false, context.steps));
    };
    if disjointness::has_unsized(facts, &mut context, &inputs)? {
        return Ok((false, context.steps));
    }
    let mode = crate::sema::coherence_policy::predicate(facts, header.0);
    if mode.negative() {
        inputs::Inputs::freeze(facts, &mut context, &[&bindings])?;
    }
    // A source impl against the canonical blanket is this crate's pair.
    let possible = selection::may_overlap_with(
        facts,
        &mut context,
        &prerequisite,
        mode,
        Perspective::Local,
        // This single-header comparison still assumes no MetaSized premise.
        None,
        |facts, context, proof, sized, meta_sized| {
            inputs.pay_negative(facts, context, proof, sized, meta_sized, &[])
        },
    )?;
    Ok((possible, context.steps))
}

#[cfg(test)]
pub(in crate::sema) fn input_layout() -> usize {
    size_of::<inputs::Inputs>()
}
