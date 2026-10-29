//! Nounwind is an ABI/target contract, never an intrinsic or callable permission.
use crate::{pipeline, test_source};
type TestResult = Result<(), Box<dyn std::error::Error>>;

fn source(name: &str) -> Result<String, Box<dyn std::error::Error>> {
    Ok(test_source::fixture(&format!(
        "pa31/tests/spec/source-nounwind/{name}.rs"
    ))?)
}

fn lowered(input: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let report = test_source::source_provider(input, "source_nounwind")?;
    let pipeline::Analysis {
        mut facts,
        prepared,
    } = pipeline::analyze_source(&report)?;
    let catalog = facts.source_bindings.as_deref().ok_or("source catalog")?;
    for function in &facts.functions {
        assert!(
            !catalog.intrinsic(function.definition)?,
            "ordinary marker invented an intrinsic"
        );
    }
    let program = crate::lower::instances::build_roots_prepared(
        &report,
        &mut facts,
        crate::lower::Roots {
            entry: None,
            exports: &[],
        },
        prepared,
    )?;
    for name in ["observed_rust", "observed_c", "observed_pointer"] {
        assert!(
            program
                .symbols
                .iter()
                .any(|symbol| symbol.name == name && symbol.body.is_some()),
            "{name}"
        );
    }
    let mut bytes = Vec::new();
    crate::lowir::writer::write(&program, &mut bytes)?;
    Ok(bytes)
}

#[test]
fn ordinary_markers_preserve_actual_const_runtime_and_reified_bodies() -> TestResult {
    let input = source("source-runtime")?;
    let bytes = lowered(&input)?;
    assert_eq!(bytes, lowered(&source("ordinary-control")?)?);
    let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../obj/parallel/source-nounwind-v1/candidate-v1");
    std::fs::create_dir_all(&folder)?;
    std::fs::write(folder.join("provider.rs"), input)?;
    std::fs::write(folder.join("provider.lowir"), bytes)?;
    std::fs::write(
        folder.join(".keep-artifacts"),
        "Current ordinary source nounwind native input\n",
    )?;
    Ok(())
}

#[test]
fn ordinary_marker_grants_no_feature_shape_safety_or_const_permission() -> TestResult {
    for (name, needle) in [
        ("missing-feature", "rustc_attrs"),
        ("argument", "word"),
        ("unsafe-wrapper", "word"),
        ("non-function", "function"),
        ("non-const", "const"),
        ("unsafe-call", "unsafe"),
    ] {
        let result = test_source::source_provider(&source(name)?, "nounwind_opponent")
            .and_then(|report| pipeline::analyze_source(&report).map_err(|e| e.to_string()));
        let error = result
            .err()
            .ok_or_else(|| format!("invalid source accepted {name}"))?;
        assert!(error.contains(needle), "{name}: {error}");
        assert!(!error.contains("budget"), "{name}: {error}");
    }
    Ok(())
}

#[test]
fn fixed_abort_profile_cannot_be_overridden_and_unwind_abi_stays_required() -> TestResult {
    let mut config = crate::expand::config::Config::default();
    assert!(config.contains("panic", Some("abort")));
    assert!(!config.contains("panic", Some("unwind")));
    assert!(config.add("panic=\"unwind\"").is_err());
    let result = test_source::source_provider(&source("unwind-abi")?, "nounwind_future")
        .and_then(|report| pipeline::analyze_source(&report).map_err(|e| e.to_string()));
    let error = result.err().ok_or("unsupported unwind ABI admitted")?;
    assert!(error.contains("ABI"), "{error}");
    assert_eq!(size_of::<crate::resolve::package::Package>(), 256);
    assert_eq!(size_of::<crate::syntax::ast::Node>(), 36);
    assert_eq!(size_of::<crate::body::ir::Body>(), 456);
    Ok(())
}
