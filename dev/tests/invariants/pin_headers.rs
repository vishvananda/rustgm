//! Nominal identity and constructor completion never replace actual source headers.
use super::*;
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

#[test]
fn source_pin_and_constructor_roles_require_their_actual_completed_producers() -> TestResult {
    let source = crate::test_source::fixture("pa31/tests/spec/pin-source/actual-header.rs")?;
    let report = crate::test_source::source_provider(&source, "pin_headers")?;
    let pending = super::super::Catalog::reserve(&report)?.ok_or("source catalog")?;
    for role in [LangItem::Pin, LangItem::PinNewUnchecked] {
        assert!(pending.language(role).is_some());
        assert!(pending.validated_language(role).is_err());
    }
    let crate::pipeline::Analysis { mut facts, .. } = crate::pipeline::analyze_source(&report)?;
    let catalog = facts
        .source_bindings
        .as_deref()
        .ok_or("completed source catalog")?;
    let pin = catalog
        .validated_language(LangItem::Pin)?
        .ok_or("actual Pin")?;
    let constructor = catalog
        .validated_language(LangItem::PinNewUnchecked)?
        .ok_or("actual constructor")?;
    assert_ne!(pin.definition, constructor.definition);
    assert_eq!(
        facts
            .aggregates
            .destruction(crate::types::ItemId(pin.definition.0)),
        Some(crate::types::aggregate::FieldDestruction::Fields)
    );
    let provider = report
        .package
        .source_core()
        .ok_or("actual provider")?
        .provider
        .clone();
    let error = crate::sema::crates::authored_export::Stage::begin_module_conversions(
        &report, &mut facts, &provider,
    )
    .err()
    .ok_or("unencoded Pin role exported")?;
    assert_eq!(
        error,
        "general fundamental policy requires authenticated trait/nominal metadata transport"
    );
    let index = *facts
        .function_indices
        .get(&constructor.definition)
        .ok_or("constructor header")?;
    facts.functions[index].signature = crate::types::TypeId::UNIT;
    let error = complete(&report, &mut facts)
        .err()
        .ok_or("changed header accepted")?
        .to_string();
    assert!(error.contains("signature or body"), "{error}");
    Ok(())
}

#[test]
fn zero_domain_constructor_requires_its_completed_declaration_scheme() -> TestResult {
    let source =
        crate::test_source::fixture("pa31/tests/spec/pin-source/constructor-without-pin.rs")?;
    let report = crate::test_source::source_provider(&source, "pin_empty_header")?;
    let crate::pipeline::Analysis { mut facts, .. } = crate::pipeline::analyze_source(&report)?;
    let constructor = facts
        .source_bindings
        .as_deref()
        .ok_or("actual source catalog")?
        .validated_language(LangItem::PinNewUnchecked)?
        .ok_or("actual constructor")?;
    let signature = facts
        .function_header(constructor.definition)?
        .ok_or("source header")?
        .signature;
    assert!(declaration_signature(
        &facts,
        constructor.definition,
        signature
    ));
    let parameters = facts.types.intern_list(&[])?;
    let wrong = facts.types.intern(crate::types::Kind::Function {
        bound_regions: 0,
        parameters,
        result: crate::types::TypeId::UNIT,
        unsafe_: false,
        convention: crate::types::callable::Convention::RUST,
    })?;
    assert!(!declaration_signature(
        &facts,
        constructor.definition,
        wrong
    ));
    assert!(
        facts
            .lifetimes
            .functions
            .remove(&constructor.definition)
            .is_some()
    );
    assert!(!declaration_signature(
        &facts,
        constructor.definition,
        signature
    ));
    assert!(complete(&report, &mut facts).is_err());
    Ok(())
}

#[test]
fn real_constructor_declaration_and_relocated_callable_keep_distinct_lifetime_frames() -> TestResult
{
    let source =
        crate::test_source::fixture("pa31/tests/spec/pin-source/constructor-impl-lifetime.rs")?;
    let report = crate::test_source::source_provider(&source, "pin_frames")?;
    let crate::pipeline::Analysis { facts, .. } = crate::pipeline::analyze_source(&report)?;
    let constructor = facts
        .source_bindings
        .as_deref()
        .ok_or("source catalog")?
        .validated_language(LangItem::PinNewUnchecked)?
        .ok_or("actual constructor")?;
    let header = facts
        .function_header(constructor.definition)?
        .ok_or("declaration header")?;
    let public = facts
        .lifetimes
        .callables
        .get(crate::types::ItemId(constructor.definition.0))
        .ok_or("relocated callable")?;
    assert_ne!(header.signature, public.signature);
    assert!(declaration_signature(
        &facts,
        constructor.definition,
        header.signature
    ));
    Ok(())
}
