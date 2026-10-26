//! Canonical alias clauses name real members without granting a trait identity.
use super::*;
use crate::types::Kind as Type;

fn source(name: &str) -> TestResult<String> {
    Ok(crate::test_source::fixture(&format!(
        "pa31/tests/spec/trait-aliases/naming-v1/{name}.rs"
    ))?)
}
fn signature(checker: &mut Checker<'_>) -> TestResult<DefId> {
    let consumer = definition(checker.report, "consumer", DefKind::Function)?;
    checker.work.push(super::super::super::Work::Definition(consumer, false));
    checker.drain(&mut 0, 1_000_000)?;
    Ok(consumer)
}
fn byte_signature(checker: &Checker<'_>, consumer: DefId) -> TestResult {
    let function = checker.facts.functions.iter().find(|f| f.definition == consumer)
        .ok_or("actual consumer function")?;
    let Some(Type::Function { parameters, result, .. }) = checker.facts.types.kind(function.signature)
    else { return Err("actual consumer signature".into()); };
    let byte = TypeId::primitive(Primitive::U8);
    assert_eq!(result, byte);
    assert_eq!(checker.facts.types.list(parameters).ok_or("actual parameters")?, [byte]);
    Ok(())
}

#[test]
fn alias_shorthand_uses_real_leaf_arguments_equalities_and_shared_seed() -> TestResult {
    let report = report(&source("canonical-equality")?)?;
    let mut checker = headers(&report)?;
    let consumer = signature(&mut checker)?;
    byte_signature(&checker, consumer)?;
    let leaf = definition(&report, "Leaf", DefKind::Trait)?;
    let mut original = 0;
    for (_, ty) in checker.facts.unreduced.node_entries() {
        let Some(Type::Projection(application)) = checker.facts.types.kind(ty) else { continue };
        let actual = checker.facts.types.trait_application(application.trait_application)
            .ok_or("original real leaf application")?;
        assert_eq!(actual.item, ItemId(leaf.0));
        assert_eq!(checker.facts.types.arguments(actual.arguments).ok_or("actual arguments")?
            .collect::<Vec<_>>(), vec![Argument::Type(TypeId::primitive(Primitive::U16))]);
        original += 1;
    }
    assert!(original >= 2, "both written projections retain their original real inputs");
    for alias in ["A", "Both"] {
        let alias = definition(&report, alias, DefKind::TraitAlias)?;
        assert!(checker.facts.associated.owner(alias).is_none());
        assert!(!checker.facts.generics.trait_self_bounds.contains_key(&alias));
    }
    assert!(checker.reject_alias_execution().is_err());
    eprintln!("alias naming Inputs={} legacyClause={} Checker={}",
        size_of::<super::super::super::projections::lookup::Inputs>(),
        size_of::<Clause>(), size_of::<Checker<'_>>());
    Ok(())
}

#[test]
fn alias_shorthand_keeps_real_member_and_application_ambiguity() -> TestResult {
    for name in ["ambiguous-members", "distinct-applications"] {
        let report = report(&source(name)?)?;
        let mut checker = headers(&report)?;
        let error = signature(&mut checker).err().ok_or("ambiguous alias member was accepted")?;
        assert!(error.to_string().contains("ambiguous shorthand associated type"), "{name}: {error}");
    }
    Ok(())
}

#[test]
fn alias_shorthand_filters_other_subjects_before_opening_their_contracts() -> TestResult {
    let report = report(&source("unrelated-subject")?)?;
    let mut checker = headers(&report)?;
    let error = signature(&mut checker).err().ok_or("another parameter supplied this member")?;
    assert!(error.to_string().contains("no associated type in the parameter's declared bounds"));
    let report = super::report(&source("same-actual-subject")?)?;
    let mut checker = headers(&report)?;
    signature(&mut checker)?;
    Ok(())
}

#[test]
fn alias_shorthand_preserves_unused_quantified_slots_and_failed_payment() -> TestResult {
    let report = report(&source("unused-binder")?)?;
    let mut checker = headers(&report)?;
    let consumer = signature(&mut checker)?;
    byte_signature(&checker, consumer)?;
    let mut checker = headers(&report)?;
    let owner = checker.trait_aliases.as_ref().ok_or("actual alias owner")?;
    owner.budget.steps.set(10_000_000);
    let error = signature(&mut checker).err().ok_or("unpaid naming was accepted")?;
    assert!(error.to_string().contains("trait alias work or retention budget exceeded"));
    assert!(checker.trait_aliases.as_ref().ok_or("actual failed owner")?.budget.steps.get() > 10_000_000);
    Ok(())
}
