//! Input readiness is monotonic, local to a body, and independent of result evidence.
use rustgm::types::{
    ItemId, Kind, Primitive, Store, TypeId,
    infer::{Constructor, Context, Domain, Error, TermId, arguments::Argument},
    projection::Schema,
};
type TestResult = Result<(), Box<dyn std::error::Error>>;
const BYTE: TypeId = TypeId::primitive(Primitive::U8);
const SIZE: TypeId = TypeId::primitive(Primitive::Usize);

#[test]
fn projected_inputs_wait_for_every_mixed_argument_without_proving_a_result() -> TestResult {
    let mut store = Store::default();
    let item = ItemId(7);
    store.declare_projection(
        item,
        Schema {
            trait_item: ItemId(6),
            sized: true,
            copies: rustgm::types::copy::Properties::default(),
        },
    )?;
    let mut context = Context::default();
    let subject = context.variable(Domain::Any)?;
    let argument = context.variable(Domain::Any)?;
    let count = context.const_variable(&store, SIZE)?;
    let projected = context.projected(
        &store,
        item,
        subject,
        &[Argument::Type(argument), Argument::Const(count)],
        &[Argument::Type(subject), Argument::Const(count)],
    )?;
    let query = context.watch_resolution(&[projected], &[])?;
    assert_eq!(context.ready_resolution(&store)?, None);
    let byte = context.known(&store, BYTE)?;
    context.equate(&store, subject, byte)?;
    assert_eq!(context.ready_resolution(&store)?, None);
    context.equate(&store, argument, byte)?;
    assert_eq!(context.ready_resolution(&store)?, None);
    let array = context.construct(Constructor::ArrayInfer(count), &[byte])?;
    let fixed = context.construct(Constructor::Array(5), &[byte])?;
    context.equate(&store, array, fixed)?;
    assert_eq!(context.ready_resolution(&store)?, Some(query));
    assert_eq!(context.ready_resolution(&store)?, None);
    let canonical = context.resolved(&mut store, projected)?;
    assert!(matches!(store.kind(canonical), Some(Kind::Projection(_))));
    assert_eq!(context.head(&store, projected), Err(Error::Normalization));
    assert_eq!(
        context.equate(&store, projected, byte),
        Err(Error::Normalization)
    );
    Ok(())
}

#[test]
fn merging_watched_roots_coalesces_waits_and_reports_each_query_once() -> TestResult {
    for reverse in [false, true] {
        let store = Store::default();
        let mut context = Context::default();
        let first = context.variable(Domain::Any)?;
        let second = context.variable(Domain::Any)?;
        let a = context.watch_resolution(&[first, second, first], &[])?;
        let b = context.watch_resolution(&[second], &[])?;
        assert_eq!(context.ready_resolution(&store)?, None);
        if reverse {
            context.equate(&store, second, first)?;
        } else {
            context.equate(&store, first, second)?;
        }
        assert_eq!(context.ready_resolution(&store)?, None);
        let known = context.known(&store, BYTE)?;
        context.equate(&store, first, known)?;
        let mut ready = Vec::new();
        while let Some(id) = context.ready_resolution(&store)? {
            ready.push(id);
        }
        assert_eq!(ready.len(), 2);
        assert!(ready.contains(&a));
        assert!(ready.contains(&b));
    }
    Ok(())
}

#[test]
fn watches_follow_newly_exposed_children_without_rescanning_ancestors() -> TestResult {
    let store = Store::default();
    let mut context = Context::default();
    let mut leaf = context.variable(Domain::Any)?;
    let query = context.watch_resolution(&[leaf], &[])?;
    assert_eq!(context.ready_resolution(&store)?, None);
    let start = context.steps;
    for _ in 0..512 {
        let child = context.variable(Domain::Any)?;
        let tuple = context.construct(Constructor::Tuple, &[child, child])?;
        context.equate(&store, leaf, tuple)?;
        assert_eq!(context.ready_resolution(&store)?, None);
        leaf = child;
    }
    let known = context.known(&store, BYTE)?;
    context.equate(&store, leaf, known)?;
    assert_eq!(context.ready_resolution(&store)?, Some(query));
    assert_eq!(context.ready_resolution(&store)?, None);
    assert!(context.steps - start < 512 * 20);
    Ok(())
}

#[test]
fn unrelated_queries_stay_asleep_when_one_input_changes() -> TestResult {
    let store = Store::default();
    let mut context = Context::default();
    let mut queries = Vec::new();
    for _ in 0..4_096 {
        let input = context.variable(Domain::Any)?;
        queries.push((input, context.watch_resolution(&[input], &[])?));
    }
    assert_eq!(context.ready_resolution(&store)?, None);
    let known = context.known(&store, BYTE)?;
    for index in [3_000, 1, 2_000, 4_095] {
        let start = context.steps;
        let (input, id) = queries[index];
        context.equate(&store, input, known)?;
        assert_eq!(context.ready_resolution(&store)?, Some(id));
        assert_eq!(context.ready_resolution(&store)?, None);
        assert!(context.steps - start < 32);
    }
    Ok(())
}

#[test]
fn fallback_wakes_types_but_does_not_default_const_inputs() -> TestResult {
    let store = Store::default();
    let mut context = Context::default();
    let integer = context.variable(Domain::Integer)?;
    let float = context.variable(Domain::Float)?;
    let never = context.variable(Domain::Any)?;
    let constant = context.const_variable(&store, SIZE)?;
    let numeric = context.watch_resolution(&[integer, float], &[])?;
    let diverging = context.watch_resolution(&[never], &[])?;
    context.watch_resolution(&[], &[constant])?;
    assert_eq!(context.ready_resolution(&store)?, None);
    assert!(context.default_numbers()?);
    assert_eq!(context.ready_resolution(&store)?, Some(numeric));
    assert_eq!(context.ready_resolution(&store)?, None);
    context.default_never(never, rustgm::support::edition::Edition::Rust2024)?;
    assert_eq!(context.ready_resolution(&store)?, Some(diverging));
    assert_eq!(context.ready_resolution(&store)?, None);
    assert!(!context.default_numbers()?);
    Ok(())
}

#[test]
fn edition_fallback_changes_only_unconstrained_never_coercion_roots() -> TestResult {
    use rustgm::support::edition::Edition;
    for (edition, expected) in [
        (Edition::Rust2021, TypeId::UNIT),
        (Edition::Rust2024, TypeId::NEVER),
    ] {
        let mut store = Store::default();
        let mut context = Context::default();
        let first = context.variable(Domain::Any)?;
        let alias = context.variable(Domain::Any)?;
        context.equate(&store, first, alias)?;
        let ordinary = context.variable(Domain::Any)?;
        let integer = context.variable(Domain::Integer)?;
        let known = context.known(&store, BYTE)?;
        let waiting = context.watch_resolution(&[first, alias], &[])?;
        assert_eq!(context.ready_resolution(&store)?, None);
        context.default_never(alias, edition)?;
        assert_eq!(context.ready_resolution(&store)?, Some(waiting));
        assert_eq!(context.resolved(&mut store, first)?, expected);
        assert_eq!(context.resolved(&mut store, alias)?, expected);
        context.default_never(first, edition)?;
        context.default_never(integer, edition)?;
        context.default_never(known, edition)?;
        assert_eq!(context.ready_resolution(&store)?, None);
        assert_eq!(context.unresolved_domain(ordinary)?, Some(Domain::Any));
        assert_eq!(context.unresolved_domain(integer)?, Some(Domain::Integer));
        assert_eq!(context.resolved(&mut store, known)?, BYTE);
    }
    Ok(())
}

#[test]
fn const_unions_and_updates_before_the_first_poll_are_observed() -> TestResult {
    let store = Store::default();
    let mut context = Context::default();
    let a = context.const_variable(&store, SIZE)?;
    let b = context.const_variable(&store, SIZE)?;
    let query = context.watch_resolution(&[], &[a, b, a])?;
    context.equate_consts(a, b)?;
    assert_eq!(context.ready_resolution(&store)?, None);
    let c = context.const_variable(&store, SIZE)?;
    context.equate_consts(b, c)?;
    assert_eq!(context.ready_resolution(&store)?, None);
    let byte = context.known(&store, BYTE)?;
    let array = context.construct(Constructor::ArrayInfer(c), &[byte])?;
    let fixed = context.construct(Constructor::Array(8), &[byte])?;
    context.equate(&store, array, fixed)?;
    assert_eq!(context.ready_resolution(&store)?, Some(query));
    assert_eq!(context.ready_resolution(&store)?, None);
    let variable = context.variable(Domain::Any)?;
    let query = context.watch_resolution(&[variable], &[])?;
    context.equate(&store, variable, byte)?;
    assert_eq!(context.ready_resolution(&store)?, Some(query));
    Ok(())
}

#[test]
fn imported_probe_constraints_do_not_wake_the_source_body() -> TestResult {
    let store = Store::default();
    let mut source = Context::default();
    let unknown = source.variable(Domain::Any)?;
    let count = source.const_variable(&store, SIZE)?;
    let array = source.construct(Constructor::ArrayInfer(count), &[unknown])?;
    let query = source.watch_resolution(&[array], &[])?;
    assert_eq!(source.ready_resolution(&store)?, None);
    let mut probe = Context::default();
    let imported = probe.import(&store, &mut source, array)?;
    let probe_query = probe.watch_resolution(&[imported], &[])?;
    let byte = probe.known(&store, BYTE)?;
    let fixed = probe.construct(Constructor::Array(3), &[byte])?;
    probe.equate(&store, imported, fixed)?;
    assert_eq!(probe.ready_resolution(&store)?, Some(probe_query));
    assert_eq!(source.ready_resolution(&store)?, None);
    let byte = source.known(&store, BYTE)?;
    let fixed = source.construct(Constructor::Array(3), &[byte])?;
    source.equate(&store, array, fixed)?;
    assert_eq!(source.ready_resolution(&store)?, Some(query));
    Ok(())
}

#[test]
fn empty_known_invalid_and_exhausted_queries_have_explicit_outcomes() -> TestResult {
    let store = Store::default();
    let mut context = Context::default();
    let empty = context.watch_resolution(&[], &[])?;
    assert_eq!(context.ready_resolution(&store)?, Some(empty));
    let byte = context.known(&store, BYTE)?;
    let known = context.watch_resolution(&[byte, byte], &[])?;
    assert_eq!(context.ready_resolution(&store)?, Some(known));
    assert_eq!(context.ready_resolution(&store)?, None);
    assert_eq!(
        context.watch_resolution(&[TermId(u32::MAX)], &[]),
        Err(Error::InvalidIdentity)
    );
    context.steps = 1_000_000;
    assert_eq!(context.watch_resolution(&[byte], &[]), Err(Error::Budget));
    Ok(())
}
