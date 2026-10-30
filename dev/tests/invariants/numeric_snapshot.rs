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
