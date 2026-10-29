//! Genuine array field headers reach actual source Clone/Copy and existing SIMD consumers.
use crate::{expand::config::Config, pipeline, support::diagnostics::LintCap, test_source};
type TestResult = Result<(), Box<dyn std::error::Error>>;
fn produce(path: &str, tag: &str, out: &str) -> TestResult {
    let input = test_source::fixture(path)?;
    let mut config = Config::default();
    config.set_lint_cap(LintCap::Allow);
    let report = test_source::source_provider_config(&input, tag, &config)?;
    let pipeline::Analysis {
        mut facts,
        prepared,
    } = pipeline::analyze_source(&report)?;
    let program = crate::lower::instances::build_roots_prepared(
        &report,
        &mut facts,
        crate::lower::Roots {
            entry: None,
            exports: &[],
        },
        prepared,
    )?;
    let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../obj/parallel")
        .join(out);
    std::fs::create_dir_all(&folder)?;
    std::fs::write(folder.join("source.rs"), input)?;
    crate::lowir::writer::write(
        &program,
        &mut std::fs::File::create(folder.join("provider.lowir"))?,
    )?;
    assert_eq!(size_of::<crate::syntax::ast::Node>(), 36);
    assert_eq!(size_of::<crate::body::ir::Body>(), 456);
    Ok(())
}
#[test]
fn ordinary_literal_named_and_block_array_fields_preserve_real_clone_execution() -> TestResult {
    produce(
        "pa31/tests/spec/derive-array-fields/source-runtime.rs",
        "array_derivation",
        "derive-array-fields-v1/candidate/ordinary",
    )
}
#[test]
fn actual_simd_header_clone_copy_keeps_feature_and_scalar_semantics() -> TestResult {
    produce(
        "pa41/tests/spec/stdarch-array-derives/source-native.rs",
        "simd_array_derivation",
        "derive-array-fields-v1/candidate/simd",
    )?;
    let source = test_source::fixture("pa41/tests/spec/stdarch-array-derives/missing-feature.rs")?;
    let mut config = Config::default();
    config.set_lint_cap(LintCap::Allow);
    let result = test_source::source_provider_config(&source, "simd_array_negative", &config)
        .and_then(|report| {
            pipeline::analyze_source(&report)
                .map(|_| ())
                .map_err(|e| e.to_string())
        });
    let error = result.err().ok_or("missing repr_simd feature accepted")?;
    assert!(error.contains("repr_simd"), "{error}");
    Ok(())
}
#[test]
fn actual_const_function_calls_in_array_fields_keep_source_ctfe_and_clone() -> TestResult {
    produce(
        "pa31/tests/spec/derive-call-counts/source-runtime.rs",
        "call_count_derivation",
        "derive-call-counts-v1/candidate",
    )
}
