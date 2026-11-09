//! Only completed acyclic declaration absence proves existential disjointness.
use super::{Context, Decision, Declarations, Frame, Goal, Search, continuation};
use crate::sema::traits::{
    assumptions::Assumptions,
    implementations::{Availability, Perspective, Purpose},
    selection::Caller,
};

impl Search<'_> {
    /// Open nested goals remain possible in a compatible downstream crate.
    /// Reuse Deferred so neither this frame nor an ancestor caches false absence.
    pub(super) fn coherence_absence(
        &self,
        facts: &mut Declarations,
        frame: &mut Frame,
        decision: Decision,
    ) -> Result<Decision, String> {
        if decision == Decision::Absent
            && let Some(perspective) = self.query.purpose.perspective()
            && frame.goal.trait_arguments().is_some()
            && !crate::sema::traits::implementations::overlap::absence_closed(
                facts,
                &mut frame.inputs.context,
                &frame.goal,
                perspective,
            )?
        {
            return Ok(Decision::Deferred);
        }
        Ok(decision)
    }
}

pub(in crate::sema) fn may_overlap(
    facts: &mut Declarations,
    context: &mut Context,
    goal: &Goal,
    perspective: Perspective,
) -> Result<bool, String> {
    may_overlap_with(
        facts,
        context,
        goal,
        crate::sema::coherence_policy::Mode::Stable,
        perspective,
        None,
        |_, _, _, _, _| Ok(false),
    )
}
/// A negative coherence query with `premises`, the other header's own Sized
/// and `MetaSized` sets shared by every condition of one side of a pair,
/// decides a rigid parameter's sizing only from them. `pay` receives the
/// proof's Sized debts, then its `MetaSized` debts.
pub(in crate::sema) fn may_overlap_with(
    facts: &mut Declarations,
    context: &mut Context,
    goal: &Goal,
    mode: crate::sema::coherence_policy::Mode,
    perspective: Perspective,
    premises: Option<&crate::types::parameters::shape::Placeholders>,
    pay: impl FnOnce(
        &mut Declarations,
        &mut Context,
        &crate::types::lifetime_proof::Obligations,
        &[crate::types::TypeId],
        &[crate::types::TypeId],
    ) -> Result<bool, String>,
) -> Result<bool, String> {
    if mode.negative()
        && Search::source_fn_pointer_goal(facts, goal)?
        && Search::rigid_non_fn_pointer(facts, context, goal)?
    {
        // rustc 1.98's builtin `impl !FnPtr for T` for each rigid T that is not
        // a function pointer: a negated where clause with no nested goals.
        return Ok(false);
    }
    let empty = Assumptions::default();
    let caller = Caller {
        assumptions: &empty,
        environment: None,
        first_region: 1,
        retain_sized: true,
    };
    let first = Frame::new(facts, context, goal, caller.first_region)?;
    let mut search = Search::new(caller, Availability::Declarations);
    search.query.purpose = if mode.negative() {
        Purpose::NegativeCoherence(perspective)
    } else {
        Purpose::Coherence(perspective)
    };
    if mode.negative()
        && let Some(premises) = premises
    {
        context.account_work(1).map_err(|e| e.to_string())?;
        let sizing = super::sizing_fallback::NegativeSizing::assume(premises);
        search
            .query
            .builtins
            .assume_placeholders(&facts.types, sizing.placeholders().clone())?;
        search.query.negative_sizing = Some(Box::new(sizing));
    }
    let progress = search.begin(facts, first)?;
    context
        .account_work(search.query.steps)
        .map_err(|e| e.to_string())?;
    if search.query.cycles != 0 {
        return Ok(true);
    }
    match progress {
        continuation::Progress::Ready(Decision::Absent) if mode.implicit() => Ok(false),
        continuation::Progress::Ready(Decision::Refuted(node)) if mode.negative() => {
            let expected = search.query.nodes[node].goal;
            let refutation = search.refutation(facts, node, expected)?;
            refutation
                .disjoint_with(|proof, sized, meta_sized| {
                    pay(facts, context, proof, sized, meta_sized)
                })
                .map(|disjoint| !disjoint)
        }
        _ => Ok(true),
    }
}

#[cfg(test)]
#[test]
fn unrelated_cycle_cutoff_cannot_contaminate_later_closed_disjointness()
-> Result<(), Box<dyn std::error::Error>> {
    use crate::{
        support::comparisons::Comparison,
        types::{GenericArgs, ItemId, Primitive, TypeId, infer::arguments::Argument},
    };
    let (report, mut facts) = super::tests::declarations(
        "trait Cycle{}impl<T:Cycle> Cycle for T{}struct S;impl PartialEq for &S{fn eq(&self,_:&Self)->bool{true}}fn main(){}",
    )?;
    let cycle = super::tests::definition(&report, "Cycle")?;
    let nominal = super::tests::definition(&report, "S")?;
    let goal = super::tests::predicate(
        &mut facts,
        cycle,
        TypeId::primitive(Primitive::U8),
        GenericArgs::EMPTY,
    )?;
    let (mut context, goal) = super::tests::instantiate(&mut facts, goal)?;
    assert!(
        may_overlap(&mut facts, &mut context, &goal, Perspective::Local)?,
        "a recursive cutoff is not closed absence"
    );
    let comparison = report
        .resolution
        .builtin_traits
        .comparison(Comparison::PartialEq)
        .ok_or("comparison")?
        .definition;
    let ty = facts.definitions[nominal.0 as usize].ok_or("nominal")?;
    let subject = context.known(&facts.types, ty)?;
    let goal =
        Goal::trait_application(subject, ItemId(comparison.0), vec![Argument::Type(subject)]);
    assert!(
        !may_overlap(&mut facts, &mut context, &goal, Perspective::Local)?,
        "an unrelated prior cutoff cannot create overlap"
    );
    Ok(())
}
