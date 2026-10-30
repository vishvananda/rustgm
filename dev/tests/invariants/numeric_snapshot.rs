//! Real retained source conditions map hints without manufacturing trait evidence.
use super::super::{
    Context, Decision, Declarations, Frame, Goal, Search, continuation::Progress,
    frames::Inference, tests,
};
use crate::{
    sema::traits::{assumptions::Assumptions, implementations::Availability},
    types::{
        ItemId, Primitive, TypeId,
        infer::{Domain, TermId, arguments::Argument},
    },
};
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

fn inputs(ordinary: bool, nested: bool) -> TestResult<(Declarations, Context, Goal, TermId)> {
    let fixture = if ordinary {
        "nested-into-lookalike"
    } else {
        "nested-into-wrapper"
    };
    let mut source =
        crate::test_source::fixture(&format!("pa31/tests/spec/source-from/{fixture}.rs"))?;
    if nested {
        source.push_str("trait Relay<T>:Sized{}impl<T,U>Relay<U>for T where T:Into<U>{}");
    }
    let report = crate::test_source::source_provider(&source, "snapshot")?;
    let facts = crate::sema::declarations::analyze(&report)?;
    let item = tests::definition(&report, if nested { "Relay" } else { "Into" })?;
    let mut context = Context::default();
    let input = context.variable(Domain::Float)?;
    let output = context.known(&facts.types, TypeId::primitive(Primitive::F32))?;
    let goal = Goal::trait_application(input, ItemId(item.0), vec![Argument::Type(output)]);
    Ok((facts, context, goal, input))
}
fn begin(
    search: &mut Search<'_>,
    facts: &mut Declarations,
    context: &mut Context,
    goal: &Goal,
) -> TestResult {
    search.query.refining = true;
    let mut frame = Frame::new(facts, context, goal, 1)?;
    frame.inference = Inference::Selecting;
    assert_eq!(search.begin(facts, frame)?, Progress::Inferred);
    search.publish_inputs(facts, context)?;
    Ok(())
}
fn drain(context: &mut Context, facts: &Declarations) -> TestResult {
    while context.ready_resolution(&facts.types)?.is_some() {}
    Ok(())
}

#[test]
fn selected_nested_source_hints_keep_evidence_pending_until_actual_fallback() -> TestResult {
    for nested in [false, true] {
        for cancel in [false, true] {
            let (mut facts, mut context, goal, input) = inputs(false, nested)?;
            let assumptions = Assumptions::default();
            let mut search = Search::new(
                tests::caller(&assumptions, None),
                Availability::Declarations,
            );
            begin(&mut search, &mut facts, &mut context, &goal)?;
            assert!(search.query.stack[0].selected.is_none());
            let hints = search.query.float_hints(&mut facts, &mut context)?;
            assert_eq!(hints, vec![context.root(input)?]);
            assert_eq!(context.unresolved_domain(input)?, Some(Domain::Float));
            assert!(search.query.stack[0].selected.is_none());
            let parent = context.watch_predicate(&goal)?;
            drain(&mut context, &facts)?;
            let hint = context
                .watch_float_hint(parent, hints[0])?
                .ok_or("missing parent hint")?;
            drain(&mut context, &facts)?;
            if cancel {
                context.cancel_watch(hint)?;
            }
            assert!(context.default_numbers()?);
            drain(&mut context, &facts)?;
            let expected = TypeId::primitive(if cancel {
                Primitive::F64
            } else {
                Primitive::F32
            });
            assert_eq!(context.resolved(&mut facts.types, input)?, expected);
            context.cancel_watch(hint)?;
            context.cancel_watch(parent)?;
            search.resume_at(search.query.allocations.end())?;
            search.refresh_inputs(&mut facts, &mut context)?;
            let Progress::Ready(Decision::Proven(root)) = search.poll(&mut facts)? else {
                return Err("actual nested source proof did not complete after fallback".into());
            };
            assert_eq!(search.query.nodes[root].goal.ty, expected);
        }
    }
    Ok(())
}

#[test]
fn explicit_f64_and_an_ordinary_from_homonym_supply_no_nested_hint() -> TestResult {
    for ordinary in [false, true] {
        let (mut facts, mut context, goal, input) = inputs(ordinary, true)?;
        let assumptions = Assumptions::default();
        let mut search = Search::new(
            tests::caller(&assumptions, None),
            Availability::Declarations,
        );
        begin(&mut search, &mut facts, &mut context, &goal)?;
        assert!(!search.query.stack[0].materialization_only());
        if !ordinary {
            let explicit = context.known(&facts.types, TypeId::primitive(Primitive::F64))?;
            context.equate(&facts.types, input, explicit)?;
        }
        assert!(
            search
                .query
                .float_hints(&mut facts, &mut context)?
                .is_empty()
        );
        if ordinary {
            assert_eq!(context.unresolved_domain(input)?, Some(Domain::Float));
        } else {
            assert_eq!(
                context.resolved(&mut facts.types, input)?,
                TypeId::primitive(Primitive::F64)
            );
        }
    }
    Ok(())
}

#[test]
fn a_completed_blanket_keeps_canonical_proof_pending_without_a_float_preference() -> TestResult {
    for nested in [false, true] {
        let mut source =
            crate::test_source::fixture("pa31/tests/spec/source-from/completed-blanket.rs")?;
        if nested {
            source.push_str("trait Into<T>:Sized{fn into(self)->T;}impl<T,U>Into<U>for T where U:From<T>{fn into(self)->U{<U as From<T>>::from(self)}}");
        }
        let report = crate::test_source::source_provider(&source, "blanket_snapshot")?;
        let mut facts = crate::sema::declarations::analyze(&report)?;
        let item = tests::definition(&report, if nested { "Into" } else { "From" })?;
        let mut context = Context::default();
        let input = context.variable(Domain::Float)?;
        let output = context.known(&facts.types, TypeId::primitive(Primitive::F32))?;
        let (subject, argument) = if nested {
            (input, output)
        } else {
            (output, input)
        };
        let goal = Goal::trait_application(subject, ItemId(item.0), vec![Argument::Type(argument)]);
        let assumptions = Assumptions::default();
        let mut search = Search::new(
            tests::caller(&assumptions, None),
            Availability::Declarations,
        );
        begin(&mut search, &mut facts, &mut context, &goal)?;
        assert_eq!(search.query.stack[0].materialization_only(), !nested);
        assert!(search.query.stack[0].selected.is_none());
        assert!(
            search
                .query
                .float_hints(&mut facts, &mut context)?
                .is_empty()
        );
        let watch = context.watch_predicate(&goal)?;
        drain(&mut context, &facts)?;
        assert!(context.default_numbers()?);
        assert_eq!(
            context.resolved(&mut facts.types, input)?,
            TypeId::primitive(Primitive::F64)
        );
        context.cancel_watch(watch)?;
        search.resume_at(search.query.allocations.end())?;
        search.refresh_inputs(&mut facts, &mut context)?;
        assert!(matches!(
            search.poll(&mut facts)?,
            Progress::Ready(Decision::Proven(_))
        ));
    }
    Ok(())
}

fn failed_materialization_payment(
    facts: &mut Declarations,
    item: ItemId,
    count: usize,
) -> TestResult {
    assert!(
        crate::sema::header_attributes::source_payments(facts.must_use_contracts.as_deref())?
            .charge(0, crate::support::crates::MAX_BYTES)
            .is_err()
    );
    for _ in 0..2 {
        let mut context = Context::default();
        let subject = context.known(&facts.types, TypeId::primitive(Primitive::F32))?;
        let mut arguments = Vec::new();
        let input = context.variable(Domain::Float)?;
        arguments.push(Argument::Type(input));
        for _ in 1..count {
            arguments.push(Argument::Type(context.variable(Domain::Float)?));
        }
        let goal = Goal::trait_application(subject, item, arguments);
        let assumptions = Assumptions::default();
        let mut search = Search::new(
            tests::caller(&assumptions, None),
            Availability::Declarations,
        );
        assert!(begin(&mut search, facts, &mut context, &goal).is_err());
        assert_eq!(context.unresolved_domain(input)?, Some(Domain::Float));
    }
    Ok(())
}

#[test]
fn numeric_materialization_is_paid_monotone_and_linear_in_actual_selected_edges() -> TestResult {
    use std::fmt::Write as _;
    let mut observed = Vec::new();
    for count in [8, 32, 128] {
        let mut source =
            crate::test_source::fixture("pa31/tests/spec/source-from/completed-blanket.rs")?;
        let formals = (0..count)
            .map(|index| format!("T{index}"))
            .collect::<Vec<_>>()
            .join(",");
        write!(
            source,
            "trait Wide<{formals}>:Sized{{}}impl<{formals}>Wide<{formals}>for f32{{}}"
        )?;
        let report = crate::test_source::source_provider(&source, "wide_materialization")?;
        let mut facts = crate::sema::declarations::analyze(&report)?;
        let item = tests::definition(&report, "Wide")?;
        let mut context = Context::default();
        let subject = context.known(&facts.types, TypeId::primitive(Primitive::F32))?;
        let mut arguments = Vec::new();
        for _ in 0..count {
            arguments.push(Argument::Type(context.variable(Domain::Float)?));
        }
        let goal = Goal::trait_application(subject, ItemId(item.0), arguments);
        let assumptions = Assumptions::default();
        let mut search = Search::new(
            tests::caller(&assumptions, None),
            Availability::Declarations,
        );
        begin(&mut search, &mut facts, &mut context, &goal)?;
        assert!(search.query.stack[0].materialization_only());
        observed.push(search.query.steps + context.steps);
        let frame = &mut search.query.stack[0];
        let trial = frame.trial.as_mut().ok_or("actual selected trial")?;
        let mut requirements = trial
            .requirements
            .take()
            .ok_or("retained numeric requirements")?;
        assert_eq!(requirements.numeric_cursor, trial.instance.goals.len());
        let retained = trial.inputs.context.retained_allocation();
        let before = trial.inputs.context.steps;
        for _ in 0..32 {
            trial.classify_materialization(&facts, &mut requirements)?;
        }
        assert_eq!(trial.inputs.context.steps - before, 32);
        assert_eq!(trial.inputs.context.retained_allocation(), retained);
        requirements.numeric_cursor = 0;
        trial.inputs.context.steps = 1_000_000;
        for _ in 0..2 {
            assert!(
                trial
                    .classify_materialization(&facts, &mut requirements)
                    .is_err()
            );
            assert_eq!(requirements.numeric_cursor, 0);
            assert_eq!(trial.inputs.context.retained_allocation(), retained);
        }
        failed_materialization_payment(&mut facts, ItemId(item.0), count)?;
    }
    assert!(observed[1] < observed[0] * 5);
    assert!(observed[2] < observed[1] * 5);
    eprintln!(
        "numeric materialization 8/32/128 work={observed:?}; Requirements={} Frame={} Trial={} Query={} Node={}",
        size_of::<super::super::builtins::Requirements>(),
        size_of::<Frame>(),
        size_of::<super::super::frames::Trial>(),
        size_of::<super::super::Query>(),
        size_of::<super::super::Node>()
    );
    assert_eq!(size_of::<super::super::builtins::Requirements>(), 64);
    assert_eq!(size_of::<Frame>(), 1312);
    assert_eq!(size_of::<super::super::Node>(), 160);
    Ok(())
}

#[test]
fn retained_source_hints_revalidate_the_current_completed_language_header() -> TestResult {
    let (mut facts, mut context, goal, input) = inputs(false, true)?;
    let assumptions = Assumptions::default();
    let mut search = Search::new(
        tests::caller(&assumptions, None),
        Availability::Declarations,
    );
    begin(&mut search, &mut facts, &mut context, &goal)?;
    facts.generics.traits = None;
    assert!(search.query.float_hints(&mut facts, &mut context).is_err());
    assert_eq!(context.unresolved_domain(input)?, Some(Domain::Float));
    Ok(())
}

#[test]
fn disjoint_ambiguous_candidates_cannot_publish_their_nested_float_hints() -> TestResult {
    let mut source =
        crate::test_source::fixture("pa31/tests/spec/source-from/nested-into-wrapper.rs")?;
    source.push_str("struct Left;struct Right;trait Route<X>:Sized{}impl<X>Route<X>for Left where f32:From<X>{}impl<X>Route<X>for Right where f32:From<X>{}");
    let report = crate::test_source::source_provider(&source, "ambiguous_snapshot")?;
    let mut facts = crate::sema::declarations::analyze(&report)?;
    let item = tests::definition(&report, "Route")?;
    let mut context = Context::default();
    let subject = context.variable(Domain::Any)?;
    let input = context.variable(Domain::Float)?;
    let goal = Goal::trait_application(subject, ItemId(item.0), vec![Argument::Type(input)]);
    let assumptions = Assumptions::default();
    let mut search = Search::new(
        tests::caller(&assumptions, None),
        Availability::Declarations,
    );
    search.query.refining = true;
    // An earlier discarded candidate may have observed a real hint. That cold
    // presence state cannot authorize publishing this ambiguous root's inputs.
    search.query.publication.observe(true);
    let mut frame = Frame::new(&facts, &mut context, &goal, 1)?;
    frame.inference = Inference::Selecting;
    assert_eq!(
        search.begin(&mut facts, frame)?,
        Progress::Ready(Decision::Deferred)
    );
    assert!(search.query.publication.has_hints());
    assert!(search.query.float_hints(&mut facts, &mut context).is_err());
    assert_eq!(context.unresolved_domain(subject)?, Some(Domain::Any));
    assert_eq!(context.unresolved_domain(input)?, Some(Domain::Float));
    Ok(())
}
