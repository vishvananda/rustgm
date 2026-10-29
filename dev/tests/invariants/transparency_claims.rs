//! Transparency promises preserve layout, privacy and authenticated source gates.
use crate::{pipeline, test_source};
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
fn fixture(name: &str) -> TestResult<String> {
    Ok(test_source::fixture(&format!(
        "pa31/tests/spec/public-transparency/{name}.rs"
    ))?)
}

#[test]
fn transparency_markers_preserve_actual_owners_gates_layout_and_field_privacy() -> TestResult {
    for name in ["record", "tuple-enum", "generated", "inactive"] {
        let report = test_source::resolve(&fixture(name)?, true)?;
        pipeline::analyze_source(&report).map_err(|error| format!("{name}: {error}"))?;
    }
    for name in [
        "missing-gate",
        "missing-repr",
        "repr-c",
        "function",
        "field",
        "crate",
        "arguments",
        "empty-arguments",
        "value-argument",
        "unsafe-wrapper",
        "two-fields",
        "private-field",
        "two-variants",
        "duplicates",
    ] {
        let result = test_source::resolve(&fixture(name)?, true).and_then(|report| {
            pipeline::analyze_source(&report)
                .map(|_| ())
                .map_err(|error| error.to_string())
        });
        let error = result.err().ok_or_else(|| format!("accepted {name}"))?;
        assert!(!error.contains("budget"), "{name}: {error}");
        assert!(!error.contains("wider profile"), "{name}: {error}");
    }
    Ok(())
}

#[test]
fn transparency_native_body_and_metadata_fence_preserve_the_real_contract() -> TestResult {
    let input = fixture("record")?;
    let report = test_source::resolve(&input, true)?;
    let pipeline::Analysis {
        mut facts,
        prepared,
    } = pipeline::analyze_source(&report)?;
    let exported = crate::sema::crates::export(
        &report,
        &mut facts,
        crate::support::crates::CrateIdentity::new("transparent", "contract")?,
    );
    let error = exported
        .err()
        .ok_or("transparent source contract was silently lost in metadata")?
        .to_string();
    assert!(
        error.contains("transparent record metadata is not implemented"),
        "{error}"
    );
    let program = crate::lower::instances::build_roots_prepared(
        &report,
        &mut facts,
        crate::lower::Roots {
            entry: None,
            exports: &[],
        },
        prepared,
    )?;
    assert!(
        program
            .symbols
            .iter()
            .any(|symbol| symbol.name == "observed" && symbol.body.is_some())
    );
    let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../obj/parallel/public-transparency-v1/candidate/native");
    std::fs::create_dir_all(&folder)?;
    std::fs::write(folder.join("provider.rs"), input)?;
    crate::lowir::writer::write(
        &program,
        &mut std::fs::File::create(folder.join("provider.lowir"))?,
    )?;
    Ok(())
}
