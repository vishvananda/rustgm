//! A source declaration cannot authorize the older primitive operator fallback.
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

#[test]
fn reached_source_shift_and_or_require_genuine_execution_hooks() -> TestResult {
    for name in [
        "missing-shift-impl.rs",
        "missing-or-impl.rs",
        "wrong-output.rs",
    ] {
        let source =
            crate::test_source::fixture(&format!("pa31/tests/spec/source-shift-or/{name}"))?;
        let report = crate::test_source::source_provider(&source, "source_binary_authority")?;
        let error = crate::pipeline::analyze_source(&report)
            .err()
            .ok_or("unproved source binary accepted")?;
        assert!(
            !error.message.contains("not implemented"),
            "{name}: {error}"
        );
    }
    for name in [
        "runtime-overloads.rs",
        "const-followup/primitive-const-impl.rs",
    ] {
        let source =
            crate::test_source::fixture(&format!("pa31/tests/spec/source-shift-or/{name}"))?;
        let report = crate::test_source::source_provider(&source, "source_binary_implemented")?;
        crate::pipeline::analyze_source(&report)?;
    }
    Ok(())
}

#[test]
fn missing_actual_source_shift_trait_cannot_borrow_primitive_execution() -> TestResult {
    let source =
        crate::test_source::fixture("pa31/tests/spec/source-shift-or/missing-shift-trait.rs")?;
    let report = crate::test_source::source_provider(&source, "missing_source_shift")?;
    let error = crate::pipeline::analyze_source(&report)
        .err()
        .ok_or("missing actual source Shl declaration accepted")?;
    assert_eq!(
        error.message,
        "source binary operator requires its actual language declaration"
    );
    Ok(())
}
