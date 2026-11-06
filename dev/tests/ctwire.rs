//! Imported const precision is a producer policy, separate from consumer requests.
use crate::resolve as source;
use rustgm::{lower, pipeline, support::crates};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
fn fixture(name: &str) -> Result<String> {
    Ok(source::fixture(&format!(
        "pa31/tests/spec/source-const-cleanup/{name}.rs"
    ))?)
}
fn provider(name: &str) -> Result<Vec<u8>> {
    let report = source::source_provider(&fixture(name)?, "const_import_policy")?;
    let identity = report
        .package
        .source_core()
        .ok_or("provider")?
        .provider
        .clone();
    let pipeline::Analysis {
        mut facts,
        prepared,
    } = pipeline::analyze_source(&report)?;
    let (_, interface) = lower::instances::build_source_library_profile(
        &report,
        &mut facts,
        identity,
        prepared,
        lower::instances::SourcePublication::ModuleNominals,
    )?;
    Ok(crates::encode(&interface)?)
}
#[test]
fn conditional_composite_original_advances_to_the_required_type_row() -> Result {
    // This unchanged Rust-positive provider now passes defining cleanup. Export
    // still requires its tuple type row; no imported-policy result is claimed.
    let error = provider("impro3")
        .err()
        .ok_or("qualify actual conditional composite original")?;
    assert!(
        error
            .to_string()
            .contains("source type form requires an explicitly supported transport row"),
        "{error}"
    );
    Ok(())
}

#[test]
fn unconditional_tuple_export_retains_its_required_type_row_gap() -> Result {
    let error = provider("impro4")
        .err()
        .ok_or("qualify actual tuple transport")?;
    assert!(
        error
            .to_string()
            .contains("source type form requires an explicitly supported transport row"),
        "{error}"
    );
    Ok(())
}

#[test]
fn alternate_marker_declaration_retains_its_named_closed_export_profile() -> Result {
    let error = provider("impro5")
        .err()
        .ok_or("qualify alternate marker export profile")?;
    assert!(
        error
            .to_string()
            .contains("source Destruct requires its exact closed const marker declaration"),
        "{error}"
    );
    Ok(())
}

#[test]
fn exact_nominal_original_retains_its_required_qualified_body_profile() -> Result {
    // Raw provider and both clients are Rust-positive. Export stops before any
    // import-policy query, so end-to-end imported precision coverage is pending.
    let error = provider("impro6")
        .err()
        .ok_or("qualify exact imported const policy")?;
    assert!(
        error
            .to_string()
            .contains("source qualified body request requires a wider source profile"),
        "{error}"
    );
    Ok(())
}
