//! Float fallback requires its completed current language owner and canonical header.
use super::*;
use crate::{sema, types::TypeId};
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
const FIXTURE: &str = "pa31/tests/spec/source-from/actual-role.rs";
fn goal(facts: &Declarations, context: &mut Context) -> TestResult<(Predicate, TermId)> {
    let definition = facts
        .source_bindings
        .as_deref()
        .ok_or("catalog")?
        .role(LangItem::From)?
        .ok_or("From declaration")?
        .definition;
    let subject = context.known(&facts.types, TypeId::primitive(Primitive::F32))?;
    let input = context.variable(Domain::Float)?;
    Ok((
        Predicate::trait_application(subject, ItemId(definition.0), vec![Argument::Type(input)]),
        input,
    ))
}
#[test]
fn reserved_or_missing_canonical_headers_cannot_supply_float_preferences() -> TestResult {
    let report =
        crate::test_source::source_provider(&crate::test_source::fixture(FIXTURE)?, "float_owner")?;
    let reserved = Declarations::new(&report)?;
    let mut context = Context::default();
    let (predicate, _) = goal(&reserved, &mut context)?;
    assert!(float_default(&reserved, &mut context, &predicate).is_err());
    let mut facts = sema::declarations::analyze(&report)?;
    let mut context = Context::default();
    let (predicate, input) = goal(&facts, &mut context)?;
    assert_eq!(
        float_default(&facts, &mut context, &predicate)?,
        Some(input)
    );
    facts.generics.traits = None;
    assert!(float_default(&facts, &mut context, &predicate).is_err());
    Ok(())
}
#[test]
fn completed_preferences_are_warm_and_require_the_exact_role_identity() -> TestResult {
    let report =
        crate::test_source::source_provider(&crate::test_source::fixture(FIXTURE)?, "float_warm")?;
    let facts = sema::declarations::analyze(&report)?;
    let mut context = Context::default();
    let (predicate, input) = goal(&facts, &mut context)?;
    assert_eq!(
        float_default(&facts, &mut context, &predicate)?,
        Some(input)
    );
    let allocation = context.retained_allocation();
    for _ in 0..32 {
        assert_eq!(
            float_default(&facts, &mut context, &predicate)?,
            Some(input)
        );
    }
    assert_eq!(context.retained_allocation(), allocation);
    let ordinary = Predicate::trait_application(
        predicate.subject(),
        ItemId(u32::MAX),
        vec![Argument::Type(input)],
    );
    assert_eq!(float_default(&facts, &mut context, &ordinary)?, None);
    Ok(())
}

#[test]
fn actual_type_formal_identity_and_sticky_query_payment_are_required() -> TestResult {
    let report = crate::test_source::source_provider(
        &crate::test_source::fixture(FIXTURE)?,
        "float_formal",
    )?;
    let mut facts = sema::declarations::analyze(&report)?;
    let mut context = Context::default();
    let (predicate, input) = goal(&facts, &mut context)?;
    let definition = facts
        .source_bindings
        .as_deref()
        .ok_or("catalog")?
        .role(LangItem::From)?
        .ok_or("From role")?
        .definition;
    let saved = facts
        .generics
        .parameters
        .get(&definition)
        .ok_or("formals")?[0];
    let self_type = facts
        .associated
        .schema(definition)
        .ok_or("schema")?
        .self_type;
    facts
        .generics
        .parameters
        .get_mut(&definition)
        .ok_or("formals")?[0]
        .argument = crate::types::arguments::Argument::Type(self_type);
    assert!(float_default(&facts, &mut context, &predicate).is_err());
    facts
        .generics
        .parameters
        .get_mut(&definition)
        .ok_or("formals")?[0] = saved;
    assert_eq!(
        float_default(&facts, &mut context, &predicate)?,
        Some(input)
    );
    let allocation = context.retained_allocation();
    for count in [8, 32, 128] {
        let before = sema::header_attributes::source_contract_costs(&facts);
        for _ in 0..count {
            assert_eq!(
                float_default(&facts, &mut context, &predicate)?,
                Some(input)
            );
        }
        let after = sema::header_attributes::source_contract_costs(&facts);
        assert_eq!(after.0 - before.0, count * 16);
        assert_eq!(after.1, before.1);
        assert_eq!(context.retained_allocation(), allocation);
    }
    assert!(
        sema::header_attributes::charge_source_contract_query(
            &facts,
            crate::support::crates::MAX_COMPONENTS,
        )
        .is_err()
    );
    for _ in 0..2 {
        assert!(float_default(&facts, &mut context, &predicate).is_err());
        assert_eq!(context.retained_allocation(), allocation);
    }
    Ok(())
}
