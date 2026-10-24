//! Actual trait/member tables constrain identity before conformance proves types.
use super::*;
use crate::{
    expand::marks::{SyntaxContextId, Transparency},
    sema::model::Declarations,
};
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

fn definition(report: &Report, name: &str, kind: DefKind) -> TestResult<DefId> {
    let index = report
        .resolution
        .definitions
        .iter()
        .position(|item| item.kind == kind && report.package.names.get(item.name) == name)
        .ok_or("actual declaration")?;
    Ok(DefId(u32::try_from(index)?))
}

fn members(
    report: &Report,
    facts: &Declarations,
    trait_item: DefId,
) -> TestResult<Vec<(DefId, DefId)>> {
    let mut pairs = Vec::new();
    for (member, _) in facts.associated.declaration_members() {
        if facts.associated.trait_owner(member).is_none()
            && let Some(expected) = facts
                .associated
                .matching_member(report, trait_item, member)?
        {
            pairs.push((expected, member));
        }
    }
    Ok(pairs)
}

#[test]
fn canonical_members_preserve_early_projection_identity_and_ordinary_defaults() -> TestResult {
    let report = crate::test_source::resolve(
        "struct S; trait T { type A; const N:u8=9; fn value()->u8{7} }
         impl T for S { type A=u8; const N:u8=7; fn value()->u8{9} }",
        true,
    )?;
    let facts = crate::sema::declarations::analyze(&report)?;
    let trait_item = definition(&report, "T", DefKind::Trait)?;
    let pairs = members(&report, &facts, trait_item)?;
    assert_eq!(pairs.len(), 3);
    for (expected, actual) in pairs {
        if report.resolution.definitions[actual.0 as usize].kind == DefKind::TypeAlias {
            let owner = facts
                .associated
                .member_owner(actual)
                .ok_or("actual impl owner")?;
            assert_eq!(
                facts.associated.type_definition(owner, expected),
                Some(actual)
            );
            assert_eq!(
                facts.associated.type_definition(owner, actual),
                Some(actual)
            );
        }
    }
    println!(
        "canonical-member-layout inventory={} declarations={}",
        size_of::<Inventory>(),
        size_of::<Declarations>()
    );
    Ok(())
}

#[test]
fn actual_macro_impl_identifier_adjusts_against_the_trait_container() -> TestResult {
    let mut report = crate::test_source::resolve(
        "struct S; trait T { fn value()->u8; }
         macro_rules! make { () => { fn value()->u8 {7} } }
         impl T for S { make!(); }",
        true,
    )?;
    let facts = crate::sema::declarations::analyze(&report)?;
    let trait_item = definition(&report, "T", DefKind::Trait)?;
    let pairs = members(&report, &facts, trait_item)?;
    let &(expected, actual) = pairs.first().ok_or("actual override")?;
    let invocation = report
        .invocations
        .first()
        .ok_or("actual invocation")?
        .context;
    let syntax = report.package.marks.apply(
        SyntaxContextId::ROOT,
        invocation,
        Transparency::Opaque,
        SyntaxContextId::ROOT,
    )?;
    let context = report.package.marks.item(syntax)?;
    // A supplied opaque query isolates the consumer; scoped source execution stays disabled.
    for _ in 0..2 {
        assert_eq!(
            facts
                .associated
                .matching_context(&report, trait_item, actual, context)?,
            Some(expected)
        );
    }
    report.invocations[0].definition = usize::MAX;
    assert!(
        facts
            .associated
            .matching_context(&report, trait_item, actual, context)
            .is_err()
    );
    Ok(())
}

#[test]
fn duplicate_canonical_targets_cannot_disappear_from_the_member_inventory() -> TestResult {
    let mut report = crate::test_source::resolve(
        "struct S; trait T { fn first(); fn second(); }
         impl T for S { fn first() {} fn second() {} }",
        true,
    )?;
    let facts = crate::sema::declarations::analyze(&report)?;
    let trait_item = definition(&report, "T", DefKind::Trait)?;
    let pairs = members(&report, &facts, trait_item)?;
    assert_eq!(pairs.len(), 2);
    let actual: Vec<_> = pairs.iter().map(|pair| pair.1).collect();
    report.resolution.definitions[actual[1].0 as usize].name =
        report.resolution.definitions[actual[0].0 as usize].name;
    let first = facts
        .associated
        .matching_member(&report, trait_item, actual[0])?;
    assert_eq!(
        facts
            .associated
            .matching_member(&report, trait_item, actual[1])?,
        first
    );
    // Actual source duplicates are also rejected by the earlier resolver. This
    // independently exercises the canonical mapping gate after inventory.
    let error = crate::sema::declarations::analyze(&report)
        .err()
        .ok_or("duplicate must fail")?;
    assert!(
        error
            .to_string()
            .contains("multiple implementation members match the same trait member")
    );
    Ok(())
}

#[test]
fn member_kinds_and_foreign_containers_do_not_gain_identity_by_spelling() -> TestResult {
    let mut report = crate::test_source::resolve(
        "struct S; trait T { fn value(); } trait U { fn value(); }
         impl T for S { fn value() {} }",
        true,
    )?;
    let facts = crate::sema::declarations::analyze(&report)?;
    let trait_item = definition(&report, "T", DefKind::Trait)?;
    let other = definition(&report, "U", DefKind::Trait)?;
    let pairs = members(&report, &facts, trait_item)?;
    let &(_, actual) = pairs.first().ok_or("actual override")?;
    report.resolution.definitions[actual.0 as usize].kind = DefKind::Const;
    assert_eq!(
        facts
            .associated
            .matching_member(&report, trait_item, actual)?,
        None
    );
    report.resolution.definitions[actual.0 as usize].kind = DefKind::Function;
    let name = report.resolution.definitions[actual.0 as usize].name;
    let namespace = report.resolution.definitions[actual.0 as usize].namespace;
    let scope = report.resolution.definitions[trait_item.0 as usize]
        .members
        .ok_or("trait scope")?;
    let foreign = report.resolution.definitions[other.0 as usize]
        .members
        .ok_or("other scope")?;
    let binding = *report
        .resolution
        .bindings
        .get_key((foreign, name, namespace, ItemContext::ROOT))
        .ok_or("foreign binding")?;
    report
        .resolution
        .bindings
        .insert_key((scope, name, namespace, ItemContext::ROOT), binding)?;
    assert_eq!(
        facts
            .associated
            .matching_member(&report, trait_item, actual)?,
        None
    );
    assert!(
        facts
            .associated
            .matching_member(&report, DefId(u32::MAX), actual)
            .is_err()
    );
    Ok(())
}
