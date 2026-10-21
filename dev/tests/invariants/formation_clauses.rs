//! Nominal clauses retain their own binder while lifting only independent callable captures.
#[path = "formation_clauses/builtin_instances.rs"]
mod builtin_instances;
#[path = "formation_clauses/builtins.rs"]
mod builtins;
#[path = "formation_clauses/declarations.rs"]
mod declarations;
#[path = "formation_clauses/terms.rs"]
mod terms;
use super::{Declarations, quantified_tests, tests};
use crate::{
    resolve::{expansion::Report, model::DefId},
    sema::nominals::{Proof, Requirements, requirements},
    types::{
        ItemId, Kind, Primitive, RegionId, TypeId,
        arguments::Argument,
        bound_uses::captures::Captures,
        parameters::{
            clauses::{Clause, Quantified},
            predicates::{Bound, Predicate},
        },
    },
};
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
const BYTE: TypeId = TypeId::primitive(Primitive::U8);

fn quantified(facts: &mut Declarations, count: u32, predicate: Predicate) -> TestResult<Clause> {
    Ok(Clause::ForAll(facts.types.intern_quantified(
        Quantified {
            bound_regions: count,
            predicate,
        },
    )?))
}
fn install(
    facts: &mut Declarations,
    report: &Report,
    count: u32,
    regions: &[RegionId],
) -> TestResult<DefId> {
    let owner = tests::definition(report, "N")?;
    let old = facts.generics.predicates[&owner][0].value.ordinary()?;
    assert!(matches!(old.bound, Bound::Trait(_)));
    let arguments = facts.types.intern_arguments(
        &regions
            .iter()
            .copied()
            .map(Argument::Lifetime)
            .collect::<Vec<_>>(),
    )?;
    let predicate = tests::predicate(facts, tests::definition(report, "A")?, old.ty, arguments)?;
    let clause = quantified(facts, count, predicate)?;
    facts
        .generics
        .predicates
        .get_mut(&owner)
        .ok_or("missing nominal predicates")?[0]
        .value = clause;
    facts.generics.nominal_requirements.clear();
    Ok(owner)
}
fn nominal(facts: &mut Declarations, owner: DefId, args: &[Argument]) -> TestResult<TypeId> {
    let arguments = facts.types.intern_arguments(args)?;
    Ok(facts.types.intern(Kind::Adt(ItemId(owner.0), arguments))?)
}
fn callable(facts: &mut Declarations, parameters: &[TypeId], count: u32) -> TestResult<TypeId> {
    let parameters = facts.types.intern_list(parameters)?;
    Ok(facts.types.intern(Kind::Function {
        parameters,
        result: TypeId::UNIT,
        bound_regions: count,
        unsafe_: false,
        convention: crate::types::callable::Convention::RUST,
    })?)
}

#[test]
fn nominal_formation_keeps_universal_implementation_evidence_until_the_region_gate() -> TestResult {
    for (implementation, expected) in [
        ("impl<'a>A<'a>for u8{}", true),
        ("impl A<'static>for u8{}", false),
    ] {
        let (report, mut facts) = tests::declarations(&format!(
            "trait A<'a>{{}}struct N<T:A<'static>>(T);{implementation}"
        ))?;
        let owner = install(&mut facts, &report, 1, &[RegionId::bound(0, 0)?])?;
        let ty = nominal(&mut facts, owner, &[Argument::Type(BYTE)])?;
        assert!(Proof::new(None).check(&mut facts, ty).is_err());
        let mut proof = Proof::for_declaration(None, 0, 1);
        assert!(proof.check(&mut facts, ty)?.well_formed);
        let evidence = proof
            .take_trait_proof()
            .ok_or("missing nominal universal proof")?;
        assert_eq!(evidence.scope_count(), 1);
        assert_eq!(quantified_tests::valid(&facts, &evidence, 0)?, expected);
    }
    Ok(())
}

#[test]
fn a_clause_local_cannot_hide_an_independent_nominal_requirement_inside_a_callable() -> TestResult {
    let (report, mut facts) = tests::declarations("trait A<'a>{}struct N<T:A<'static>>(T);")?;
    let owner = install(&mut facts, &report, 1, &[RegionId::bound(0, 0)?])?;
    let ty = nominal(&mut facts, owner, &[Argument::Type(BYTE)])?;
    let function = callable(&mut facts, &[ty], 1)?;
    let mut checked = Requirements::default();
    let values = requirements(&mut facts, function, &mut checked)?;
    assert_eq!(
        values
            .iter()
            .filter(|value| matches!(value, Clause::ForAll(_)))
            .count(),
        1
    );
    let mut proof = Proof::for_declaration(None, 0, 1);
    assert!(proof.check(&mut facts, function).is_err());
    assert!(requirements(&mut facts, function, &mut checked)?.is_empty());
    Ok(())
}

#[test]
fn callable_local_requirements_defer_but_ancestor_captures_lift_with_their_quantifier() -> TestResult
{
    for (captured, count) in [(false, 0), (true, 0), (false, 1), (true, 1)] {
        let (report, mut facts) =
            tests::declarations("trait A<'a,'x>{}struct N<'x,T:A<'static,'x>>(&'x T);")?;
        let local = if count == 0 {
            RegionId::STATIC
        } else {
            RegionId::bound(0, 0)?
        };
        let owner = install(&mut facts, &report, count, &[local, RegionId::bound(1, 0)?])?;
        let ty = nominal(
            &mut facts,
            owner,
            &[
                Argument::Lifetime(RegionId::bound(u16::from(captured), 0)?),
                Argument::Type(BYTE),
            ],
        )?;
        let function = callable(&mut facts, &[ty], 1)?;
        let values = requirements(&mut facts, function, &mut Requirements::default())?;
        let clauses: Vec<_> = values
            .into_iter()
            .filter(|value| matches!(value, Clause::ForAll(_)))
            .collect();
        assert_eq!(clauses.len(), usize::from(captured));
        if captured {
            let Clause::ForAll(id) = clauses[0] else {
                return Err("missing lifted quantifier".into());
            };
            let value = facts.types.quantified(id).ok_or("missing lifted clause")?;
            assert_eq!(value.bound_regions, count);
            assert_eq!(value.predicate.ty, BYTE);
            let Bound::Trait(application) = value.predicate.bound else {
                return Err("missing lifted trait".into());
            };
            let args = facts
                .types
                .trait_application(application)
                .ok_or("missing lifted application")?
                .arguments;
            assert_eq!(
                facts.types.region_args(args),
                Some([local, RegionId::bound(1, 0)?].as_slice())
            );
        }
    }
    Ok(())
}

#[test]
fn capture_queries_keep_ordinary_and_quantified_application_caches_separate() -> TestResult {
    let (report, mut facts) = tests::declarations("trait A<'a>{}")?;
    let arguments = facts
        .types
        .intern_arguments(&[Argument::Lifetime(RegionId::bound(0, 0)?)])?;
    let predicate = tests::predicate(
        &mut facts,
        tests::definition(&report, "A")?,
        BYTE,
        arguments,
    )?;
    let clause = quantified(&mut facts, 1, predicate)?;
    for reverse in [false, true] {
        let mut captures = Captures::new(1);
        if reverse {
            assert!(!captures.clause(&facts.types, clause)?);
        }
        assert!(captures.predicate(&facts.types, predicate)?);
        assert!(!captures.clause(&facts.types, clause)?);
        let steps = captures.steps;
        assert!(captures.predicate(&facts.types, predicate)?);
        assert!(!captures.clause(&facts.types, clause)?);
        assert_eq!(captures.steps, steps);
    }
    Ok(())
}

#[test]
fn nominal_clause_lifting_shares_growing_captured_argument_tails() -> TestResult {
    let mut observations = Vec::new();
    for width in [16, 32, 64, 128, 256, 512] {
        let (report, mut facts) = tests::declarations("trait A<'a>{}struct N<T:A<'static>>(T);")?;
        let owner = install(&mut facts, &report, 1, &[RegionId::bound(0, 0)?])?;
        let mut tail = BYTE;
        let mut parameters = Vec::new();
        for _ in 0..width {
            tail = facts.types.intern(Kind::Reference {
                mutable: false,
                region: RegionId::bound(1, 0)?,
                referent: tail,
            })?;
            parameters.push(nominal(&mut facts, owner, &[Argument::Type(tail)])?);
        }
        let function = callable(&mut facts, &parameters, 1)?;
        let before = facts.types.len();
        let substitution_before = facts.generics.substitution_steps();
        let mut checked = Requirements::default();
        let values = requirements(&mut facts, function, &mut checked)?;
        assert_eq!(
            values
                .iter()
                .filter(|value| matches!(value, Clause::ForAll(_)))
                .count(),
            width
        );
        let collection_steps = checked.steps();
        let substitution_steps = facts.generics.substitution_steps() - substitution_before;
        let steps = collection_steps + substitution_steps;
        let growth = facts.types.len() - before;
        println!(
            "nominal-clause width={width} steps={steps} collect={collection_steps} substitute={substitution_steps} types={growth}"
        );
        observations.push((width, steps, growth));
        assert!(steps <= 128 * width + 128, "{width}: {steps}");
        assert!(growth <= 8 * width + 32, "{width}: {growth}");
        assert!(requirements(&mut facts, function, &mut checked)?.is_empty());
        assert_eq!(checked.steps(), collection_steps);
        assert_eq!(
            facts.generics.substitution_steps() - substitution_before,
            substitution_steps
        );
    }
    for pair in observations.windows(2) {
        assert!(pair[1].1 <= 2 * pair[0].1 + 128, "{pair:?}");
        assert!(pair[1].2 <= 2 * pair[0].2 + 32, "{pair:?}");
    }
    Ok(())
}

#[test]
fn quantified_outlives_is_a_nominal_obligation_not_an_omitted_ordinary_contract() -> TestResult {
    for (region, count, expected) in [
        (RegionId::STATIC, 0, true),
        (RegionId::bound(0, 0)?, 1, false),
    ] {
        let (report, mut facts) = tests::declarations("struct N<T:'static>(T);")?;
        let owner = tests::definition(&report, "N")?;
        let mut predicate = facts.generics.predicates[&owner][0].value.ordinary()?;
        assert!(matches!(predicate.bound, Bound::Outlives(_)));
        predicate.bound = Bound::Outlives(RegionId::bound(0, 0)?);
        let clause = quantified(&mut facts, 1, predicate)?;
        facts
            .generics
            .predicates
            .get_mut(&owner)
            .ok_or("missing nominal")?[0]
            .value = clause;
        facts.generics.nominal_requirements.clear();
        let bounds = crate::types::outlives::Bounds::infer_with_clauses(
            &mut facts.types,
            &facts.aggregates,
            &[],
            [(ItemId(owner.0), clause)],
        )?;
        assert_eq!(bounds.parameters(ItemId(owner.0)), Some([].as_slice()));
        assert_eq!(bounds.get(ItemId(owner.0)), Some([].as_slice()));
        let argument = facts.types.intern(Kind::Reference {
            mutable: false,
            region,
            referent: BYTE,
        })?;
        let ty = nominal(&mut facts, owner, &[Argument::Type(argument)])?;
        let clauses = requirements(&mut facts, ty, &mut Requirements::default())?;
        assert_eq!(
            clauses
                .iter()
                .filter(|clause| matches!(clause, Clause::ForAll(_)))
                .count(),
            1
        );
        let mut proof = Proof::for_declaration(None, count, count + 1);
        assert!(proof.check(&mut facts, ty)?.well_formed);
        let evidence = proof
            .take_trait_proof()
            .ok_or("missing quantified outlives evidence")?;
        assert_eq!(evidence.scope_count(), 1);
        assert_eq!(quantified_tests::valid(&facts, &evidence, count)?, expected);
    }
    Ok(())
}

#[test]
fn capture_queries_visit_whole_clause_subjects_results_and_outlives_regions() -> TestResult {
    let (_, mut facts) = tests::declarations("")?;
    for depth in [0, 1, 2] {
        let region = RegionId::bound(depth, 0)?;
        let ty = facts.types.intern(Kind::Reference {
            mutable: false,
            region,
            referent: BYTE,
        })?;
        let function_argument = facts.types.intern(Kind::Reference {
            mutable: false,
            region: RegionId::bound(depth + 1, 0)?,
            referent: BYTE,
        })?;
        let function = callable(&mut facts, &[function_argument], 1)?;
        for predicate in [
            Predicate {
                ty,
                bound: Bound::Copy,
            },
            Predicate {
                ty: BYTE,
                bound: Bound::ProjectionEqual(ty),
            },
            Predicate {
                ty: BYTE,
                bound: Bound::Outlives(region),
            },
            Predicate {
                ty: function,
                bound: Bound::Sized,
            },
        ] {
            let clause = quantified(&mut facts, 1, predicate)?;
            let mut captures = Captures::new(1);
            assert_eq!(captures.clause(&facts.types, clause)?, depth == 1);
        }
    }
    Ok(())
}
