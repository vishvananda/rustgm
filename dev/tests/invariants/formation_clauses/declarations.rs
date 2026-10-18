//! Declaration formation reaches actual nominal and scoped lifetime proof consumers.
use super::{
    Argument, BYTE, Bound, Clause, Declarations, Kind, Predicate, Proof, RegionId, Report,
    TestResult, TypeId, nominal, quantified, quantified_tests, tests,
};
use crate::types::parameters::environment::Environment;
use std::sync::Arc;

fn trait_clause(
    facts: &mut Declarations,
    report: &Report,
    subject: TypeId,
    arguments: &[Argument],
    count: u32,
) -> TestResult<Clause> {
    let arguments = facts.types.intern_arguments(arguments)?;
    let value = tests::predicate(facts, tests::definition(report, "B")?, subject, arguments)?;
    quantified(facts, count, value)
}

fn local_reference(facts: &mut Declarations, referent: TypeId) -> TestResult<TypeId> {
    Ok(facts.types.intern(Kind::Reference {
        mutable: false,
        region: RegionId::bound(0, 0)?,
        referent,
    })?)
}

#[test]
fn deferring_a_whole_quantified_subject_does_not_authorize_its_nominal_body_type() -> TestResult {
    let (report, mut facts) = tests::declarations("trait A{}struct N<T:A>(T);trait B<U>{}")?;
    let owner = tests::definition(&report, "N")?;
    let ty = nominal(&mut facts, owner, &[Argument::Type(BYTE)])?;
    let subject = local_reference(&mut facts, ty)?;
    let clause = trait_clause(
        &mut facts,
        &report,
        subject,
        &[Argument::Type(TypeId::UNIT)],
        1,
    )?;
    let mut proof = Proof::for_declaration(None, 0, 1);
    let formation = proof.form_trait(&mut facts, clause)?;
    assert_eq!(formation.subject(), None);
    assert_eq!(formation.declaration(), None);
    assert_eq!(
        proof.check(&mut facts, ty).err(),
        Some("nominal type argument must implement its declared trait")
    );
    Ok(())
}

#[test]
fn independent_arguments_still_require_nominal_formation_when_self_is_local() -> TestResult {
    for supplied in [false, true] {
        let implementation = if supplied { "impl A for u8{}" } else { "" };
        let (report, mut facts) = tests::declarations(&format!(
            "trait A{{}}struct N<T:A>(T);trait B<U>{{}}{implementation}"
        ))?;
        let owner = tests::definition(&report, "N")?;
        let ty = nominal(&mut facts, owner, &[Argument::Type(BYTE)])?;
        let subject = local_reference(&mut facts, BYTE)?;
        let clause = trait_clause(&mut facts, &report, subject, &[Argument::Type(ty)], 1)?;
        let mut proof = Proof::for_declaration(None, 0, 1);
        match proof.form_trait(&mut facts, clause) {
            Ok(formation) => {
                assert!(supplied);
                assert_eq!(formation.types(), &[ty]);
            }
            Err(error) => {
                assert!(!supplied);
                assert_eq!(
                    error,
                    "nominal type argument must implement its declared trait"
                );
            }
        }
    }
    Ok(())
}

#[test]
fn independent_trait_argument_requirements_need_an_actual_caller_or_implementation_proof()
-> TestResult {
    for supplied in [false, true] {
        let implementation = if supplied { "impl A for u8{}" } else { "" };
        let (report, mut facts) =
            tests::declarations(&format!("trait A{{}}trait B<U:A>{{}}{implementation}"))?;
        let subject = local_reference(&mut facts, BYTE)?;
        let clause = trait_clause(&mut facts, &report, subject, &[Argument::Type(BYTE)], 1)?;
        let mut proof = Proof::for_declaration(None, 0, 1);
        let result = proof.form_trait(&mut facts, clause);
        assert_eq!(result.is_ok(), supplied);
        if !supplied {
            assert_eq!(
                result.err(),
                Some("trait argument does not satisfy its declaration requirements")
            );
        }
    }
    Ok(())
}

#[test]
fn quantified_inherited_requirements_keep_their_universe_until_final_lifetime_proof() -> TestResult
{
    for (implementation, expected) in [
        ("impl<'a>A<'a>for u8{}", true),
        ("impl A<'static>for u8{}", false),
    ] {
        let (report, mut facts) = tests::declarations(&format!(
            "trait A<'a>{{}}trait B<U:A<'static>>{{}}{implementation}"
        ))?;
        let owner = tests::definition(&report, "B")?;
        let located = facts.generics.predicates[&owner]
            .iter()
            .copied()
            .find(|p| {
                matches!(
                    p.value,
                    Clause::Predicate(Predicate {
                        bound: Bound::Trait(_),
                        ..
                    })
                )
            })
            .ok_or("missing trait argument contract")?;
        let arguments = facts
            .types
            .intern_arguments(&[Argument::Lifetime(RegionId::bound(0, 0)?)])?;
        let body = tests::predicate(
            &mut facts,
            tests::definition(&report, "A")?,
            located.value.ordinary()?.ty,
            arguments,
        )?;
        let requirement = quantified(&mut facts, 1, body)?;
        facts
            .generics
            .traits
            .as_mut()
            .ok_or("missing trait contracts")?
            .complete_bindings(
                &facts.types,
                vec![(
                    crate::types::ItemId(owner.0),
                    vec![crate::sema::generics::Predicate {
                        value: requirement,
                        ..located
                    }],
                )],
            )?;
        let subject = local_reference(&mut facts, BYTE)?;
        let clause = trait_clause(&mut facts, &report, subject, &[Argument::Type(BYTE)], 1)?;
        assert!(Proof::new(None).form_trait(&mut facts, clause).is_err());
        let mut proof = Proof::for_declaration(None, 0, 1);
        let formation = proof.form_trait(&mut facts, clause)?;
        assert!(
            formation
                .required()
                .iter()
                .any(|c| matches!(c, Clause::ForAll(_)))
        );
        let evidence = proof
            .take_trait_proof()
            .ok_or("missing universal formation evidence")?;
        assert_eq!(evidence.scope_count(), 1);
        assert_eq!(quantified_tests::valid(&facts, &evidence, 0)?, expected);
    }
    Ok(())
}

#[test]
fn independent_captured_lifetime_requirements_are_checked_in_the_enclosing_owner() -> TestResult {
    for (implementation, expected) in [
        ("impl<'a>A<'a>for u8{}", true),
        ("impl A<'static>for u8{}", false),
    ] {
        let (report, mut facts) = tests::declarations(&format!(
            "trait A<'a>{{}}trait B<'a,U:A<'a>>{{}}{implementation}"
        ))?;
        let subject = local_reference(&mut facts, BYTE)?;
        let clause = trait_clause(
            &mut facts,
            &report,
            subject,
            &[
                Argument::Lifetime(RegionId::bound(1, 0)?),
                Argument::Type(BYTE),
            ],
            1,
        )?;
        let mut proof = Proof::for_declaration(None, 1, 1);
        let formation = proof.form_trait(&mut facts, clause)?;
        assert_eq!(formation.declaration(), None);
        let evidence = proof
            .take_trait_proof()
            .ok_or("missing captured formation evidence")?;
        assert_eq!(quantified_tests::valid(&facts, &evidence, 1)?, expected);
    }
    Ok(())
}

#[test]
fn formation_alone_cannot_satisfy_an_empty_quantified_closed_bound() -> TestResult {
    let (report, mut facts) = tests::declarations("trait B{}")?;
    let value = tests::predicate(
        &mut facts,
        tests::definition(&report, "B")?,
        BYTE,
        crate::types::GenericArgs::EMPTY,
    )?;
    let environment =
        Arc::new(Environment::new(&facts.types, &[])?.with_predicates(&facts.types, &[value])?);
    for count in [0, 1, 3] {
        let clause = quantified(&mut facts, count, value)?;
        let mut proof = Proof::for_declaration(Some(Arc::clone(&environment)), 0, 1);
        let formation = proof.form_trait(&mut facts, clause)?;
        assert_eq!(formation.declaration(), Some(value));
        assert_eq!(clause.requires_definition_proof(&facts.types)?, count == 0);
        assert!(proof.prove(&mut facts, value)?);
        assert!(!proof.prove_closed_trait(&mut facts, value)?);
    }
    Ok(())
}
