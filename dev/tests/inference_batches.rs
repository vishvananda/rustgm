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

#[test]
fn mixed_inputs_keep_lifetime_equations_in_source_order() -> TestResult {
    use rustgm::types::RegionId;
    let store = Store::default();
    for callables in [false, true] {
        let mut context = Context::default();
        let byte = context.known(&store, BYTE)?;
        let mut references = Vec::new();
        for id in [1, 2, 5, 6] {
            references.push(context.construct(
                Constructor::Reference {
                    mutable: false,
                    region: RegionId(id),
                },
                &[byte],
            )?);
        }
        let mut inputs = vec![
            (Argument::Type(references[0]), Argument::Type(references[1])),
            (
                Argument::Lifetime(RegionId(3)),
                Argument::Lifetime(RegionId(4)),
            ),
            (Argument::Type(references[2]), Argument::Type(references[3])),
        ];
        if callables {
            let kind = Constructor::Function {
                bound_regions: 0,
                parameters: 0,
                unsafe_: false,
            };
            let a = context.construct(kind, &[byte])?;
            let b = context.construct(kind, &[byte])?;
            inputs.push((Argument::Type(a), Argument::Type(b)));
        }
        context.equate_input_pairs(&store, &inputs)?;
        assert_eq!(
            context.region_equalities,
            [
                (RegionId(1), RegionId(2)),
                (RegionId(3), RegionId(4)),
                (RegionId(5), RegionId(6)),
            ]
        );
    }
    Ok(())
}

#[test]
fn one_canonical_wide_input_binds_shared_prefixes_with_linear_work() -> TestResult {
    use rustgm::types::Kind;
    for count in [16, 128, 1024, 4096] {
        let mut store = Store::default();
        let mut prefix = BYTE;
        let mut expected = Vec::new();
        for _ in 0..count {
            let children = store.intern_list(&[prefix])?;
            prefix = store.intern(Kind::Tuple(children))?;
            expected.push(prefix);
        }
        let children = store.intern_list(&expected)?;
        let known = store.intern(Kind::Tuple(children))?;
        let mut context = Context::default();
        // Other constraints may have expanded the shared descendants already.
        // The wide root remains canonical, but its cached children are graphs.
        let mut expanded = context.known(&store, prefix)?;
        for _ in 0..count {
            expanded = context.child(&store, expanded, 0)?;
        }
        let inputs = (0..count)
            .map(|_| context.variable(Domain::Any))
            .collect::<Result<Vec<_>, _>>()?;
        let pattern = context.construct(Constructor::Tuple, &inputs)?;
        let known = context.known(&store, known)?;
        let before = context.steps;
        context.equate_input_pairs(&store, &[(Argument::Type(pattern), Argument::Type(known))])?;
        let work = context.steps - before;
        eprintln!("single canonical width={count} work={work}");
        assert!(work <= 64 * count + 128, "{work} at {count}");
        let solved = context.finish(&mut store)?;
        for (input, ty) in inputs.into_iter().zip(expected) {
            assert_eq!(solved.types[input.0 as usize], ty);
        }
    }
    Ok(())
}
