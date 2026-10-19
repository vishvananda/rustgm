//! Batched constraints preserve ordinary domains, cycle rejection and dependency wakes.
use rustgm::types::{
    Primitive, Store, TypeId,
    infer::{Constructor, Context, Domain, Error, TermId, arguments::Argument},
};
type TestResult = Result<(), Box<dyn std::error::Error>>;
const BYTE: TypeId = TypeId::primitive(Primitive::U8);
fn graph(store: &Store, domain: Domain) -> Result<(Context, Vec<TermId>), Error> {
    let mut context = Context::default();
    let mut terms = Vec::new();
    for current in [domain, Domain::Any, Domain::Any] {
        terms.push(context.variable(current)?);
    }
    for index in 0..3 {
        terms.push(context.construct(Constructor::Tuple, &[terms[index]])?);
    }
    terms.push(context.known(store, BYTE)?);
    terms.push(context.known(store, TypeId::primitive(Primitive::Bool))?);
    Ok((context, terms))
}
fn finish(
    context: &mut Context,
    store: &mut Store,
    terms: &[TermId],
) -> Result<Vec<TypeId>, Error> {
    let byte = context.known(store, BYTE)?;
    for &term in &terms[..3] {
        if context.unresolved_domain(term)?.is_some() {
            context.equate(store, term, byte)?;
        }
    }
    terms
        .iter()
        .map(|&term| context.resolved(store, term))
        .collect()
}
#[test]
fn all_small_two_equation_systems_match_sequential_unification() -> TestResult {
    let mut store = Store::default();
    let equations: Vec<_> = (0..8).flat_map(|a| (a..8).map(move |b| (a, b))).collect();
    for domain in [Domain::Any, Domain::Integer] {
        for &first in &equations {
            for &second in &equations {
                let (mut sequential, terms) = graph(&store, domain)?;
                let (mut batch, copied) = graph(&store, domain)?;
                let input = [first, second];
                let ordinary = input
                    .iter()
                    .try_for_each(|&(a, b)| sequential.equate(&store, terms[a], terms[b]));
                let pairs: Vec<_> = input
                    .iter()
                    .map(|&(a, b)| (Argument::Type(copied[a]), Argument::Type(copied[b])))
                    .collect();
                let combined = batch.equate_input_pairs(&store, &pairs);
                assert_eq!(
                    ordinary.is_ok(),
                    combined.is_ok(),
                    "domain={domain:?} input={input:?} ordinary={ordinary:?} combined={combined:?}"
                );
                if ordinary.is_ok() {
                    assert_eq!(
                        finish(&mut sequential, &mut store, &terms)?,
                        finish(&mut batch, &mut store, &copied)?,
                        "domain={domain:?} input={input:?}"
                    );
                }
            }
        }
    }
    Ok(())
}
#[test]
fn cycles_closed_by_three_equations_reject_in_every_order() -> TestResult {
    let store = Store::default();
    for order in [
        [0, 1, 2],
        [0, 2, 1],
        [1, 0, 2],
        [1, 2, 0],
        [2, 0, 1],
        [2, 1, 0],
    ] {
        let (mut context, terms) = graph(&store, Domain::Any)?;
        let system = [
            (terms[0], terms[4]),
            (terms[1], terms[5]),
            (terms[2], terms[0]),
        ];
        let pairs: Vec<_> = order
            .into_iter()
            .map(|index| {
                (
                    Argument::Type(system[index].0),
                    Argument::Type(system[index].1),
                )
            })
            .collect();
        assert_eq!(
            context.equate_input_pairs(&store, &pairs),
            Err(Error::Occurs)
        );
    }
    Ok(())
}
#[test]
fn partial_batch_refinement_wakes_and_rearms_real_resolution_watches() -> TestResult {
    let store = Store::default();
    let mut context = Context::default();
    let outer = context.variable(Domain::Any)?;
    let inner = context.variable(Domain::Any)?;
    let count = context.const_variable(&store, TypeId::primitive(Primitive::Usize))?;
    let watch = context.watch_resolution(&[outer], &[])?;
    assert_eq!(context.ready_resolution(&store)?, None);
    let array = context.construct(Constructor::ArrayInfer(count), &[inner])?;
    context.equate_input_pairs(&store, &[(Argument::Type(outer), Argument::Type(array))])?;
    assert_eq!(context.ready_resolution(&store)?, None);
    let byte = context.known(&store, BYTE)?;
    let fixed = context.construct(Constructor::Array(3), &[byte])?;
    context.equate_input_pairs(&store, &[(Argument::Type(array), Argument::Type(fixed))])?;
    assert_eq!(context.ready_resolution(&store)?, Some(watch));
    assert_eq!(context.ready_resolution(&store)?, None);
    Ok(())
}
