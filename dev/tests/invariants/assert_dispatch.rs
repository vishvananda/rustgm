//! Source Assert expansion has a real owner and leaves callable proof ordinary.
use super::*;
use crate::{syntax::ast::Unary, test_source};
type TestResult = Result<(), Box<dyn std::error::Error>>;
fn source() -> Result<String, Box<dyn std::error::Error>> {
    Ok(String::from(
        "#![feature(no_core,lang_items,rustc_attrs,allow_internal_unstable,staged_api)]#![no_core]#![unstable(feature=\"assert_consumer\",issue=\"none\")]",
    ) + &test_source::fixture("pa31/tests/spec/formatter-derives/assert-declaration.rs")?
        + "fn source(){assert!(condition());}")
}
#[test]
fn actual_assert_owner_installs_once_without_fabricating_panic_or_operator_permissions()
-> TestResult {
    let source = source()?;
    super::super::role_declarations::with_engine(&source, true, |engine| {
        engine.run()?;
        let invocation = engine
            .invocations
            .iter()
            .find(|invocation| {
                matches!(
                    engine.definitions[invocation.definition].body,
                    Body::Builtin(Builtin::Assert)
                )
            })
            .ok_or("actual source Assert invocation")?;
        assert_eq!(invocation.parser_calls, 1);
        let unit = &engine.loader.package.units[0];
        assert!(unit.source.text()[source.len()..].contains("assertion failed: condition()"));
        assert_eq!(
            unit.configured
                .ast
                .nodes
                .iter()
                .filter(|node| matches!(node.kind, Kind::Unary { op: Unary::Not, .. }))
                .count(),
            1
        );

        assert!(Builtin::Assert.bootstrap_slot().is_none());
        assert!(
            engine
                .paths
                .state
                .facts
                .source_bindings
                .as_deref()
                .and_then(|reserved| reserved.language(crate::support::bindings::LangItem::Not))
                .is_none()
        );
        Ok(())
    })?;
    let error = test_source::source_provider(&source, "source_assert_missing_panic")
        .err()
        .ok_or("source Assert fabricated its missing real panic target")?;
    assert!(error.contains("unresolved"), "{error}");
    let prefix = "#![feature(no_core,lang_items,rustc_attrs)]#![no_core]";
    for declaration in [
        "#[rustc_diagnostic_item=\"assert_macro\"]fn fake(){}",
        "#[rustc_diagnostic_item=\"assert_macro\"]#[rustc_builtin_macro(line)]macro_rules! fake{()=>{}}",
        "#[rustc_diagnostic_item=\"assert_macro\"]#[rustc_builtin_macro]fn assert(){}",
    ] {
        assert!(
            test_source::source_provider(&(String::from(prefix) + declaration), "bad_assert_owner")
                .is_err(),
            "{declaration}"
        );
    }
    eprintln!(
        "assert layouts Call={} Request={} Value={} Rvalue={} Body={} Package={} Report={} Reservations={}",
        size_of::<Call>(),
        size_of::<crate::expand::builtin::assertion::Request<'_>>(),
        size_of::<crate::body::ir::Value>(),
        size_of::<crate::body::ir::Rvalue>(),
        size_of::<crate::body::ir::Body>(),
        size_of::<crate::resolve::package::Package>(),
        size_of::<crate::resolve::expansion::Report>(),
        size_of::<crate::resolve::bindings::Reservations>()
    );
    Ok(())
}
