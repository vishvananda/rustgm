//! Partial inputs remain distinct from proof and retain one candidate across resumption.
#[path = "retained_candidates/lifetimes.rs"]
mod lifetimes;
use super::{
    Context, Decision, Declarations, Frame, Goal, Search, continuation::Progress,
    frames::Inference, header_tests, tests,
};
use crate::{
    resolve::model::DefId,
    sema::traits::{assumptions::Assumptions, implementations::Availability},
    types::{
        ItemId, Kind, Primitive, TypeId,
        arguments::Argument,
        infer::{Domain, Instantiation, TermId, arguments::Argument as Inferred},
        parameters::{Parameter, ParameterKind},
    },
};
use std::fmt::Write;
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

fn open_goal(facts: &mut Declarations, item: DefId) -> TestResult<(Context, Goal, TermId)> {
    let parameter = facts.types.intern_parameter(Parameter {
        owner: ItemId(1_000_000),
        index: 0,
        kind: ParameterKind::Type { sized: true },
    })?;
    let ty = facts.types.intern(Kind::Parameter(parameter))?;
    let args = facts.types.intern_arguments(&[Argument::Type(ty)])?;
    let predicate = tests::predicate(facts, item, TypeId::primitive(Primitive::U8), args)?;
    let mut context = Context::default();
    let unknown = context.variable(Domain::Any)?;
    let mut substitution = Instantiation::new(
        &mut context,
        &facts.types,
        &[(parameter, Inferred::Type(unknown))],
    )?;
    let goal = context.instantiate_predicate(&mut facts.types, predicate, &mut substitution)?;
    Ok((context, goal, unknown))
}
fn begin(
    search: &mut Search<'_>,
    facts: &mut Declarations,
    context: &mut Context,
    goal: &Goal,
) -> TestResult<Progress> {
    search.query.refining = true;
    let mut frame = Frame::new(facts, context, goal, 1)?;
    frame.inference = Inference::Selecting;
    Ok(search.begin(facts, frame)?)
}
fn complete_inner(
    facts: &mut Declarations,
    context: &mut Context,
    unknown: TermId,
    ty: TypeId,
) -> TestResult {
    let inner = context.child(&facts.types, unknown, 0)?;
    let value = context.known(&facts.types, ty)?;
    context.equate(&facts.types, inner, value)?;
    Ok(())
}

#[test]
fn retained_candidate_waits_for_its_condition_header_after_caller_inference() -> TestResult {
    let (report, mut facts) = tests::declarations(
        "struct W<T>(T);trait A<T>{}trait B{}impl B for u16{}impl<T:B>A<W<T>>for u8{}",
    )?;
    let (mut context, goal, unknown) = open_goal(&mut facts, tests::definition(&report, "A")?)?;
    let mut delayed = header_tests::delay(&mut facts, &[tests::definition(&report, "B")?])?;
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
    assert!(search.query.nodes.is_empty());
    complete_inner(
        &mut facts,
        &mut context,
        unknown,
        TypeId::primitive(Primitive::U16),
    )?;
    search.resume_at(4)?;
    search.refresh_inputs(&mut facts, &mut context)?;
    let header = delayed.pop().ok_or("missing condition header")?;
    assert_eq!(
        search.poll(&mut facts)?,
        Progress::Pending(crate::sema::traits::selection::Dependency::Header(
            header.owner
        ))
    );
    header_tests::stable_poll(&mut search, &mut facts, header.owner)?;
    header_tests::publish(&mut facts, header)?;
    search.resume_at(7)?;
    let Progress::Ready(Decision::Proven(root)) = search.poll(&mut facts)? else {
        return Err("retained condition did not complete".into());
    };
    context.equate_proved_predicate(&mut facts.types, &goal, search.query.nodes[root].goal)?;
    assert!(search.query.suspended.iter().all(Option::is_none));
    let evidence = search.evidence(&facts, root)?;
    assert_eq!(evidence.proof.owned_regions().count(), 0);
    for id in 1..7 {
        assert!(!evidence.proof.owns_region(crate::types::RegionId(id)));
    }
    Ok(())
}

#[test]
fn partial_and_complete_alternatives_do_not_publish_a_guess() -> TestResult {
    for alternatives in [
        "impl<T>A<W<T>>for u8{}impl A<u16>for u8{}",
        "impl A<u16>for u8{}impl<T>A<W<T>>for u8{}",
        "impl<T>A<W<T>>for u8{}impl<T>A<[T;3]>for u8{}",
    ] {
        let (report, mut facts) =
            tests::declarations(&format!("struct W<T>(T);trait A<T>{{}}{alternatives}"))?;
        let (mut context, goal, unknown) = open_goal(&mut facts, tests::definition(&report, "A")?)?;
        let assumptions = Assumptions::default();
        let mut search = Search::new(
            tests::caller(&assumptions, None),
            Availability::Declarations,
        );
        assert_eq!(
            begin(&mut search, &mut facts, &mut context, &goal)?,
            Progress::Ready(Decision::Deferred)
        );
        assert!(matches!(
            context.resolved(&mut facts.types, unknown),
            Err(crate::types::infer::Error::Unresolved)
        ));
    }
    Ok(())
}

#[test]
fn a_pending_builtin_is_checked_after_partial_publication() -> TestResult {
    let (report, mut facts) =
        tests::declarations("struct W<T>(T);struct N;trait A<T>{}impl<T:Copy>A<W<T>>for u8{}")?;
    let noncopy =
        facts.definitions[tests::definition(&report, "N")?.0 as usize].ok_or("missing N")?;
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
    complete_inner(&mut facts, &mut context, unknown, noncopy)?;
    search.resume_at(search.query.allocations.end())?;
    search.refresh_inputs(&mut facts, &mut context)?;
    assert_eq!(search.poll(&mut facts)?, Progress::Ready(Decision::Absent));
    Ok(())
}

#[test]
fn deep_retained_conditions_have_flat_storage_and_bounded_work() -> TestResult {
    for depth in [8, 32, 128, 512, 2048] {
        let mut source = String::from("struct W<T>(T);trait B0<T>{}impl<T>B0<W<T>>for u8{}");
        for i in 1..=depth {
            write!(
                source,
                "trait B{i}<T>{{}}impl<T>B{i}<T>for u8 where u8:B{}<T>{{}}",
                i - 1
            )?;
        }
        let (report, mut facts) = tests::declarations(&source)?;
        let (mut context, goal, unknown) = open_goal(
            &mut facts,
            tests::definition(&report, &format!("B{depth}"))?,
        )?;
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
        let slots = search.query.suspended.len();
        assert_eq!(slots, depth);
        let before = context.steps;
        complete_inner(
            &mut facts,
            &mut context,
            unknown,
            TypeId::primitive(Primitive::U16),
        )?;
        search.resume_at(search.query.allocations.end())?;
        search.refresh_inputs(&mut facts, &mut context)?;
        let Progress::Ready(Decision::Proven(root)) = search.poll(&mut facts)? else {
            return Err("retained chain was not proved".into());
        };
        let after = search.query.steps;
        eprintln!("partial chain depth={depth} initial={before} resumed={after} slots={slots}");
        assert_eq!(search.query.suspended.len(), slots);
        assert!(search.query.suspended.iter().all(Option::is_none));
        assert_eq!(search.query.nodes.len(), depth + 1);
        let proof = search.evidence(&facts, root)?;
        assert_eq!(proof.proof.owned_regions().count(), 0);
        assert!(
            before <= 200 * (depth + 1),
            "initial work {before} at {depth}"
        );
        assert!(
            after <= 100 * (depth + 1),
            "resumed work {after} at {depth}"
        );
    }
    Ok(())
}

#[test]
fn sibling_constraints_wake_only_the_pending_condition_that_uses_them() -> TestResult {
    for bounds in ["T:B,u8:C<T>", "u8:C<T>,T:B"] {
        let source = format!(
            "struct W<T>(T);trait A<T>{{}}trait B{{}}trait C<T>{{}}impl B for W<u16>{{}}impl<T>C<W<T>>for u8{{}}impl<T>A<T>for u8 where {bounds}{{}}"
        );
        let (report, mut facts) = tests::declarations(&source)?;
        let (mut context, goal, unknown) = open_goal(&mut facts, tests::definition(&report, "A")?)?;
        let assumptions = Assumptions::default();
        let mut search = Search::new(
            tests::caller(&assumptions, None),
            Availability::Declarations,
        );
        let Progress::Ready(Decision::Proven(root)) =
            begin(&mut search, &mut facts, &mut context, &goal)?
        else {
            return Err("sibling input notification failed to complete the candidate".into());
        };
        context.equate_proved_predicate(&mut facts.types, &goal, search.query.nodes[root].goal)?;
        let inner = context.child(&facts.types, unknown, 0)?;
        assert_eq!(
            context.resolved(&mut facts.types, inner)?,
            TypeId::primitive(Primitive::U16)
        );
    }
    Ok(())
}
