//! Suspended required queries share one report budget and one preparation cache.
use super::Context;
use crate::test_source as source;
use crate::{const_eval::Limits, sema::declarations};

fn analyze(text: &str, slots: usize) -> Result<crate::sema::model::Declarations, String> {
    let report = source::resolve(text, true)?;
    let context = Context::bounded(Limits {
        slots,
        ..Limits::default()
    });
    declarations::analyze_with_executor(&report, context.executor()).map_err(|e| e.to_string())
}

#[test]
fn nested_readiness_cannot_multiply_the_execution_storage_budget() -> Result<(), String> {
    let mut individual_peak = 0;
    for text in [
        "const fn a()->usize{let mut a=[0usize;16];a[0]=1;N+a[0]}type A=[u8;a()];const N:usize=2;",
        "const fn b()->usize{let mut a=[0usize;16];a[0]=1;a[0]}type B=[u8;b()];",
    ] {
        let facts = analyze(text, 64)?;
        individual_peak = individual_peak.max(facts.constant_work.peak_slots);
    }
    let combined = "const fn a()->usize{let mut a=[0usize;16];a[0]=1;N+a[0]}const fn b()->usize{let mut a=[0usize;16];a[0]=1;a[0]}type A=[u8;a()];const N:usize={let _:[u8;b()]=[0;1];2};";
    let facts = analyze(combined, 256)?;
    assert!(facts.constant_work.peak_suspended_slots > 0);
    let Err(error) = analyze(combined, individual_peak) else {
        return Err("nested continuations unexpectedly exceeded their shared budget".into());
    };
    assert!(error.contains("storage limit"), "{error}");
    Ok(())
}

#[test]
fn forward_value_headers_resume_without_evaluating_unrequested_values() -> Result<(), String> {
    for (text, cfgs) in [
        (
            "const fn f()->usize{N+1}type A=[u8;f()];const N:usize=2;",
            1,
        ),
        (
            "const N:usize=2;const fn f()->usize{N+1}type A=[u8;f()];",
            1,
        ),
        (
            "const fn f()->usize{g()}type A=[u8;f()];const fn g()->usize{3}",
            2,
        ),
        (
            "const fn g()->usize{3}const fn f()->usize{g()}type A=[u8;f()];",
            2,
        ),
        (
            "const fn f()->usize{let _=S(7);3}type A=[u8;f()];struct S(u32);",
            1,
        ),
        (
            "struct S(u32);const fn f()->usize{let _=S(7);3}type A=[u8;f()];",
            1,
        ),
        (
            "const fn f()->usize{let _=&S;3}type A=[u8;f()];static S:u32=7;",
            1,
        ),
        (
            "static S:u32=7;const fn f()->usize{let _=&S;3}type A=[u8;f()];",
            1,
        ),
    ] {
        let facts = analyze(text, 256).map_err(|error| format!("{text}: {error}"))?;
        assert!(
            facts.array_lengths.values().any(|&length| length == 3),
            "{text}"
        );
        assert_eq!(facts.body_work.cfg_functions.get(), cfgs, "{text}");
        assert!(facts.body_work.steps < 1000, "{text}");
    }
    let cycle = "const fn f()->usize{N}type A=[u8;f()];const N:usize=f();";
    let Err(error) = analyze(cycle, 256) else {
        return Err("true constant dependency cycle unexpectedly succeeded".into());
    };
    assert!(error.contains("cycle"), "{error}");
    Ok(())
}

#[test]
fn early_generic_const_functions_keep_closure_annotation_binders() -> Result<(), String> {
    for text in [
        "const fn value<T>()->usize{4}const fn wrapper<T>()->usize{return value::<T>();let _f=|x:&'static u8|x;}const N:usize=wrapper::<u32>();type A=[u8;N];",
        "const fn value<T>()->usize{4}const fn wrapper<T>()->usize{return value::<T>();let _f=|x:&u8|->u8{*x};}const N:usize=wrapper::<u32>();type A=[u8;N];",
    ] {
        let facts = analyze(text, 256)?;
        assert!(facts.array_lengths.values().any(|&n| n == 4));
    }
    Ok(())
}

#[test]
fn materialized_proof_input_is_a_cold_borrow_and_preserves_hot_body_sizes() {
    assert_eq!(size_of::<super::ProofInput<'_>>(), 32);
    assert_eq!(size_of::<crate::body::ir::Body>(), 448);
    assert_eq!(size_of::<crate::body::regions::Contract>(), 184);
    println!(
        "transient_proof_input={} engine={}",
        size_of::<super::ProofInput<'_>>(),
        size_of::<super::Engine>()
    );
}
