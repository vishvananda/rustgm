//! An actual callable-role identity is current data, never call permission.
use super::*;
type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn callable_role_data_rejects_pending_missing_and_mismatched_current_registry() -> TestResult {
    let source = crate::test_source::fixture(
        "pa31/tests/spec/source-const-payment/callable/const-function-item.rs",
    )?;
    let report = crate::test_source::source_provider(&source, "invocation_roles")?;
    let mut facts = crate::sema::declarations::analyze(&report)?;
    for (kind, role) in [
        (CallTrait::FnOnce, LangItem::FnOnce),
        (CallTrait::FnMut, LangItem::FnMut),
        (CallTrait::Fn, LangItem::Fn),
    ] {
        assert_eq!(
            definition(&facts, kind)?,
            facts
                .source_bindings
                .as_deref()
                .ok_or("catalog")?
                .language(role)
                .ok_or("role")?
                .definition
        );
    }
    let mut saved = facts.source_bindings.take().ok_or("catalog")?;
    assert!(definition(&facts, CallTrait::FnOnce).is_err());
    facts.source_bindings = Catalog::reserve(&report)?;
    assert!(definition(&facts, CallTrait::FnOnce).is_err());
    let original = saved.language(LangItem::FnOnce).ok_or("FnOnce")?;
    saved.languages[LangItem::FnOnce.index()] = saved.language(LangItem::FnMut);
    facts.source_bindings = Some(saved);
    assert!(definition(&facts, CallTrait::FnOnce).is_err());
    let current = facts.source_bindings.as_deref_mut().ok_or("catalog")?;
    current.languages[LangItem::FnOnce.index()] = None;
    assert!(definition(&facts, CallTrait::FnOnce).is_err());
    facts
        .source_bindings
        .as_deref_mut()
        .ok_or("catalog")?
        .languages[LangItem::FnOnce.index()] = Some(original);
    assert_eq!(definition(&facts, CallTrait::FnOnce)?, original.definition);
    Ok(())
}
