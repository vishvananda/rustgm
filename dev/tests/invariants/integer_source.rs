//! Authored closed-range source supplies the real constructors and foreign call boundaries.
use crate::{expand::config::Config, pipeline, support::diagnostics::LintCap, test_source};
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
fn report(name: &str) -> Result<crate::resolve::expansion::Report, Box<dyn std::error::Error>> {
    let source = test_source::fixture(&format!("pa31/tests/spec/source-integer-niches/{name}.rs"))?;
    let mut config = Config::default();
    config.set_lint_cap(LintCap::Allow);
    Ok(test_source::source_provider_config(
        &source,
        "integer_niches",
        &config,
    )?)
}
fn produce(name: &str) -> TestResult<crate::lowir::program::Program> {
    let report = report(name)?;
    let pipeline::Analysis {
        mut facts,
        prepared,
    } = pipeline::analyze_source(&report)?;
    Ok(crate::lower::instances::build_roots_prepared(
        &report,
        &mut facts,
        crate::lower::Roots {
            entry: None,
            exports: &[],
        },
        prepared,
    )?)
}
#[test]
fn genuine_source_unsigned_niches_reach_ctfe_and_all_native_boundaries() -> TestResult {
    let program = produce("source-runtime")?;
    for primitive in ["u8", "u16", "u32", "u64", "u128", "usize"] {
        for prefix in [
            "echo",
            "make",
            "get",
            "invoke",
            "size",
            "align",
            "constant_none",
            "constant_some",
            "constant_round",
        ] {
            let name = format!("{prefix}_{primitive}");
            assert_eq!(
                program
                    .symbols
                    .iter()
                    .filter(|s| s.name == name && s.body.is_some())
                    .count(),
                1,
                "{name}"
            );
        }
    }
    let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../obj/parallel/source-integer-niches-v1/candidate");
    std::fs::create_dir_all(&folder)?;
    std::fs::write(
        folder.join("source.rs"),
        test_source::fixture("pa31/tests/spec/source-integer-niches/source-runtime.rs")?,
    )?;
    crate::lowir::writer::write(
        &program,
        &mut std::fs::File::create(folder.join("provider.lowir"))?,
    )?;
    assert_eq!(size_of::<crate::lower::context::ScalarStorage>(), 16);
    assert_eq!(size_of::<crate::lower::context::Representation>(), 24);
    assert_eq!(size_of::<crate::body::ir::Body>(), 456);
    assert_eq!(size_of::<crate::syntax::ast::Node>(), 36);
    Ok(())
}
#[test]
fn invalid_some_zero_and_wider_range_requirements_are_not_none_values() -> TestResult {
    for (name, needle) in [
        ("some-zero", "scalar pattern"),
        ("partial-range", "complete ABI profile"),
    ] {
        let result = produce(name);
        let error = result
            .err()
            .ok_or_else(|| format!("{name}: invalid or unsupported niche admitted"))?
            .to_string();
        assert!(error.contains(needle), "{name}: {error}");
    }
    Ok(())
}

#[test]
fn actual_noncopy_source_cleanup_distinguishes_none_and_some_integer_carriers() -> TestResult {
    let program = produce("cleanup/source-drop")?;
    for name in ["drop_value", "drop_local_none", "drop_local_some"] {
        assert_eq!(
            program
                .symbols
                .iter()
                .filter(|s| s.name == name && s.body.is_some())
                .count(),
            1,
            "{name}"
        );
    }
    let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../obj/parallel/source-integer-niches-v1/candidate/cleanup");
    std::fs::create_dir_all(&folder)?;
    std::fs::write(
        folder.join("source.rs"),
        test_source::fixture("pa31/tests/spec/source-integer-niches/cleanup/source-drop.rs")?,
    )?;
    crate::lowir::writer::write(
        &program,
        &mut std::fs::File::create(folder.join("provider.lowir"))?,
    )?;
    Ok(())
}
