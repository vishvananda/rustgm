//! Impl universals retain selected witnesses without exporting local inference or scopes.
#[path = "implementation_conditions/alternatives.rs"]
mod alternatives;
#[path = "implementation_conditions/clauses.rs"]
mod clauses;
#[path = "implementation_conditions/scaling.rs"]
mod scaling;
use super::{Decision, Declarations, Frame, Search, header_tests, quantified_tests, tests};
use crate::{
    resolve::{expansion::Report, model::DefId},
    sema::{
        nominals::traits::State,
        traits::{assumptions::Assumptions, implementations::Availability},
    },
    types::{
        GenericArgs, ItemId, Primitive, RegionId, TypeId,
        arguments::Argument,
        parameters::{
            clauses::{Clause, Quantified},
            predicates::{Bound, Predicate},
        },
    },
};
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
const BYTE: TypeId = TypeId::primitive(Primitive::U8);

fn owner(facts: &Declarations, item: DefId) -> TestResult<DefId> {
    facts
        .generics
        .implementations()
        .find_map(|header| {
            let Bound::Trait(application) = header.predicate.bound else {
                return None;
            };
            (facts.types.trait_application(application)?.item == ItemId(item.0))
                .then_some(header.owner)
        })
        .ok_or_else(|| "missing condition implementation".into())
}
fn quantify(
    facts: &mut Declarations,
    report: &Report,
    target: &str,
    premise: &str,
    count: u32,
    arguments: &[RegionId],
) -> TestResult {
    let owner = owner(facts, tests::definition(report, target)?)?;
    quantify_owner(
        facts,
        owner,
        tests::definition(report, premise)?,
        count,
        arguments,
    )
}
fn quantify_owner(
    facts: &mut Declarations,
    owner: DefId,
    premise: DefId,
    count: u32,
    arguments: &[RegionId],
) -> TestResult {
    let premise = ItemId(premise.0);
    let position = facts.generics.predicates[&owner]
        .iter()
        .position(|p| {
            let Clause::Predicate(Predicate {
                bound: Bound::Trait(application),
                ..
            }) = p.value
            else {
                return false;
            };
            facts
                .types
                .trait_application(application)
                .is_some_and(|application| application.item == premise)
        })
        .ok_or("missing ordinary condition to quantify")?;
    let ty = facts.generics.predicates[&owner][position]
        .value
        .ordinary()?
        .ty;
    let arguments = facts.types.intern_arguments(
        &arguments
            .iter()
            .copied()
            .map(Argument::Lifetime)
            .collect::<Vec<_>>(),
    )?;
    let predicate = tests::predicate(facts, DefId(premise.0), ty, arguments)?;
    let id = facts.types.intern_quantified(Quantified {
        bound_regions: count,
        predicate,
    })?;
    facts
        .generics
        .predicates
        .get_mut(&owner)
        .ok_or("missing impl conditions")?[position]
        .value = Clause::ForAll(id);
    Ok(())
}
fn root(facts: &mut Declarations, report: &Report, name: &str) -> TestResult<Predicate> {
    Ok(tests::predicate(
        facts,
        tests::definition(report, name)?,
        BYTE,
        GenericArgs::EMPTY,
    )?)
}

#[test]
fn universal_impl_conditions_require_every_lifetime() -> TestResult {
    for (implementation, expected) in [
        ("impl<'a>A<'a>for u8{}", true),
        ("impl A<'static>for u8{}", false),
        ("impl<'a:'static>A<'a>for u8{}", false),
        ("impl<'a:'a>A<'a>for u8{}", true),
    ] {
        let (report, mut facts) = tests::declarations(&format!(
            "trait A<'a>{{}}trait B{{}}impl<T:A<'static>>B for T{{}}{implementation}"
        ))?;
        quantify(&mut facts, &report, "B", "A", 1, &[RegionId::bound(0, 0)?])?;
        let root = root(&mut facts, &report, "B")?;
        let mut state = State::instance(&mut facts, 0)?;
        assert!(state.prove(&mut facts, root, true)?, "{implementation}");
        let proof = state.finish();
        assert_eq!(proof.scope_count(), 1);
        assert_eq!(
            quantified_tests::valid(&facts, &proof, 0)?,
            expected,
            "{implementation}"
        );
    }
    Ok(())
}

#[test]
fn implementation_slots_remain_independent_and_outer_captures_remain_outer() -> TestResult {
    for captured in [false, true] {
        for (implementation, expected) in [
            ("impl<'a,'b>A<'a,'b>for u8{}", true),
            ("impl<'a>A<'a,'a>for u8{}", false),
        ] {
            let declaration = if captured {
                "trait B<'x>{}impl<'x,T:A<'static,'x>>B<'x>for T{}"
            } else {
                "trait B{}impl<T:A<'static,'static>>B for T{}"
            };
            let (report, mut facts) =
                tests::declarations(&format!("trait A<'a,'b>{{}}{declaration}{implementation}"))?;
            let second = if captured {
                RegionId::bound(1, 0)?
            } else {
                RegionId::bound(0, 1)?
            };
            quantify(
                &mut facts,
                &report,
                "B",
                "A",
                if captured { 1 } else { 2 },
                &[RegionId::bound(0, 0)?, second],
            )?;
            let arguments = if captured {
                facts
                    .types
                    .intern_arguments(&[Argument::Lifetime(RegionId::bound(0, 0)?)])?
            } else {
                GenericArgs::EMPTY
            };
            let root = tests::predicate(
                &mut facts,
                tests::definition(&report, "B")?,
                BYTE,
                arguments,
            )?;
            let mut state = State::instance(&mut facts, u32::from(captured))?;
            assert!(state.prove(&mut facts, root, true)?);
            assert_eq!(
                quantified_tests::valid(&facts, &state.finish(), u32::from(captured))?,
                expected
            );
        }
    }
    Ok(())
}

#[test]
fn nested_impl_conditions_preserve_independent_universes() -> TestResult {
    for (implementation, expected) in [
        ("impl<'a,'b>A<'a,'b>for u8{}", true),
        ("impl<'a>A<'a,'a>for u8{}", false),
    ] {
        let (report, mut facts) = tests::declarations(&format!(
            "trait A<'a,'b>{{}}trait B<'a>{{}}trait C{{}}impl<'a,T:A<'a,'static>>B<'a>for T{{}}impl<T:B<'static>>C for T{{}}{implementation}"
        ))?;
        quantify(
            &mut facts,
            &report,
            "B",
            "A",
            1,
            &[RegionId::bound(1, 0)?, RegionId::bound(0, 0)?],
        )?;
        quantify(&mut facts, &report, "C", "B", 1, &[RegionId::bound(0, 0)?])?;
        let root = root(&mut facts, &report, "C")?;
        let mut state = State::instance(&mut facts, 0)?;
        assert!(state.prove(&mut facts, root, true)?);
        let proof = state.finish();
        assert_eq!(proof.scope_count(), 2);
        assert_eq!(quantified_tests::valid(&facts, &proof, 0)?, expected);
    }
    Ok(())
}

#[test]
fn ordinary_equalities_resolve_parent_types_before_universal_conditions() -> TestResult {
    for (implementation, expected) in [
        ("impl<'a>A<'a>for u8{}", true),
        ("impl A<'static>for u8{}", false),
    ] {
        let (report, mut facts) = tests::declarations(&format!(
            "trait Pick{{type Out;}}impl Pick for u8{{type Out=u8;}}trait A<'a>{{}}trait B{{}}impl<T>B for u8 where u8:Pick<Out=T>,T:A<'static>{{}}{implementation}"
        ))?;
        quantify(&mut facts, &report, "B", "A", 1, &[RegionId::bound(0, 0)?])?;
        let root = root(&mut facts, &report, "B")?;
        let mut state = State::instance(&mut facts, 0)?;
        assert!(state.prove(&mut facts, root, true)?);
        assert_eq!(
            quantified_tests::valid(&facts, &state.finish(), 0)?,
            expected
        );
    }
    Ok(())
}

#[test]
fn an_empty_quantifier_cannot_enter_the_flat_completed_cache() -> TestResult {
    let (report, mut facts) = tests::declarations(
        "trait A{}trait B{}trait C{}impl A for u8{}impl<T:A>B for T{}impl<T:B>C for T{}",
    )?;
    quantify(&mut facts, &report, "B", "A", 0, &[])?;
    let root = root(&mut facts, &report, "C")?;
    let (mut context, goal) = tests::instantiate(&mut facts, root)?;
    let assumptions = Assumptions::default();
    let mut search = Search::new(tests::caller(&assumptions, None), Availability::Validated);
    let first = Frame::new(&facts, &mut context, &goal, 1)?;
    let Decision::Proven(root) = search.run(&mut facts, first)? else {
        return Err("missing empty-binder proof".into());
    };
    assert!(search.query.completed.is_empty());
    let evidence = search.evidence(root)?;
    assert_eq!(evidence.proof.scope_count(), 1);
    assert!(evidence.proof.regions.is_empty());
    assert!(quantified_tests::valid(&facts, &evidence.proof, 0)?);
    Ok(())
}

#[test]
fn header_waits_preserve_the_universal_and_exclude_foreign_allocations() -> TestResult {
    let (report, mut facts) = tests::declarations(
        "trait A<'a>{}trait B{}impl<'a>A<'a>for u8{}impl<T:A<'static>>B for T{}",
    )?;
    quantify(&mut facts, &report, "B", "A", 1, &[RegionId::bound(0, 0)?])?;
    let root = root(&mut facts, &report, "B")?;
    let (mut context, goal) = tests::instantiate(&mut facts, root)?;
    let assumptions = Assumptions::default();
    let mut delayed = header_tests::delay(&mut facts, &[tests::definition(&report, "A")?])?;
    let mut search = Search::new(
        tests::caller(&assumptions, None),
        Availability::Declarations,
    );
    let first = Frame::new(&facts, &mut context, &goal, 1)?;
    let super::continuation::Progress::Pending(crate::sema::traits::selection::Dependency::Header(
        owner,
    )) = search.begin(&mut facts, first)?
    else {
        return Err("missing universal header wait".into());
    };
    header_tests::stable_poll(&mut search, &mut facts, owner)?;
    let frontier = search.query.allocations.end();
    assert!(frontier > 1);
    search.resume_at(frontier + 3)?;
    header_tests::publish(&mut facts, delayed.pop().ok_or("missing delayed header")?)?;
    let super::continuation::Progress::Ready(Decision::Proven(root)) = search.poll(&mut facts)?
    else {
        return Err("universal condition did not resume".into());
    };
    let evidence = search.evidence(root)?;
    assert_eq!(evidence.proof.scope_count(), 1);
    for id in frontier..frontier + 3 {
        assert!(!evidence.proof.owns_region(RegionId(id)));
    }
    assert!(quantified_tests::valid(&facts, &evidence.proof, 0)?);
    Ok(())
}
