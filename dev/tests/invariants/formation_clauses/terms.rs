//! Scoped terms recurse into nominal contracts and retain projection lifetime evidence.
use super::{
    Argument, BYTE, Bound, Clause, Declarations, Kind, Predicate, Proof, RegionId, Report,
    TestResult, TypeId, callable, nominal, quantified, quantified_tests, tests,
};
use crate::types::{GenericArgs, ItemId, parameters::environment::Environment, projection};
use std::sync::Arc;

fn reference(facts: &mut Declarations, ty: TypeId, depth: u16) -> TestResult<TypeId> {
    Ok(facts.types.intern(Kind::Reference {
        mutable: false,
        region: RegionId::bound(depth, 0)?,
        referent: ty,
    })?)
}
fn term(facts: &mut Declarations, ty: TypeId, equality: bool, count: u32) -> TestResult<Clause> {
    let value = if equality {
        Predicate {
            ty: BYTE,
            bound: Bound::ProjectionEqual(ty),
        }
    } else {
        Predicate {
            ty,
            bound: Bound::Outlives(RegionId::STATIC),
        }
    };
    quantified(facts, count, value)
}
pub(super) fn projected(
    facts: &mut Declarations,
    report: &Report,
    region: RegionId,
) -> TestResult<TypeId> {
    let arguments = facts
        .types
        .intern_arguments(&[Argument::Lifetime(region)])?;
    let value = tests::predicate(facts, tests::definition(report, "A")?, BYTE, arguments)?;
    let Bound::Trait(trait_application) = value.bound else {
        return Err("missing trait".into());
    };
    Ok(facts
        .types
        .intern(Kind::Projection(projection::Application {
            item: ItemId(tests::definition(report, "Out")?.0),
            subject: BYTE,
            trait_application,
            arguments: GenericArgs::EMPTY,
        }))?)
}

#[test]
fn recursive_terms_check_independent_nominal_requirements_beneath_local_references() -> TestResult {
    for equality in [false, true] {
        for supplied in [false, true] {
            let implementation = if supplied { "impl A for u8{}" } else { "" };
            for nested in [false, true] {
                let (report, mut facts) =
                    tests::declarations(&format!("trait A{{}}struct N<T:A>(T);{implementation}"))?;
                let mut ty = nominal(
                    &mut facts,
                    tests::definition(&report, "N")?,
                    &[Argument::Type(BYTE)],
                )?;
                if nested {
                    ty = reference(&mut facts, ty, 0)?;
                }
                let clause = term(&mut facts, ty, equality, 1)?;
                let result =
                    Proof::for_declaration(None, 0, 1).form_clause_term(&mut facts, clause);
                assert_eq!(result.is_ok(), supplied);
                if !supplied {
                    assert_eq!(
                        result.err(),
                        Some("nominal type argument must implement its declared trait")
                    );
                }
            }
        }
    }
    Ok(())
}

#[test]
fn deferred_local_nominal_requirements_cannot_authorize_the_body() -> TestResult {
    for equality in [false, true] {
        let (report, mut facts) = tests::declarations("trait A{}struct N<T:A>(T);")?;
        let reference = reference(&mut facts, BYTE, 0)?;
        let ty = nominal(
            &mut facts,
            tests::definition(&report, "N")?,
            &[Argument::Type(reference)],
        )?;
        let clause = term(&mut facts, ty, equality, 1)?;
        assert!(
            Proof::new(None)
                .form_clause_term(&mut facts, clause)
                .is_err()
        );
        let mut proof = Proof::for_declaration(None, 1, 7);
        assert!(proof.form_clause_term(&mut facts, clause)?.well_formed);
        assert_eq!(
            proof.check(&mut facts, ty).err(),
            Some("nominal type argument must implement its declared trait")
        );
    }
    Ok(())
}

#[test]
fn local_sized_requirements_defer_without_hiding_independent_malformed_children() -> TestResult {
    for (local, first_unsized, expected) in [
        (true, false, true),
        (false, false, false),
        (true, true, false),
    ] {
        let (_, mut facts) = tests::declarations("")?;
        let scalar = if local {
            reference(&mut facts, BYTE, 0)?
        } else {
            BYTE
        };
        let string = TypeId::primitive(crate::types::Primitive::Str);
        let elements = if first_unsized {
            [string, scalar]
        } else {
            [scalar, string]
        };
        let list = facts.types.intern_list(&elements)?;
        let tail = facts.types.intern(Kind::Tuple(list))?;
        let ty = facts.types.intern(Kind::Array {
            element: tail,
            length: 1,
        })?;
        for equality in [false, true] {
            let clause = term(&mut facts, ty, equality, 1)?;
            let mut proof = Proof::for_declaration(None, 1, 9);
            let shape = proof.form_clause_term(&mut facts, clause)?;
            assert_eq!(shape.well_formed, expected);
            assert!(!shape.sized);
            assert!(!proof.check(&mut facts, ty)?.well_formed);
        }
    }
    Ok(())
}

#[test]
fn opaque_projection_sizedness_is_deferred_only_in_formation_under_a_local_binder() -> TestResult {
    for supplied in [false, true] {
        let (report, mut facts) = tests::declarations("trait A<'a>{type Out:?Sized;}fn f<U>(){}")?;
        let owner = tests::definition(&report, "f")?;
        let Argument::Type(subject) = facts.generics.parameters[&owner][0].argument else {
            return Err("missing test parameter".into());
        };
        let value = projected(&mut facts, &report, RegionId::bound(0, 0)?)?;
        let Some(Kind::Projection(mut application)) = facts.types.kind(value) else {
            return Err("missing projection".into());
        };
        application.subject = subject;
        let ty = facts.types.intern(Kind::Projection(application))?;
        let premise = quantified(
            &mut facts,
            1,
            Predicate {
                ty: subject,
                bound: Bound::Trait(application.trait_application),
            },
        )?;
        let mut environment = facts.generics.parameter_environment(&facts.types, owner)?;
        if supplied {
            environment = environment.with_clauses(&facts.types, &[premise])?;
        }
        let ty = facts.types.intern(Kind::Array {
            element: ty,
            length: 1,
        })?;
        let clause = term(&mut facts, ty, false, 1)?;
        let mut proof = Proof::for_declaration(Some(Arc::new(environment)), 1, 9);
        let result = proof.form_clause_term(&mut facts, clause);
        assert_eq!(result.is_ok(), supplied);
        if supplied {
            let shape = result?;
            assert!(shape.well_formed);
            assert!(!shape.sized);
            assert!(!proof.check(&mut facts, ty)?.well_formed);
            let evidence = proof
                .take_trait_proof()
                .ok_or("missing opaque parent formation proof")?;
            let mut constraints = crate::types::relations::Constraints::new(
                &facts.types,
                &facts.lifetimes.variances,
                1,
                100_000,
            )?;
            let Kind::Parameter(parameter) =
                facts.types.kind(subject).ok_or("missing parameter")?
            else {
                return Err("invalid test parameter".into());
            };
            constraints.parameter(parameter)?;
            let mut queries = crate::types::outlives::Queries::new(
                &facts.types,
                &facts.lifetimes.nominal_requirements,
                &facts.lifetimes.callables,
            );
            evidence.require(&mut constraints, &mut queries)?;
            constraints.prove()?;
        }
    }
    Ok(())
}

#[test]
fn enclosing_nominal_debts_lift_across_empty_clause_and_nested_callable_binders() -> TestResult {
    for count in [0, 1, 3] {
        for nested in [false, true] {
            for supplied in [false, true] {
                let (report, mut facts) = tests::declarations("trait A{}struct N<T:A>(T);")?;
                let outer = reference(&mut facts, BYTE, 0)?;
                let value = tests::predicate(
                    &mut facts,
                    tests::definition(&report, "A")?,
                    outer,
                    GenericArgs::EMPTY,
                )?;
                let environment = Environment::new(&facts.types, &[])?;
                let environment = if supplied {
                    environment.with_predicates(&facts.types, &[value])?
                } else {
                    environment
                };
                let capture = reference(&mut facts, BYTE, 1 + u16::from(nested))?;
                let mut ty = nominal(
                    &mut facts,
                    tests::definition(&report, "N")?,
                    &[Argument::Type(capture)],
                )?;
                if nested {
                    ty = callable(&mut facts, &[ty], 0)?;
                }
                let clause = term(&mut facts, ty, true, count)?;
                let mut proof = Proof::for_declaration(Some(Arc::new(environment)), 1, 9);
                assert_eq!(proof.form_clause_term(&mut facts, clause).is_ok(), supplied);
            }
        }
    }
    Ok(())
}

#[test]
fn projection_formation_preserves_local_and_captured_universes_until_the_region_gate() -> TestResult
{
    for (implementation, expected) in [
        ("impl<'a>A<'a>for u8{type Out=&'a u8;}", true),
        ("impl A<'static>for u8{type Out=&'static u8;}", false),
    ] {
        for (depth, count, nested) in [(0, 1, false), (1, 1, false), (1, 0, false), (2, 0, true)] {
            let (report, mut facts) =
                tests::declarations(&format!("trait A<'a>{{type Out;}}{implementation}"))?;
            let mut ty = projected(&mut facts, &report, RegionId::bound(depth, 0)?)?;
            if nested {
                ty = callable(&mut facts, &[ty], 0)?;
            }
            let clause = term(&mut facts, ty, false, count)?;
            let mut proof = Proof::for_declaration(None, 1, 19);
            assert!(proof.form_clause_term(&mut facts, clause)?.well_formed);
            let evidence = proof
                .take_trait_proof()
                .ok_or("missing clause projection proof")?;
            assert_eq!(evidence.scope_count(), 1 + usize::from(nested));
            assert_eq!(quantified_tests::valid(&facts, &evidence, 1)?, expected);
        }
    }
    Ok(())
}

#[test]
fn projection_terms_reject_foreign_free_lifetimes_and_missing_outer_slots() -> TestResult {
    for region in [RegionId(8), RegionId(100), RegionId::bound(1, 1)?] {
        let (report, mut facts) =
            tests::declarations("trait A<'a>{type Out;}impl<'a>A<'a>for u8{type Out=&'a u8;}")?;
        let ty = projected(&mut facts, &report, region)?;
        let clause = term(&mut facts, ty, false, 1)?;
        assert!(
            Proof::for_declaration(None, 1, 9)
                .form_clause_term(&mut facts, clause)
                .is_err()
        );
    }
    Ok(())
}

#[test]
fn growing_projection_terms_share_opening_normalization_and_restoration_work() -> TestResult {
    let mut observations = Vec::new();
    for width in [16, 32, 64, 128, 256, 512, 1024] {
        let (report, mut facts) =
            tests::declarations("trait A<'a>{type Out;}impl<'a>A<'a>for u8{type Out=&'a u8;}")?;
        let mut ty = projected(&mut facts, &report, RegionId::bound(0, 0)?)?;
        let mut clauses = Vec::new();
        for _ in 0..width {
            ty = reference(&mut facts, ty, 1)?;
            clauses.push(term(&mut facts, ty, false, 1)?);
        }
        let before = facts.types.len();
        let mut proof = Proof::for_declaration(None, 1, 9);
        for &clause in &clauses {
            assert!(proof.form_clause_term(&mut facts, clause)?.well_formed);
        }
        let steps = proof.formation_steps();
        let growth = facts.types.len() - before;
        println!("clause-term width={width} steps={steps} types={growth}");
        assert!(steps <= 64 * width + 128, "{width}: {steps}");
        assert!(growth <= 4 * width + 32, "{width}: {growth}");
        for &clause in &clauses {
            assert!(proof.form_clause_term(&mut facts, clause)?.well_formed);
        }
        assert!(proof.formation_steps() - steps <= 2 * width);
        assert_eq!(facts.types.len() - before, growth);
        let evidence = proof
            .take_trait_proof()
            .ok_or("missing shared projection evidence")?;
        assert_eq!(evidence.scope_count(), 1);
        assert!(quantified_tests::valid(&facts, &evidence, 1)?);
        observations.push((steps, growth));
    }
    for pair in observations.windows(2) {
        assert!(pair[1].0 <= 2 * pair[0].0 + 64, "{pair:?}");
        assert!(pair[1].1 <= 2 * pair[0].1 + 16, "{pair:?}");
    }
    Ok(())
}
