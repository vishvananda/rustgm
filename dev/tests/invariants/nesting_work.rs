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
    let own = [[20, 38, 56, 74], [22, 38, 54, 70]];
    let total = [[79, 342, 999, 2442], [79, 343, 1004, 2459]];
    for depth in 1..=4 {
        for (index, premised) in [false, true].into_iter().enumerate() {
            let row = observe(&mut fixture, &environment, depth, premised)?;
            assert_eq!(row.result, Ok(true));
            assert_eq!(row.own, own[index][depth - 1]);
            assert_eq!(row.own + row.nested, total[index][depth - 1]);
            assert_eq!(row.nested, row.tally);
            assert_eq!(usize::from(row.level), 3 * depth - usize::from(premised));
        }
    }
    Ok(())
}

#[test]
fn nested_work_exhausts_before_the_exponential_tree_can_continue() -> TestResult {
    for premised in [false, true] {
        let mut fixture = fixture()?;
        let environment = scoped(&fixture.facts)?;
        let before = observe(&mut fixture, &environment, 12, premised)?;
        assert_eq!(before.result, Ok(true));
        assert_eq!(before.level, if premised { 35 } else { 36 });
        assert!(before.own + before.nested <= 1_000_000);
        let after = observe(&mut fixture, &environment, 13, premised)?;
        assert_eq!(after.result, Err("size normalization work budget exceeded"));
    }
    Ok(())
}

#[test]
fn depth_failure_reaches_the_root_without_stack_overflow() -> TestResult {
    let worker = std::thread::Builder::new()
        .stack_size(16 * 1024 * 1024)
        .spawn(|| -> Result<(), String> {
            let mut nesting = selection::Nesting::ROOT;
            for _ in 0..90 {
                nesting = nesting.nested();
            }
            for premised in [false, true] {
                let mut fixture = fixture().map_err(|error| error.to_string())?;
                let environment = scoped(&fixture.facts).map_err(|error| error.to_string())?;
                let row = observe_at(&mut fixture, &environment, 4, premised, nesting)
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
