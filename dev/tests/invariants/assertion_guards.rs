//! Existing role completion and canonical source owners remain the only authority.
use super::*;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

fn prepared(name: &str) -> Result<(Report, Declarations)> {
    let source =
        crate::test_source::fixture(&format!("pa31/tests/spec/assertion-panics/{name}.rs"))?;
    let report = crate::test_source::source_provider(&source, "assertion_guards")?;
    let facts = crate::sema::declarations::analyze(&report)?;
    Ok((report, facts))
}

#[test]
fn panic_headers_complete_actual_roles_without_granting_guard_execution() -> Result {
    let (report, facts) = prepared("original-signatures")?;
    let catalog = facts.source_bindings.as_deref().ok_or("actual catalog")?;
    assert_eq!(
        complete(&report, &facts, catalog)?
            .into_iter()
            .flatten()
            .count(),
        28
    );
    for role in ROLES {
        assert!(catalog.validated_role(role)?.is_some());
        assert!(require_guard(&facts, role).is_err());
    }
    assert!(
        require_future_guard(&facts)
            .err()
            .ok_or("future boundary")?
            .contains("distinct checked source panic causes")
    );
    assert!(require_guard(&facts, LangItem::Sized).is_err());
    Ok(())
}

#[test]
fn stale_function_and_foreign_provider_never_supply_a_panic_guard() -> Result {
    let (report, mut facts) = prepared("unused-bounds-alternative")?;
    let source = crate::test_source::fixture(
        "pa31/tests/spec/assertion-panics/unused-bounds-alternative.rs",
    )?;
    let foreign = crate::test_source::source_provider(&source, "foreign_panic_provider")?;
    assert!(
        complete(
            &foreign,
            &facts,
            facts.source_bindings.as_deref().ok_or("catalog")?
        )
        .is_err()
    );
    let role = facts
        .source_bindings
        .as_deref()
        .ok_or("catalog")?
        .language(LangItem::PanicBoundsCheck)
        .ok_or("actual role")?;
    let original =
        facts.definitions[role.definition.0 as usize].replace(crate::types::TypeId::UNIT);
    assert!(
        complete(
            &report,
            &facts,
            facts.source_bindings.as_deref().ok_or("catalog")?
        )
        .is_err()
    );
    assert!(require_guard(&facts, LangItem::PanicBoundsCheck).is_err());
    facts.definitions[role.definition.0 as usize] = original;
    facts
        .source_bindings
        .as_deref_mut()
        .ok_or("catalog")?
        .traits = None;
    assert!(require_guard(&facts, LangItem::PanicBoundsCheck).is_err());
    Ok(())
}

#[test]
fn guard_queries_keep_absence_cheap_and_real_source_payments_sticky() -> Result {
    let (_, facts) = prepared("unused-bounds-alternative")?;
    let before = crate::sema::header_attributes::source_contract_costs(&facts);
    for _ in 0..128 {
        require_guard(&facts, LangItem::PanicAddOverflow)?;
    }
    assert_eq!(
        crate::sema::header_attributes::source_contract_costs(&facts),
        before
    );
    let payments =
        crate::sema::header_attributes::source_payments(facts.must_use_contracts.as_deref())?;
    assert!(
        payments
            .charge(crate::support::crates::MAX_COMPONENTS + 1, 0)
            .is_err()
    );
    assert!(require_guard(&facts, LangItem::PanicBoundsCheck).is_err());
    assert!(require_guard(&facts, LangItem::PanicBoundsCheck).is_err());
    Ok(())
}
