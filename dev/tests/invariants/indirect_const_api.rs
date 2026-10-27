//! Retained const API words cannot manufacture source role or call permission.
use crate::{support::bindings::LangItem, test_source};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const FOUNDATION: &str = "pa31/tests/spec/source-location/location-copy-clone.rs";

#[test]
fn standard_handler_claim_and_weak_import_keep_distinct_actual_declarations() -> Result {
    // These are declaration-stage opponents, not validated handler signatures.
    let report = test_source::source_provider(
        "#![feature(no_core)]#![no_core]#[panic_handler]fn handler()->!{loop{}}",
        "standard_handler_claim",
    )?;
    let declaration = report
        .resolution
        .source_bindings
        .as_deref()
        .and_then(|bindings| bindings.language(LangItem::PanicImpl))
        .ok_or("standard claim")?;
    assert_eq!(
        report
            .package
            .names
            .get(report.resolution.definitions[declaration.definition.0 as usize].name),
        "handler"
    );
    let prefix = "#![feature(no_core,lang_items)]#![no_core]";
    let weak = "unsafe extern \"Rust\"{#[lang=\"panic_impl\"]fn weak()->!;}";
    let source = format!("{prefix}{weak}#[panic_handler]fn handler()->!{{loop{{}}}}");
    let report = test_source::source_provider(&source, "weak_handler_claim")?;
    let reserved = report
        .resolution
        .source_bindings
        .as_deref()
        .ok_or("reservations")?;
    let handler = reserved
        .language(LangItem::PanicImpl)
        .ok_or("defining handler")?;
    assert_eq!(report.resolution.foreign_owners.len(), 1);
    assert!(
        !report
            .resolution
            .foreign_owners
            .contains_key(&handler.definition)
    );
    let report = test_source::source_provider(&format!("{prefix}{weak}"), "weak_only")?;
    assert!(
        report
            .resolution
            .source_bindings
            .as_deref()
            .ok_or("weak reservations")?
            .language(LangItem::PanicImpl)
            .is_none()
    );
    for source in [
        "#[panic_handler(x)]fn handler()->!{loop{}}",
        "#[panic_handler]struct Wrong;",
        "#[panic_handler]fn a()->!{loop{}}#[panic_handler]fn b()->!{loop{}}",
        "unsafe extern \"Rust\"{#[panic_handler]fn foreign()->!;}",
    ] {
        assert!(
            test_source::source_provider(&format!("{prefix}{source}"), "bad_handler").is_err(),
            "{source}"
        );
    }
    Ok(())
}

fn check(tail: &str) -> Result<crate::pipeline::Analysis> {
    let source = test_source::fixture(FOUNDATION)? + tail;
    let report = test_source::source_provider(&source, "indirect_const_api")?;
    Ok(crate::pipeline::analyze_source(&report)?)
}
const TARGET: &str = "#[unstable(feature=\"indirect_regular\",issue=\"none\")]#[rustc_const_unstable(feature=\"indirect_const\",issue=\"none\")]#[rustc_const_stable_indirect]const fn indirect(x:u32)->u32{x}";
const CALLER: &str = "#[stable(feature=\"rust1\",since=\"1.0.0\")]#[rustc_const_stable(feature=\"caller_const\",since=\"1.0.0\")]const fn caller(x:u32)->u32{indirect(x)}";

#[test]
fn explicit_indirect_const_policy_allows_real_const_calls_without_runtime_exemption() -> Result {
    check(&(TARGET.to_owned() + CALLER))?;
    eprintln!(
        "const panic layouts Reservations={} Package={} Report={} Body={} Value={} Rvalue={} Library={} Inventory={}",
        size_of::<crate::resolve::bindings::Reservations>(),
        size_of::<crate::resolve::package::Package>(),
        size_of::<crate::resolve::expansion::Report>(),
        size_of::<crate::body::ir::Body>(),
        size_of::<crate::body::ir::Value>(),
        size_of::<crate::body::ir::Rvalue>(),
        size_of::<crate::resolve::stability::Library>(),
        size_of::<crate::resolve::stability::Inventory>()
    );
    let source = TARGET.replace("#[rustc_const_stable_indirect]", "") + CALLER;
    let error = check(&source)
        .err()
        .ok_or("missing indirect marker accepted")?;
    assert!(
        error
            .to_string()
            .contains("without const-stable API policy"),
        "{error}"
    );
    let runtime =
        "#[rustc_const_stable_indirect]fn runtime(x:u32)->u32{x}const BAD:u32=runtime(1u32);";
    assert!(
        check(runtime).is_err(),
        "API marker changed actual const eligibility"
    );
    Ok(())
}

#[test]
fn malformed_const_words_unknown_roles_and_explicit_stable_pairing_remain_closed() {
    for tail in [
        "#[rustc_const_stable_indirect(x)]const fn malformed(){}",
        "#[rustc_const_stable_indirect]trait Wrong{}",
        "#[rustc_do_not_const_check]#[rustc_const_stable_indirect]const fn unclaimed(){loop{}}",
        "#[stable(feature=\"rust1\",since=\"1.0.0\")]#[rustc_const_stable(feature=\"already_stable\",since=\"1.0.0\")]#[rustc_const_stable_indirect]const fn conflicting(){}",
        "#[lang=\"panic_fmt\"]#[rustc_do_not_const_check]#[rustc_const_stable_indirect]const fn wrong_signature()->!{loop{}}",
    ] {
        assert!(check(tail).is_err(), "{tail}");
    }
}
