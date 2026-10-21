//! Universal goals are accepted only after their scoped lifetime evidence is proved.
use super::{Declarations, tests};
use crate::{
    sema::nominals::traits::State,
    types::{
        GenericArgs, ItemId, Kind, Primitive, RegionId, TypeId,
        arguments::Argument,
        lifetime_proof::Obligations,
        outlives::Queries,
        parameters::{
            Parameter, ParameterKind,
            clauses::{Clause, Quantified},
            environment::{Abilities, Environment},
            predicates::{Bound, Predicate},
        },
        relations::Constraints,
    },
};
use std::{fmt::Write as _, sync::Arc};
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
const BYTE: TypeId = TypeId::primitive(Primitive::U8);

fn clause(facts: &mut Declarations, count: u32, predicate: Predicate) -> TestResult<Clause> {
    Ok(Clause::ForAll(facts.types.intern_quantified(
        Quantified {
            bound_regions: count,
            predicate,
        },
    )?))
}
pub(super) fn valid(facts: &Declarations, proof: &Obligations, outer: u32) -> TestResult<bool> {
    let mut constraints =
        Constraints::new(&facts.types, &facts.lifetimes.variances, outer, 100_000)?;
    let mut queries = Queries::new(
        &facts.types,
        &facts.lifetimes.nominal_requirements,
        &facts.lifetimes.callables,
    );
    proof.require(&mut constraints, &mut queries)?;
    match constraints.prove() {
        Ok(_) => Ok(true),
        Err(
            crate::types::region_graph::Error::Unsatisfied { .. }
            | crate::types::region_graph::Error::Verification { .. },
        ) => Ok(false),
        Err(error) => Err(error.into()),
    }
}

#[test]
fn static_only_and_restricted_implementations_do_not_prove_a_universal_goal() -> TestResult {
    for (implementation, expected) in [
        ("impl<'a>A<'a>for u8{}", true),
        ("impl A<'static>for u8{}", false),
        ("impl<'a:'static>A<'a>for u8{}", false),
        ("impl<'a:'a>A<'a>for u8{}", true),
    ] {
        let (report, mut facts) = tests::declarations(&format!("trait A<'a>{{}}{implementation}"))?;
        let arguments = facts
            .types
            .intern_arguments(&[Argument::Lifetime(RegionId::bound(0, 0)?)])?;
        let predicate = tests::predicate(
            &mut facts,
            tests::definition(&report, "A")?,
            BYTE,
            arguments,
        )?;
        let clause = clause(&mut facts, 1, predicate)?;
        let mut state = State::instance(&mut facts, 0)?;
        assert!(
            state.prove_clause(&mut facts, clause, true)?,
            "{implementation}"
        );
        assert!(state.prove_clause(&mut facts, clause, true)?);
        let proof = state.finish();
        assert_eq!(proof.scope_count(), 1);
        assert_eq!(valid(&facts, &proof, 0)?, expected, "{implementation}");
    }
    Ok(())
}

#[test]
fn independent_local_slots_and_enclosing_captures_do_not_collapse() -> TestResult {
    for (implementation, captured, expected) in [
        ("impl<'a,'b>A<'a,'b>for u8{}", false, true),
        ("impl<'a>A<'a,'a>for u8{}", false, false),
        ("impl<'a,'b>A<'a,'b>for u8{}", true, true),
        ("impl<'a>A<'a,'a>for u8{}", true, false),
    ] {
        let (report, mut facts) =
            tests::declarations(&format!("trait A<'a,'b>{{}}{implementation}"))?;
        let second = if captured {
            RegionId::bound(1, 0)?
        } else {
            RegionId::bound(0, 1)?
        };
        let arguments = facts.types.intern_arguments(&[
            Argument::Lifetime(RegionId::bound(0, 0)?),
            Argument::Lifetime(second),
        ])?;
        let predicate = tests::predicate(
            &mut facts,
            tests::definition(&report, "A")?,
            BYTE,
            arguments,
        )?;
        let clause = clause(&mut facts, if captured { 1 } else { 2 }, predicate)?;
        let outer = u32::from(captured);
        let mut state = State::instance(&mut facts, outer)?;
        assert!(state.prove_clause(&mut facts, clause, true)?);
        let proof = state.finish();
        assert_eq!(
            valid(&facts, &proof, outer)?,
            expected,
            "{implementation}; captured={captured}"
        );
    }
    Ok(())
}

#[test]
fn empty_quantifiers_keep_the_enclosing_binder_and_canonical_subject_scope() -> TestResult {
    let (report, mut facts) = tests::declarations("trait A<'a>{}impl<'a>A<'a>for u8{}")?;
    let arguments = facts
        .types
        .intern_arguments(&[Argument::Lifetime(RegionId::bound(1, 0)?)])?;
    let predicate = tests::predicate(
        &mut facts,
        tests::definition(&report, "A")?,
        BYTE,
        arguments,
    )?;
    let clause = clause(&mut facts, 0, predicate)?;
    let mut state = State::instance(&mut facts, 1)?;
    assert!(state.prove_clause(&mut facts, clause, true)?);
    let proof = state.finish();
    assert_eq!(proof.scope_count(), 1);
    assert!(valid(&facts, &proof, 1)?);
    Ok(())
}

fn parameter(
    facts: &mut Declarations,
) -> TestResult<(TypeId, crate::types::parameters::ParameterId)> {
    let id = facts.types.intern_parameter(Parameter {
        owner: ItemId(1_000_000),
        index: 0,
        kind: ParameterKind::Type { sized: true },
    })?;
    Ok((facts.types.intern(Kind::Parameter(id))?, id))
}

#[test]
fn an_ordinary_caller_premise_cannot_be_reused_as_universal_evidence() -> TestResult {
    let (report, mut facts) = tests::declarations("trait A<'a>{}")?;
    let (ty, id) = parameter(&mut facts)?;
    let arguments = facts
        .types
        .intern_arguments(&[Argument::Lifetime(RegionId::bound(0, 0)?)])?;
    let predicate = tests::predicate(&mut facts, tests::definition(&report, "A")?, ty, arguments)?;
    let environment = Environment::new(&facts.types, &[(id, Abilities::default())])?
        .with_predicates(&facts.types, &[predicate])?;
    // The same canonical slot denotes the outer lifetime in the environment and
    // the local lifetime in the new whole-predicate binder.
    let clause = clause(&mut facts, 1, predicate)?;
    let mut state = State::new(&mut facts, Some(Arc::new(environment)), Some((1, 1)))?;
    assert!(state.prove_clause(&mut facts, clause, false)?);
    assert!(!valid(&facts, &state.finish(), 1)?);
    Ok(())
}

#[test]
fn completed_queries_do_not_grant_another_mode_or_caller_permission() -> TestResult {
    let (report, mut facts) = tests::declarations("trait A{}")?;
    let (ty, id) = parameter(&mut facts)?;
    let predicate = tests::predicate(
        &mut facts,
        tests::definition(&report, "A")?,
        ty,
        GenericArgs::EMPTY,
    )?;
    let environment = Environment::new(&facts.types, &[(id, Abilities::default())])?
        .with_predicates(&facts.types, &[predicate])?;
    let clause = clause(&mut facts, 0, predicate)?;
    let mut state = State::new(&mut facts, Some(Arc::new(environment)), Some((0, 1)))?;
    assert!(state.prove_clause(&mut facts, clause, false)?);
    assert!(state.prove_clause(&mut facts, clause, false)?);
    assert!(!state.prove_clause(&mut facts, clause, true)?);
    let proof = state.finish();
    assert_eq!(proof.scope_count(), 1);
    assert!(valid(&facts, &proof, 0)?);
    let mut other = State::instance(&mut facts, 0)?;
    assert!(!other.prove_clause(&mut facts, clause, false)?);
    assert_eq!(other.finish().scope_count(), 0);
    Ok(())
}

#[test]
fn repeated_universal_requirements_share_one_retained_implementation_proof() -> TestResult {
    let mut source = String::from("trait A0<'a>{}impl<'a>A0<'a>for u8{}");
    for index in 1..=16 {
        write!(
            source,
            "trait A{index}<'a>{{}}impl<'a,T:A{}<'a>>A{index}<'a>for T{{}}",
            index - 1
        )?;
    }
    let (report, mut facts) = tests::declarations(&source)?;
    let arguments = facts
        .types
        .intern_arguments(&[Argument::Lifetime(RegionId::bound(0, 0)?)])?;
    let predicate = tests::predicate(
        &mut facts,
        tests::definition(&report, "A16")?,
        BYTE,
        arguments,
    )?;
    let clause = clause(&mut facts, 1, predicate)?;
    let mut state = State::instance(&mut facts, 0)?;
    assert!(state.prove_clause(&mut facts, clause, true)?);
    let types = facts.types.len();
    for _ in 0..512 {
        assert!(state.prove_clause(&mut facts, clause, true)?);
    }
    assert_eq!(facts.types.len(), types);
    let proof = state.finish();
    assert_eq!(proof.scope_count(), 1);
    assert!(proof.regions.end < 40, "{:?}", proof.regions);
    assert!(valid(&facts, &proof, 0)?);
    Ok(())
}

#[test]
fn universal_outlives_requirements_preserve_their_type_and_region_scope() -> TestResult {
    for (region, outer, expected) in [
        (RegionId::STATIC, 0, true),
        (RegionId::bound(1, 0)?, 1, false),
    ] {
        let (_, mut facts) = tests::declarations("")?;
        let ty = facts.types.intern(Kind::Reference {
            mutable: false,
            region,
            referent: BYTE,
        })?;
        let clause = clause(
            &mut facts,
            1,
            Predicate {
                ty,
                bound: Bound::Outlives(RegionId::bound(0, 0)?),
            },
        )?;
        let mut state = State::instance(&mut facts, outer)?;
        assert!(state.prove_clause(&mut facts, clause, true)?);
        assert_eq!(valid(&facts, &state.finish(), outer)?, expected);
    }
    Ok(())
}

fn reference(facts: &mut Declarations, region: RegionId) -> TestResult<TypeId> {
    Ok(facts.types.intern(Kind::Reference {
        mutable: false,
        region,
        referent: BYTE,
    })?)
}

#[test]
fn a_quantifier_correlates_the_subject_with_all_trait_arguments() -> TestResult {
    for (implementation, independent, expected) in [
        ("impl<'a>A<'a>for &'a u8{}", false, true),
        ("impl<'a>A<'a>for &'static u8{}", false, false),
        ("impl<'a>A<'a>for &'a u8{}", true, false),
        ("impl<'a,'b>A<'a>for &'b u8{}", true, true),
    ] {
        let (report, mut facts) = tests::declarations(&format!("trait A<'a>{{}}{implementation}"))?;
        let subject = reference(&mut facts, RegionId::bound(0, 0)?)?;
        let arguments = facts
            .types
            .intern_arguments(&[Argument::Lifetime(RegionId::bound(
                0,
                u16::from(independent),
            )?)])?;
        let predicate = tests::predicate(
            &mut facts,
            tests::definition(&report, "A")?,
            subject,
            arguments,
        )?;
        let clause = clause(&mut facts, 1 + u32::from(independent), predicate)?;
        let mut state = State::instance(&mut facts, 0)?;
        assert!(state.prove_clause(&mut facts, clause, true)?);
        assert_eq!(
            valid(&facts, &state.finish(), 0)?,
            expected,
            "{implementation}"
        );
    }
    Ok(())
}

#[test]
fn associated_equality_results_retain_the_quantified_input_scope() -> TestResult {
    for (result, captured, expected) in [
        ("&'a u8", false, true),
        ("&'static u8", false, false),
        ("&'a u8", true, false),
    ] {
        let (report, mut facts) = tests::declarations(&format!(
            "trait A<'a>{{type Out;}}impl<'a>A<'a>for u8{{type Out={result};}}"
        ))?;
        let arguments = facts
            .types
            .intern_arguments(&[Argument::Lifetime(RegionId::bound(0, 0)?)])?;
        let Bound::Trait(trait_application) = tests::predicate(
            &mut facts,
            tests::definition(&report, "A")?,
            BYTE,
            arguments,
        )?
        .bound
        else {
            return Err("missing equality parent trait".into());
        };
        let ty = facts
            .types
            .intern(Kind::Projection(crate::types::projection::Application {
                item: ItemId(tests::definition(&report, "Out")?.0),
                subject: BYTE,
                trait_application,
                arguments: GenericArgs::EMPTY,
            }))?;
        let result = reference(&mut facts, RegionId::bound(u16::from(captured), 0)?)?;
        let clause = clause(
            &mut facts,
            1,
            Predicate {
                ty,
                bound: Bound::ProjectionEqual(result),
            },
        )?;
        let outer = u32::from(captured);
        let mut state = State::instance(&mut facts, outer)?;
        assert!(state.prove_clause(&mut facts, clause, true)?);
        assert!(state.prove_clause(&mut facts, clause, true)?);
        let proof = state.finish();
        assert_eq!(proof.scope_count(), 1);
        assert_eq!(valid(&facts, &proof, outer)?, expected);
    }
    Ok(())
}

#[test]
fn nested_callable_locals_remain_distinct_from_the_quantified_capture() -> TestResult {
    for (capture, expected) in [("'a", true), ("'static", false)] {
        let (report, mut facts) = tests::declarations(&format!(
            "trait A<'a,T>{{}}impl<'a>A<'a,for<'b>fn(&'b u8,&{capture} u8)>for u8{{}}"
        ))?;
        let local = reference(&mut facts, RegionId::bound(0, 0)?)?;
        let captured = reference(&mut facts, RegionId::bound(1, 0)?)?;
        let parameters = facts.types.intern_list(&[local, captured])?;
        let function = facts.types.intern(Kind::Function {
            parameters,
            result: TypeId::UNIT,
            bound_regions: 1,
            unsafe_: false,
            convention: crate::types::callable::Convention::RUST,
        })?;
        let arguments = facts.types.intern_arguments(&[
            Argument::Lifetime(RegionId::bound(0, 0)?),
            Argument::Type(function),
        ])?;
        let predicate = tests::predicate(
            &mut facts,
            tests::definition(&report, "A")?,
            BYTE,
            arguments,
        )?;
        let clause = clause(&mut facts, 1, predicate)?;
        let mut state = State::instance(&mut facts, 0)?;
        assert!(state.prove_clause(&mut facts, clause, true)?);
        assert_eq!(valid(&facts, &state.finish(), 0)?, expected, "{capture}");
    }
    Ok(())
}
