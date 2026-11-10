//! Account every nested owner without changing the caller's reported own work.
use super::{Fixture, TestResult, fixture, nominal, premises, prove_sized, scoped};
use crate::{
    sema::{
        nominals::{Proof, traits::State},
        traits::{
            assumptions::Assumptions,
            implementations::Availability,
            selection::{self, Caller, OwnedCaller, search::Search},
        },
    },
    types::{Kind, TypeId, infer::Context, parameters::environment::Environment},
};
use std::sync::Arc;

struct Observation {
    result: Result<bool, &'static str>,
    own: usize,
    nested: usize,
    level: u16,
    tally: usize,
}

fn chain(fixture: &mut Fixture, depth: usize, premised: bool) -> TestResult<TypeId> {
    let mut ty = if premised {
        fixture.unsized_
    } else {
        fixture.sized
    };
    for _ in 0..depth {
        ty = nominal(&mut fixture.facts, fixture.projected, ty)?;
    }
    Ok(ty)
}

fn proof(fixture: &Fixture, environment: &Arc<Environment>, premised: bool) -> TestResult<Proof> {
    if premised {
        let mut proof = Proof::new(None);
        proof.assume_placeholders(&fixture.facts.types, premises(&[fixture.unsized_], &[])?)?;
        Ok(proof)
    } else {
        Ok(Proof::new(Some(Arc::clone(environment))))
    }
}

fn observe(
    fixture: &mut Fixture,
    environment: &Arc<Environment>,
    depth: usize,
    premised: bool,
) -> TestResult<Observation> {
    observe_at(
        fixture,
        environment,
        depth,
        premised,
        selection::Nesting::ROOT,
    )
}

fn observe_at(
    fixture: &mut Fixture,
    environment: &Arc<Environment>,
    depth: usize,
    premised: bool,
    nesting: selection::Nesting,
) -> TestResult<Observation> {
    let ty = chain(fixture, depth, premised)?;
    let mut proof = proof(fixture, environment, premised)?;
    proof.nest_at(nesting);
    selection::take_nested_observation();
    let result = prove_sized(&mut proof, &mut fixture.facts, ty);
    let (level, tally) = selection::take_nested_observation();
    Ok(Observation {
        result,
        own: proof.own_work(),
        nested: proof.nested_work(),
        level,
        tally,
    })
}

#[test]
fn nested_work_is_the_sum_of_each_owners_own_work_once() -> TestResult {
    let mut fixture = fixture()?;
    let environment = scoped(&fixture.facts)?;
    for depth in 1..=4 {
        for premised in [false, true] {
            let row = observe(&mut fixture, &environment, depth, premised)?;
            assert_eq!(row.result, Ok(true));
            assert_eq!(
                row.own,
                if premised {
                    20 + depth.max(2)
                } else {
                    18 * depth + 2
                }
            );
            assert_eq!(row.nested, row.tally);
            assert!(row.own + row.nested <= 80 * depth * depth);
            assert_eq!(
                row.level,
                if !premised {
                    3
                } else if depth == 1 {
                    2
                } else {
                    5
                }
            );
            eprintln!(
                "conservation depth={depth} premised={premised} own={} nested={}",
                row.own, row.nested
            );
        }
    }
    Ok(())
}

#[test]
fn nested_work_budget_remains_live_after_reuse() -> TestResult {
    const CAUSE: &str = "size normalization work budget exceeded";
    for premised in [false, true] {
        let mut fixture = fixture()?;
        let environment = scoped(&fixture.facts)?;
        let ty = chain(&mut fixture, 4, premised)?;
        let mut proof = proof(&fixture, &environment, premised)?;
        proof.preload_nested(&mut fixture.facts, 0)?;
        let preload = 1_000_000 - proof.own_work() - 5;
        proof.preload_nested(&mut fixture.facts, preload)?;
        assert_eq!(prove_sized(&mut proof, &mut fixture.facts, ty), Err(CAUSE));
        let mut saturated = self::proof(&fixture, &environment, premised)?;
        assert_eq!(
            saturated.preload_nested(&mut fixture.facts, usize::MAX),
            Err(CAUSE)
        );
        assert_eq!(
            prove_sized(&mut saturated, &mut fixture.facts, ty),
            Err(CAUSE)
        );
    }
    Ok(())
}

#[test]
fn depth_failure_reaches_the_root_without_stack_overflow() -> TestResult {
    let worker = std::thread::Builder::new()
        .stack_size(16 * 1024 * 1024)
        .spawn(|| -> Result<(), String> {
            let mut nesting = selection::Nesting::ROOT;
            for _ in 0..92 {
                nesting = nesting.nested();
            }
            for premised in [false, true] {
                let mut fixture = fixture().map_err(|error| error.to_string())?;
                let environment = scoped(&fixture.facts).map_err(|error| error.to_string())?;
                let row = observe_at(&mut fixture, &environment, 1, premised, nesting)
                    .map_err(|error| error.to_string())?;
                assert_eq!(row.level, 94, "result: {:?}", row.result);
                assert_eq!(row.result, Err(selection::NESTED_PROOF_DEPTH));
            }
            Ok(())
        })?;
    worker
        .join()
        .map_err(|_| "nested proof worker panicked")??;
    assert!(crate::sema::census::exhausts_budget(
        selection::NESTED_PROOF_DEPTH
    ));
    Ok(())
}

#[test]
fn repeated_builtin_calls_charge_only_the_new_work() -> TestResult {
    let mut fixture = fixture()?;
    let ty = chain(&mut fixture, 3, false)?;
    let environment = scoped(&fixture.facts)?;
    let assumptions = Assumptions::default();
    let caller = Caller::root(&assumptions, Some(&environment), 1);
    let mut search = Search::assuming(&fixture.facts.types, caller, Availability::Validated)?;
    let before = search.query.builtins.own_work() + search.query.builtins.nested_work();
    for _ in 0..2 {
        let previous = search.query.nested;
        assert!(
            search.with_builtins(&mut fixture.facts, |proof, facts| prove_sized(
                proof, facts, ty
            ))?
        );
        let actual = search.query.builtins.own_work() + search.query.builtins.nested_work();
        assert_eq!(search.query.nested as usize, actual - before);
        assert!(search.query.nested >= previous);
    }
    Ok(())
}

fn projected_tail(fixture: &mut Fixture, subject: TypeId) -> TestResult<TypeId> {
    for index in 0..fixture.facts.types.len() {
        let ty = TypeId(u32::try_from(index)?);
        if let Some(Kind::Projection(application)) = fixture.facts.types.kind(ty)
            && let Some(Kind::Parameter(parameter)) = fixture.facts.types.kind(application.subject)
            && fixture
                .facts
                .types
                .parameter(parameter)
                .is_some_and(|p| p.owner == fixture.projected)
        {
            return Ok(fixture.facts.types.intern(Kind::Projection(
                crate::types::projection::Application {
                    subject,
                    ..application
                },
            ))?);
        }
    }
    Err("P's associated field prototype is missing".into())
}

#[test]
fn session_normalization_forwards_its_nested_work_once() -> TestResult {
    let mut fixture = fixture()?;
    let environment = scoped(&fixture.facts)?;
    for depth in 1..=4 {
        for premised in [false, true] {
            let ty = chain(&mut fixture, depth, premised)?;
            let alias = projected_tail(&mut fixture, ty)?;
            let mut proof = proof(&fixture, &environment, premised)?;
            selection::take_nested_observation();
            assert_eq!(proof.normalize(&mut fixture.facts, alias)?, ty);
            let (_, tally) = selection::take_nested_observation();
            assert!(proof.nested_work() > 0);
            assert_eq!(proof.nested_work(), tally);
        }
    }
    Ok(())
}

#[test]
fn nested_saturation_is_a_sticky_error_and_hot_layouts_stay_fixed() -> TestResult {
    let mut context = Context::default();
    context.account_work(5)?;
    context.account_nested(7)?;
    assert_eq!((context.steps, context.take_nested()), (5, 7));
    assert_eq!(context.nested(), 0);
    assert_eq!(
        context.account_nested(usize::MAX),
        Err(crate::types::infer::Error::Budget)
    );
    assert_eq!(
        context.account_work(0),
        Err(crate::types::infer::Error::Budget)
    );
    let context = context.recycled();
    assert_eq!((context.steps, context.nested()), (0, 0));
    assert_eq!(size_of::<Context>(), 400);
    assert_eq!(size_of::<State>(), 808);
    assert_eq!(size_of::<Caller<'_>>(), 32);
    assert_eq!(size_of::<OwnedCaller>(), 32);
    assert_eq!(size_of::<crate::sema::projections::Cursor>(), 584);
    assert_eq!(
        size_of::<crate::sema::traits::selection::search::continuation::Query>(),
        696
    );
    Ok(())
}

#[test]
fn reuse_counts_and_work_observation() -> TestResult {
    let worker = std::thread::Builder::new().stack_size(16 * 1024 * 1024)
        .spawn(|| -> Result<(), String> {
            let mut fixture = fixture().map_err(|e| e.to_string())?;
            let environment = scoped(&fixture.facts).map_err(|e| e.to_string())?;
            for depth in [1, 2, 3, 4, 10, 13, 31] {
                for premised in [false, true] {
                    let before = crate::sema::nominals::traits::size_normalizations();
                    let row = observe(&mut fixture, &environment, depth, premised).map_err(|e| e.to_string())?;
                    let after = crate::sema::nominals::traits::size_normalizations();
                    let count = after[usize::from(premised)] - before[usize::from(premised)];
                    eprintln!("reuse depth={depth} premised={premised} result={:?} own={} nested={} tally={} level={} normalizations={count}",row.result,row.own,row.nested,row.tally,row.level);
                    assert_eq!(row.result, Ok(true));
                    assert_eq!(count, depth);
                    assert_eq!(row.nested, row.tally);
                }
            }
            Ok(())
        })?;
    worker
        .join()
        .map_err(|_| "reuse observer worker panicked")??;
    Ok(())
}

#[test]
fn completed_sizes_do_not_cross_premises_or_implementation_views() -> TestResult {
    use crate::sema::nominals::sizes;
    let mut fixture = fixture()?;
    let ty = chain(&mut fixture, 2, true)?;
    let source = premises(&[fixture.unsized_], &[])?;
    let equal = premises(&[fixture.unsized_], &[])?;
    let opposed = premises(&[], &[fixture.unsized_])?;
    for (target, availability, reused, expected) in [
        (source.clone(), Availability::Declarations, true, true),
        (equal, Availability::Declarations, false, true),
        (opposed, Availability::Declarations, false, false),
        (source.clone(), Availability::Validated, false, true),
    ] {
        let mut producer =
            Proof::for_selection(None, selection::Nesting::ROOT, Availability::Declarations);
        producer.assume_placeholders(&fixture.facts.types, source.clone())?;
        let physical = producer.physical_shape(&mut fixture.facts, ty)?;
        assert!(physical.sized && physical.well_formed);
        producer.record_physical_size(&fixture.facts, ty, physical)?;
        let learned = producer
            .take_sizes()
            .ok_or("missing completed size result")?;
        let mut query = sizes::Query::with_placeholders(None, Some(target.clone()));
        query.seed(*learned, availability)?;
        let mut proof = Proof::for_selection(None, selection::Nesting::ROOT, availability);
        proof.assume_placeholders(&fixture.facts.types, target)?;
        let mut visits = 0;
        let answer = query
            .demand(&mut fixture.facts, ty, |facts, ty| {
                visits += 1;
                proof.normalize(facts, ty)
            })?
            .final_sized()?;
        assert_eq!(answer, expected);
        assert_eq!(visits == 0, reused);
    }
    Ok(())
}

#[test]
fn source_reductions_accept_deep_generic_and_closed_fields() -> TestResult {
    for name in [
        "depth-1",
        "depth-4",
        "depth-12",
        "depth-16",
        "depth-31",
        "depth-32",
        "closed-12",
        "closed-31",
    ] {
        let source = crate::test_source::fixture(&format!(
            "pa31/tests/spec/nested-projection-work/{name}.rs"
        ))?;
        let report = crate::test_source::source_provider(&source, "nested_source_reuse")?;
        crate::pipeline::analyze_source(&report)?;
    }
    Ok(())
}

#[test]
fn unrelated_completed_shapes_do_not_multiply_snapshot_work() -> TestResult {
    let mut work = Vec::new();
    for warm in [false, true] {
        let mut fixture = fixture()?;
        let environment = scoped(&fixture.facts)?;
        let ty = chain(&mut fixture, 4, false)?;
        let mut proof = Proof::new(Some(environment));
        if warm {
            for length in 0..128 {
                let array = fixture.facts.types.intern(Kind::Array {
                    element: fixture.sized,
                    length,
                })?;
                let unrelated = nominal(&mut fixture.facts, fixture.projected, array)?;
                assert!(prove_sized(&mut proof, &mut fixture.facts, unrelated)?);
            }
        }
        let own = proof.own_work();
        let nested = proof.nested_work();
        let before = crate::sema::nominals::traits::size_normalizations();
        selection::take_nested_observation();
        assert!(prove_sized(&mut proof, &mut fixture.facts, ty)?);
        let (_, tally) = selection::take_nested_observation();
        assert_eq!(proof.nested_work() - nested, tally);
        assert_eq!(
            crate::sema::nominals::traits::size_normalizations()[0] - before[0],
            4
        );
        work.push(proof.own_work() - own + proof.nested_work() - nested);
    }
    assert!(work[1] <= work[0] + 10, "{work:?}");
    Ok(())
}
