//! Impl proof scopes retain caller assumptions, equality results and type-outlives debts.
use super::{BYTE, State, TestResult, owner, quantified_tests, quantify, tests};
use crate::types::{
    GenericArgs, ItemId, Kind, RegionId,
    arguments::Argument,
    parameters::{
        Parameter, ParameterKind,
        clauses::{Clause, Quantified},
        environment::{Abilities, Environment},
        predicates::{Bound, Predicate},
    },
};
use std::sync::Arc;

#[test]
fn a_universal_caller_assumption_proves_an_impl_condition_but_an_ordinary_one_cannot() -> TestResult
{
    for universal in [false, true] {
        let (report, mut facts) =
            tests::declarations("trait A<'a>{}trait B{}impl<T:A<'static>>B for T{}")?;
        quantify(&mut facts, &report, "B", "A", 1, &[RegionId::bound(0, 0)?])?;
        let parameter = facts.types.intern_parameter(Parameter {
            owner: ItemId(1_000_000),
            index: 0,
            kind: ParameterKind::Type { sized: true },
        })?;
        let ty = facts.types.intern(Kind::Parameter(parameter))?;
        let region = if universal {
            RegionId::bound(0, 0)?
        } else {
            RegionId::STATIC
        };
        let arguments = facts
            .types
            .intern_arguments(&[Argument::Lifetime(region)])?;
        let predicate =
            tests::predicate(&mut facts, tests::definition(&report, "A")?, ty, arguments)?;
        let clause = if universal {
            Clause::ForAll(facts.types.intern_quantified(Quantified {
                bound_regions: 1,
                predicate,
            })?)
        } else {
            predicate.into()
        };
        let env = Environment::new(&facts.types, &[(parameter, Abilities::default())])?
            .with_clauses(&facts.types, &[clause])?;
        let root = tests::predicate(
            &mut facts,
            tests::definition(&report, "B")?,
            ty,
            GenericArgs::EMPTY,
        )?;
        let mut state = State::new(&mut facts, Some(Arc::new(env)), Some((0, 1)))?;
        assert!(state.prove(&mut facts, root, false)?);
        let proof = state.finish();
        assert_eq!(proof.scope_count(), 1);
        assert_eq!(quantified_tests::valid(&facts, &proof, 0)?, universal);
    }
    Ok(())
}

#[test]
fn associated_equalities_keep_the_universal_result_and_input_correlated() -> TestResult {
    for (result, expected) in [("&'a u8", true), ("&'static u8", false)] {
        let (report, mut facts) = tests::declarations(&format!(
            "trait A<'a>{{type Out;}}trait B{{}}impl<'a>A<'a>for u8{{type Out={result};}}impl<T:A<'static,Out=&'static u8>>B for T{{}}"
        ))?;
        quantify(&mut facts, &report, "B", "A", 1, &[RegionId::bound(0, 0)?])?;
        let owner = owner(&facts, tests::definition(&report, "B")?)?;
        let index = facts.generics.predicates[&owner]
            .iter()
            .position(|p| {
                matches!(
                    p.value,
                    Clause::Predicate(Predicate {
                        bound: Bound::ProjectionEqual(_),
                        ..
                    })
                )
            })
            .ok_or("missing impl equality")?;
        let old = facts.generics.predicates[&owner][index].value.ordinary()?;
        let Some(Kind::Projection(mut application)) = facts.types.kind(old.ty) else {
            return Err("missing projected condition".into());
        };
        let arguments = facts
            .types
            .intern_arguments(&[Argument::Lifetime(RegionId::bound(0, 0)?)])?;
        let Bound::Trait(trait_application) = tests::predicate(
            &mut facts,
            tests::definition(&report, "A")?,
            application.subject,
            arguments,
        )?
        .bound
        else {
            return Err("missing parent trait".into());
        };
        application.trait_application = trait_application;
        let ty = facts.types.intern(Kind::Projection(application))?;
        let value = facts.types.intern(Kind::Reference {
            mutable: false,
            region: RegionId::bound(0, 0)?,
            referent: BYTE,
        })?;
        let predicate = Predicate {
            ty,
            bound: Bound::ProjectionEqual(value),
        };
        let id = facts.types.intern_quantified(Quantified {
            bound_regions: 1,
            predicate,
        })?;
        facts
            .generics
            .predicates
            .get_mut(&owner)
            .ok_or("missing impl")?[index]
            .value = Clause::ForAll(id);
        let root = super::root(&mut facts, &report, "B")?;
        let mut state = State::instance(&mut facts, 0)?;
        assert!(state.prove(&mut facts, root, true)?);
        let proof = state.finish();
        assert_eq!(proof.scope_count(), 1);
        assert_eq!(quantified_tests::valid(&facts, &proof, 0)?, expected);
    }
    Ok(())
}

#[test]
fn universal_type_outlives_conditions_keep_captured_inputs_in_the_outer_scope() -> TestResult {
    for (region, count, expected) in [
        (RegionId::STATIC, 0, true),
        (RegionId::bound(0, 0)?, 1, false),
    ] {
        let (report, mut facts) = tests::declarations("trait B{}impl<T:'static>B for T{}")?;
        let item = tests::definition(&report, "B")?;
        let owner = owner(&facts, item)?;
        let index = facts.generics.predicates[&owner]
            .iter()
            .position(|p| {
                matches!(
                    p.value,
                    Clause::Predicate(Predicate {
                        bound: Bound::Outlives(_),
                        ..
                    })
                )
            })
            .ok_or("missing impl outlives requirement")?;
        let mut predicate = facts.generics.predicates[&owner][index].value.ordinary()?;
        predicate.bound = Bound::Outlives(RegionId::bound(0, 0)?);
        let id = facts.types.intern_quantified(Quantified {
            bound_regions: 1,
            predicate,
        })?;
        facts
            .generics
            .predicates
            .get_mut(&owner)
            .ok_or("missing impl")?[index]
            .value = Clause::ForAll(id);
        let ty = facts.types.intern(Kind::Reference {
            mutable: false,
            region,
            referent: BYTE,
        })?;
        let root = tests::predicate(&mut facts, item, ty, GenericArgs::EMPTY)?;
        let mut state = State::instance(&mut facts, count)?;
        assert!(state.prove(&mut facts, root, true)?);
        let proof = state.finish();
        assert_eq!(proof.scope_count(), 1);
        assert_eq!(quantified_tests::valid(&facts, &proof, count)?, expected);
    }
    Ok(())
}
