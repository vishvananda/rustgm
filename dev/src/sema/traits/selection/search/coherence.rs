//! Only completed acyclic declaration absence proves existential disjointness.
use super::{Context, Decision, Declarations, Frame, Goal, Search, continuation};
use crate::sema::traits::{
    assumptions::Assumptions, implementations::Availability, selection::Caller,
};

pub(in crate::sema) fn may_overlap(
    facts: &mut Declarations,
    context: &mut Context,
    goal: &Goal,
) -> Result<bool, String> {
    let empty = Assumptions::default();
    let caller = Caller {
        assumptions: &empty,
        environment: None,
        first_region: 1,
        retain_sized: true,
    };
    let first = Frame::new(facts, context, goal, caller.first_region)?;
    let mut search = Search::new(caller, Availability::Declarations);
    let progress = search.begin(facts, first)?;
    context
        .account_work(search.query.steps)
        .map_err(|e| e.to_string())?;
    Ok(
        !matches!(progress, continuation::Progress::Ready(Decision::Absent))
            || search.query.cycles != 0,
    )
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
        may_overlap(&mut facts, &mut context, &goal)?,
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
        !may_overlap(&mut facts, &mut context, &goal)?,
        "an unrelated prior cutoff cannot create overlap"
    );
    Ok(())
}
