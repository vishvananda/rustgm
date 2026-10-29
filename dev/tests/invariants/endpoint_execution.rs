//! Authored helper bodies make genuine source selection visible without hardcoding.
use crate::{pipeline, test_source};
type TestResult = Result<(), Box<dyn std::error::Error>>;
fn source(name: &str) -> Result<String, Box<dyn std::error::Error>> {
    Ok(test_source::fixture(&format!(
        "pa31/tests/spec/source-pattern-endpoints/{name}.rs"
    ))?)
}
#[test]
fn authored_source_helpers_select_actual_constants_and_execute_the_real_override() -> TestResult {
    let input = source("authored-runtime")?;
    let report = test_source::source_provider(&input, "endpoint_execution")?;
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
    for name in [
        "observed_upper",
        "observed_lower",
        "observed_exclusive",
        "observed_signed",
        "observed_positive",
    ] {
        assert!(
            program
                .symbols
                .iter()
                .any(|s| s.name == name && s.body.is_some()),
            "{name}"
        );
    }
    let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../obj/parallel/pattern-endpoints-v1/candidate-v1");
    std::fs::create_dir_all(&folder)?;
    std::fs::write(folder.join("provider.rs"), input)?;
    std::fs::write(
        folder.join(".keep-artifacts"),
        "Current authored helper source/native producer\n",
    )?;
    crate::lowir::writer::write(
        &program,
        &mut std::fs::File::create(folder.join("provider.lowir"))?,
    )?;
    Ok(())
}
#[test]
fn reached_endpoints_keep_const_impl_validity_and_feature_requirements() -> TestResult {
    for (name, needle) in [
        ("outside-authored-max", "scalar pattern"),
        ("outside-authored-min", "scalar pattern"),
        ("zero-signed", "scalar pattern"),
        ("ordinary-implementation", "const"),
        ("missing-request", "pattern_types"),
        ("disconnected-positive", "wider"),
    ] {
        let result =
            test_source::source_provider(&source(name)?, "endpoint_opponent").and_then(|r| {
                pipeline::analyze_source(&r)
                    .map(|_| ())
                    .map_err(|e| e.to_string())
            });
        let error = result
            .err()
            .ok_or_else(|| format!("{name}: missing expected rejection/boundary"))?;
        assert!(error.to_lowercase().contains(needle), "{name}: {error}");
    }
    Ok(())
}
