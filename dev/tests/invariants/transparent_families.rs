//! Family input obligations stay distinct from promises about the projected result.
use super::*;
use crate::types::Kind as Type;

fn prepared(report: &Report) -> TestResult<Checker<'_>> {
    let mut checker = Checker::new(report)?;
    checker.reserve_member_parents()?;
    checker.prepare_trait_owners(&mut 0, 1_000_000)?;
    checker.prepare_associated_types(&mut 0, 1_000_000)?;
    if let Some(contracts) = &mut checker.facts.generics.traits {
        contracts.complete_results();
    }
    crate::sema::traits::results::prepare_outlives(&mut checker.facts)?;
    Ok(checker)
}

fn source(case: &str) -> TestResult<String> {
    Ok(crate::test_source::fixture(&format!(
        "pa31/tests/spec/trait-aliases/family-inputs/{case}.rs"
    ))?)
}

#[test]
fn alias_family_inputs_preserve_every_clause_without_result_promise_leakage() -> TestResult {
    let report = report(&source("accepted-input")?)?;
    let checker = prepared(&report)?;
    let item = definition(&report, "Item", DefKind::TypeAlias)?;
    let family = checker.facts.associated.families.get(&item).ok_or("actual family")?;
    let required = family.requirements.as_ref().ok_or("completed family inputs")?;
    assert_eq!(required.len(), 3);
    let input = checker.facts.generics.parameters[&item][0].argument;
    let Argument::Type(input) = input else { return Err("actual type input".into()); };
    let mut trait_ = 0;
    let mut equality = 0;
    let mut copies = 0;
    for clause in required {
        let predicate = clause.ordinary()?;
        match predicate.bound {
            Bound::Trait(_) => { assert_eq!(predicate.ty, input); trait_ += 1; }
            Bound::ProjectionEqual(result) => {
                assert!(matches!(checker.facts.types.kind(predicate.ty), Some(Type::Projection(p)) if p.subject == input));
                assert_eq!(result, TypeId::primitive(Primitive::U8));
                equality += 1;
            }
            Bound::Copy => { assert_eq!(predicate.ty, input); copies += 1; }
            _ => return Err("unexpected alias input clause".into()),
        }
    }
    assert_eq!((trait_, equality, copies), (1, 1, 1));
    let result = checker.facts.associated.type_contract(item).ok_or("actual result contract")?;
    assert_eq!(result.bounds.len(), 1);
    assert!(matches!(result.bounds[0], Clause::Predicate(p) if p.ty == result.projection && p.bound == Bound::Clone));
    assert!(checker.reject_alias_execution().is_err());
    Ok(())
}

#[test]
fn alias_family_input_copy_is_proved_at_the_real_projection_use() -> TestResult {
    for (name, accepted) in [("accepted-input", true), ("missing-copy", false)] {
        let report = report(&source(name)?)?;
        let mut checker = prepared(&report)?;
        let consumer = definition(&report, "consumer", DefKind::Function)?;
        checker.work.push(super::super::super::Work::Definition(consumer, false));
        let result = checker.drain(&mut 0, 1_000_000).and_then(|()| checker.validate_signatures());
        assert_eq!(result.is_ok(), accepted, "{name}: {result:?}");
        if let Err(error) = result {
            assert!(error.message.contains("trait obligation"), "{name}: {error}");
        }
    }
    Ok(())
}

#[test]
fn alias_family_result_expansion_does_not_extend_the_input_prefix() -> TestResult {
    let report = report(&source("alias-result")?)?;
    let checker = prepared(&report)?;
    let item = definition(&report, "Item", DefKind::TypeAlias)?;
    let family = checker.facts.associated.families.get(&item).ok_or("actual family")?;
    let required = family.requirements.as_ref().ok_or("completed input prefix")?;
    assert_eq!(required.len(), 3);
    let result = checker.facts.associated.type_contract(item).ok_or("actual result")?;
    assert_eq!(result.bounds.len(), 3);
    assert!(result.bounds.iter().any(|clause| matches!(clause, Clause::Predicate(p) if p.ty == result.projection && p.bound == Bound::Copy)));
    assert!(!required.iter().any(|clause| matches!(clause, Clause::Predicate(p) if p.ty == result.projection)));
    Ok(())
}
