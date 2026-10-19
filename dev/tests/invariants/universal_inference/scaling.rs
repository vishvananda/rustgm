//! Quantified retention scales with selected conditions and new allocations, not proof depth squared.
use super::{
    Availability, BYTE, Context, Declarations, RegionId, Selection, TestResult, begin, goal, prove,
    suspension, tests,
};
use crate::{
    sema::traits::{assumptions::Assumptions, selection::OwnedCaller},
    types::{Kind, substitute::Bound},
};
use std::{fmt::Write, sync::Arc};

#[test]
fn deep_partial_universal_conditions_keep_linear_work_and_one_export() -> TestResult {
    for depth in [8, 32, 128, 512, 2048] {
        let mut source = String::from("trait A0<'a,T>{}impl<'a,'b,U>A0<'a,(&'b u8,U)>for u8{}");
        for index in 1..=depth {
            write!(
                source,
                "trait A{index}<'a,T>{{}}impl<'a,T>A{index}<'a,T>for u8 where u8:A{}<'a,T>{{}}",
                index - 1
            )?;
        }
        let (report, mut facts) = tests::declarations(&source)?;
        let (mut context, goal, unknown) = goal(
            &mut facts,
            tests::definition(&report, &format!("A{depth}"))?,
        )?;
        let suspension::Progress::Inferred { query } = begin(&mut facts, &mut context, &goal)?
        else {
            return Err("deep universal chain did not retain its partial input".into());
        };
        let before = context.steps;
        let end = query.allocation_end();
        let inner = context.child(&facts.types, unknown, 1)?;
        let value = context.known(&facts.types, BYTE)?;
        context.equate(&facts.types, inner, value)?;
        let suspension::Progress::Ready(Selection::Proven(mut evidence)) =
            query.resume(&mut facts, &mut context, &goal, end + 17)?
        else {
            return Err("deep universal chain did not complete".into());
        };
        let after = context.steps - before;
        evidence
            .proof
            .equalities
            .append(&mut context.region_equalities);
        assert_eq!(evidence.proof.scope_count(), 1);
        // Each parent owns one impl lifetime; the leaf owns two. There is one
        // universal alias and precisely one older exported witness.
        let owned: usize = evidence.proof.owned_regions().map(|r| r.len()).sum();
        assert_eq!(owned, depth + 4);
        for id in end..end + 17 {
            assert!(!evidence.proof.owns_region(RegionId(id)));
        }
        eprintln!("universal partial depth={depth} initial={before} resumed={after} owned={owned}");
        assert!(before <= 300 * (depth + 1), "initial {before} at {depth}");
        assert!(after <= 180 * (depth + 1), "resumed {after} at {depth}");
        assert!(prove(&facts, &evidence.proof, false, false)?);
    }
    Ok(())
}

fn open_empty(
    facts: &mut Declarations,
    context: &mut Context,
    goal: &super::Goal,
) -> TestResult<super::Goal> {
    Ok(
        context.map_predicate_regions(
            &mut facts.types,
            goal,
            &mut Bound::new(vec![RegionId(1)]),
        )?,
    )
}

#[test]
fn empty_input_quantifiers_preserve_their_scope_and_older_captures() -> TestResult {
    let (report, mut facts) =
        tests::declarations("trait A<'a,T>{}impl<'a,'b>A<'a,&'b u8>for u8{}")?;
    let (mut context, input, unknown) = goal(&mut facts, tests::definition(&report, "A")?)?;
    let input = open_empty(&mut facts, &mut context, &input)?;
    let suspension::Progress::Ready(Selection::Proven(evidence)) = suspension::begin_refining(
        &mut facts,
        &mut context,
        &input,
        OwnedCaller {
            assumptions: Arc::new(Assumptions::default()),
            environment: None,
            first_region: 2,
            retain_sized: false,
        },
        Availability::Declarations,
        Some(0),
    )?
    else {
        return Err("empty quantified input did not complete".into());
    };
    assert_eq!(evidence.proof.scope_count(), 1);
    assert_eq!(evidence.proof.regions.end, 5);
    let ty = context.resolved(&mut facts.types, unknown)?;
    assert!(matches!(
        facts.types.kind(ty),
        Some(Kind::Reference {
            region: RegionId(4),
            ..
        })
    ));
    assert!(prove(&facts, &evidence.proof, false, false)?);
    Ok(())
}
