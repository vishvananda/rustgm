//! Published partial references retain private lifetimes and caller debts until final proof.
use super::super::continuation::Progress;
use super::{Decision, Search, TestResult, begin, open_goal, tests};
use crate::{
    sema::traits::{assumptions::Assumptions, implementations::Availability},
    types::{
        Primitive, RegionId, TypeId, infer::Constructor, outlives::Queries, relations::Constraints,
    },
};

#[test]
fn partial_lifetimes_preserve_final_debts_and_foreign_allocation_gaps() -> TestResult {
    for (restricted, permission, accepted) in [
        (false, false, true),
        (true, false, false),
        (true, true, true),
    ] {
        let bound = if restricted { ":'static" } else { "" };
        let (report, mut facts) = tests::declarations(&format!(
            "trait A<T>{{}}impl<'a{bound},T>A<(&'a u8,T)>for u8{{}}"
        ))?;
        let (mut context, goal, unknown) = open_goal(&mut facts, tests::definition(&report, "A")?)?;
        let assumptions = Assumptions::default();
        let mut search = Search::new(
            tests::caller(&assumptions, None),
            Availability::Declarations,
        );
        assert_eq!(
            begin(&mut search, &mut facts, &mut context, &goal)?,
            Progress::Inferred
        );
        assert!(search.publish_inputs(&mut facts, &mut context)?);
        let reference = context.child(&facts.types, unknown, 0)?;
        let generic = context.child(&facts.types, unknown, 1)?;
        let caller = RegionId::bound(0, 0)?;
        let byte = context.known(&facts.types, TypeId::primitive(Primitive::U8))?;
        let target = context.construct(
            Constructor::Reference {
                mutable: false,
                region: caller,
            },
            &[byte],
        )?;
        context.equate(&facts.types, reference, target)?;
        context.equate(&facts.types, generic, byte)?;
        let end = search.query.allocations.end();
        search.resume_at(end + 7)?;
        search.refresh_inputs(&mut facts, &mut context)?;
        let Progress::Ready(Decision::Proven(root)) = search.poll(&mut facts)? else {
            return Err("partial lifetime goal did not finish selection".into());
        };
        context.equate_proved_predicate(&mut facts.types, &goal, search.query.nodes[root].goal)?;
        let mut evidence = search.evidence(&facts, root)?;
        evidence
            .proof
            .equalities
            .append(&mut context.region_equalities);
        for id in end..end + 7 {
            assert!(!evidence.proof.owns_region(RegionId(id)));
        }
        assert_eq!(
            evidence
                .proof
                .owned_regions()
                .map(|r| r.len())
                .sum::<usize>(),
            1
        );
        let mut constraints =
            Constraints::new(&facts.types, &facts.lifetimes.variances, 1, 100_000)?;
        if permission {
            constraints.assume(caller, RegionId::STATIC)?;
        }
        let mut queries = Queries::new(
            &facts.types,
            &facts.lifetimes.nominal_requirements,
            &facts.lifetimes.callables,
        );
        evidence.proof.require(&mut constraints, &mut queries)?;
        assert_eq!(
            constraints.prove().is_ok(),
            accepted,
            "restriction={restricted} permission={permission}"
        );
    }
    Ok(())
}
