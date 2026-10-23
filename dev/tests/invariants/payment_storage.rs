//! Ordinary constants retain no new row owner or proved-contract Arc.
use super::*;
type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn ordinary_and_effect_free_source_constants_keep_payment_storage_absent() -> TestResult {
    let ordinary = crate::test_source::resolve("const fn value()->u8{7}const N:u8=value();", true)?;
    let (prefix, _) = crate::provider_cases::UNUSED_CONDITIONAL_CONST_PAID
        .split_once("const fn unused")
        .ok_or("prefix")?;
    let source = crate::test_source::source_provider(
        &(prefix.to_owned() + "const fn value()->u8{7}const N:u8=value();"),
        "no_effect_rows",
    )?;
    for report in [ordinary, source] {
        let analysis = crate::pipeline::analyze_source(&report)?;
        let context = analysis.prepared.constants.as_ref().ok_or("provider")?;
        let engine = context.engine.try_borrow()?;
        assert!(engine.const_calls.is_none());
        assert!(!engine.catalog.functions.is_empty());
    }
    assert_eq!(size_of::<CheckedInput>(), size_of::<usize>());
    println!(
        "Engine={} Machine={} State={} Body={} Code={} Value={} CheckedInput={} Row={} Continuation={}",
        size_of::<Engine>(),
        size_of::<const_eval::Machine<'_, '_>>(),
        size_of::<const_eval::State>(),
        size_of::<Body>(),
        size_of::<const_eval::code::Code>(),
        size_of::<crate::body::ir::Value>(),
        size_of::<CheckedInput>(),
        size_of::<const_eval::use_inputs::Row>(),
        const_eval::const_continuation_size()
    );
    Ok(())
}
