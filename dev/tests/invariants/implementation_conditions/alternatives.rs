//! Discarded candidates and independent trials cannot publish or share universal permissions.
use super::{
    BYTE, Decision, Frame, Search, State, TestResult, quantified_tests, quantify, quantify_owner,
    root, tests,
};
use crate::sema::traits::{assumptions::Assumptions, implementations::Availability};
use crate::types::{
    ItemId, Kind, Primitive, RegionId, TypeId,
    arguments::Argument,
    infer::{Context, Domain, Instantiation, arguments::Argument as Inferred},
    parameters::{Parameter, ParameterKind},
};

#[test]
fn discarded_candidates_publish_no_universal_debts() -> TestResult {
    let (report, mut facts) = tests::declarations(
        "trait Pick<T>{}trait B<'a>{}trait C<'a>{}impl<'a:'static>B<'a>for u8{}impl<T:B<'static>+C<'static>>Pick<u8>for T{}impl Pick<u16>for u8{}",
    )?;
    let owner = facts
        .generics
        .implementations()
        .find(|header| {
            facts
                .generics
                .predicates
                .get(&header.owner)
                .is_some_and(|values| values.len() == 2)
        })
        .ok_or("missing constrained candidate")?
        .owner;
    for name in ["B", "C"] {
        quantify_owner(
            &mut facts,
            owner,
            tests::definition(&report, name)?,
            1,
            &[RegionId::bound(0, 0)?],
        )?;
    }
    let id = facts.types.intern_parameter(Parameter {
        owner: ItemId(1_000_000),
        index: 0,
        kind: ParameterKind::Type { sized: true },
    })?;
    let ty = facts.types.intern(Kind::Parameter(id))?;
    let arguments = facts.types.intern_arguments(&[Argument::Type(ty)])?;
    let root = tests::predicate(
        &mut facts,
        tests::definition(&report, "Pick")?,
        BYTE,
        arguments,
    )?;
    let mut context = Context::default();
    let hole = context.variable(Domain::Any)?;
    let mut substitution =
        Instantiation::new(&mut context, &facts.types, &[(id, Inferred::Type(hole))])?;
    let goal = context.instantiate_predicate(&mut facts.types, root, &mut substitution)?;
    let assumptions = Assumptions::default();
    let mut search = Search::new(tests::caller(&assumptions, None), Availability::Validated);
    let first = Frame::new(&facts, &mut context, &goal, 1)?;
    let decision = search.run(&mut facts, first)?;
    assert!(matches!(decision, Decision::Proven(_)));
    let super::super::Selection::Proven(evidence) =
        search.result(&mut facts, &mut context, &goal, decision)?
    else {
        return Err("missing fallback implementation".into());
    };
    assert_eq!(
        context.resolved(&mut facts.types, hole)?,
        TypeId::primitive(Primitive::U16)
    );
    assert_eq!(evidence.proof.scope_count(), 0);
    assert!(evidence.proof.equalities.is_empty());
    assert!(evidence.proof.outlives.is_empty());
    assert!(quantified_tests::valid(&facts, &evidence.proof, 0)?);
    Ok(())
}

#[test]
fn separate_candidates_keep_distinct_universes_of_the_same_arity() -> TestResult {
    for (implementation, expected) in [
        ("impl<'a>D<'a>for u8{}", true),
        ("impl D<'static>for u8{}", false),
    ] {
        let (report, mut facts) = tests::declarations(&format!(
            "trait A<'a>{{}}trait D<'a>{{}}trait B{{}}trait C{{}}trait Root{{}}impl<'a>A<'a>for u8{{}}impl<T:A<'static>>B for T{{}}impl<T:D<'static>>C for T{{}}impl<T:B+C>Root for T{{}}{implementation}"
        ))?;
        quantify(&mut facts, &report, "B", "A", 1, &[RegionId::bound(0, 0)?])?;
        quantify(&mut facts, &report, "C", "D", 1, &[RegionId::bound(0, 0)?])?;
        let root = root(&mut facts, &report, "Root")?;
        let mut state = State::instance(&mut facts, 0)?;
        assert!(state.prove(&mut facts, root, true)?);
        let proof = state.finish();
        assert_eq!(proof.scope_count(), 2);
        assert_eq!(quantified_tests::valid(&facts, &proof, 0)?, expected);
    }
    Ok(())
}

#[test]
fn sharing_one_universe_keeps_every_condition_in_either_order() -> TestResult {
    for reverse in [false, true] {
        for (implementation, expected) in [
            ("impl<'a>D<'a>for u8{}", true),
            ("impl D<'static>for u8{}", false),
        ] {
            let bounds = if reverse {
                "D<'static>+A<'static>"
            } else {
                "A<'static>+D<'static>"
            };
            let (report, mut facts) = tests::declarations(&format!(
                "trait A<'a>{{}}trait D<'a>{{}}trait B{{}}impl<'a>A<'a>for u8{{}}impl<T:{bounds}>B for T{{}}{implementation}"
            ))?;
            for name in ["A", "D"] {
                quantify(&mut facts, &report, "B", name, 1, &[RegionId::bound(0, 0)?])?;
            }
            let root = root(&mut facts, &report, "B")?;
            let mut state = State::instance(&mut facts, 0)?;
            assert!(state.prove(&mut facts, root, true)?);
            let proof = state.finish();
            assert_eq!(proof.scope_count(), 1);
            assert_eq!(quantified_tests::valid(&facts, &proof, 0)?, expected);
        }
    }
    Ok(())
}

#[test]
fn fresh_universes_do_not_turn_an_inductive_cycle_into_proof() -> TestResult {
    let (report, mut facts) =
        tests::declarations("trait A<'a>{}impl<'a,T:A<'static>>A<'a>for T{}")?;
    quantify(&mut facts, &report, "A", "A", 1, &[RegionId::bound(0, 0)?])?;
    let arguments = facts
        .types
        .intern_arguments(&[Argument::Lifetime(RegionId::STATIC)])?;
    let root = tests::predicate(
        &mut facts,
        tests::definition(&report, "A")?,
        BYTE,
        arguments,
    )?;
    let (mut context, goal) = tests::instantiate(&mut facts, root)?;
    let assumptions = Assumptions::default();
    let mut search = Search::new(tests::caller(&assumptions, None), Availability::Validated);
    let first = Frame::new(&facts, &mut context, &goal, 1)?;
    assert_eq!(search.run(&mut facts, first)?, Decision::Absent);
    assert_eq!(search.query.cycles, 1);
    assert!(search.query.completed.is_empty());
    assert!(search.query.steps < 256);
    Ok(())
}
