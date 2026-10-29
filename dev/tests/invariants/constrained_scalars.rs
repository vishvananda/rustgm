//! Actual source pattern constructors retain validity through derived Clone/Copy.
use crate::{
    pipeline, test_source,
    types::{Constraint, Kind},
};
type TestResult = Result<(), Box<dyn std::error::Error>>;
fn source(name: &str) -> Result<String, Box<dyn std::error::Error>> {
    Ok(test_source::fixture(&format!(
        "pa31/tests/spec/source-scalar-patterns/{name}.rs"
    ))?)
}
#[test]
fn actual_closed_inclusive_source_keeps_constraints_and_emits_native_roots() -> TestResult {
    let input = source("source-runtime")?;
    let report = test_source::source_provider(&input, "closed_scalar_patterns")?;
    let pipeline::Analysis {
        mut facts,
        prepared,
    } = pipeline::analyze_source(&report)?;
    let mut ranges = 0;
    for index in 0..facts.types.len() {
        if let Some(Kind::Pattern {
            base,
            constraint: Constraint::ScalarRange(id),
        }) = facts
            .types
            .kind(crate::types::TypeId(u32::try_from(index)?))
        {
            let range = facts.types.scalar_range(id).ok_or("actual scalar range")?;
            assert_eq!(range.base, base);
            ranges += 1;
        }
    }
    assert!(ranges >= 4);
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
        "observed_low",
        "observed_mid",
        "observed_high",
        "observed_signed",
        "observed_char",
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
        .join("../obj/parallel/closed-scalar-patterns-v1/candidate-v1");
    std::fs::create_dir_all(&folder)?;
    std::fs::write(folder.join("provider.rs"), input)?;
    std::fs::write(
        folder.join(".keep-artifacts"),
        "Current closed scalar source/native producer\n",
    )?;
    crate::lowir::writer::write(
        &program,
        &mut std::fs::File::create(folder.join("provider.lowir"))?,
    )?;
    Ok(())
}
#[test]
fn reached_constants_reject_range_base_initialization_and_provenance_violations() -> TestResult {
    for (name, needle) in [
        ("outside-high", "scalar pattern"),
        ("outside-low", "scalar pattern"),
        ("char-base", "char"),
        ("uninitialized", "initial"),
        ("pointer-bits", "pointer"),
        ("reversed", "reversed"),
        ("missing-gate", "pattern_types"),
    ] {
        let result =
            test_source::source_provider(&source(name)?, "scalar_negative").and_then(|r| {
                pipeline::analyze_source(&r)
                    .map(|_| ())
                    .map_err(|e| e.to_string())
            });
        let error = result
            .err()
            .ok_or_else(|| format!("{name}: invalid constant accepted"))?;
        assert!(error.to_lowercase().contains(needle), "{name}: {error}");
    }
    Ok(())
}
