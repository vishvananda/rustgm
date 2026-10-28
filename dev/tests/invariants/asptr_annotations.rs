//! A typed raw lint annotation cannot authorize operations or change hard errors.
use crate::test_source;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
fn fixture(name: &str) -> Result<String> {
    Ok(test_source::fixture(&format!(
        "pa26/tests/spec/attributes/as-ptr/{name}.rs"
    ))?)
}
#[test]
fn actual_pointer_lint_words_keep_requests_owners_and_body_obligations() -> Result {
    let valid = fixture("valid")?;
    let report = test_source::resolve(&valid, true)?;
    crate::pipeline::analyze_source(&report)?;
    let source =
        test_source::fixture("pa31/tests/spec/source-functions-transport/minimal-provider.rs")?
            + &valid;
    // Crate requests must remain on the root; this source form uses its existing request.
    let source = source
        .replace("#![feature(rustc_attrs)]", "")
        .replace("#![allow(internal_features,dead_code)]", "");
    let report = test_source::source_provider(&source, "raw_lint_annotation")?;
    let facts = crate::pipeline::analyze_source(&report)?.facts;
    authenticated_marker(&report, &facts)?;
    publication_profiles()?;
    missing_marker_owner()?;
    for name in [
        "missing-request",
        "wrong-owner",
        "foreign-prototype",
        "malformed",
        "unsafe-wrapper",
    ] {
        let source = fixture(name)?;
        let result: Result<crate::pipeline::Analysis> = test_source::resolve(&source, true)
            .map_err(Into::into)
            .and_then(|report| crate::pipeline::analyze_source(&report).map_err(Into::into));
        assert!(result.is_err(), "{name}");
    }
    let hard = valid.replace("fn numeric()->u8{7}", "fn numeric()->u8{true}");
    let report = test_source::resolve(&hard, true)?;
    assert!(crate::pipeline::analyze_source(&report).is_err());
    assert_eq!(size_of::<crate::support::decorations::Consumer>(), 1);
    assert_eq!(size_of::<crate::body::ir::Body>(), 456);
    Ok(())
}

fn authenticated_marker(
    report: &crate::resolve::expansion::Report,
    facts: &crate::sema::model::Declarations,
) -> Result {
    use crate::sema::header_attributes::ContractKnowledge;
    let index = report
        .resolution
        .definitions
        .iter()
        .position(|row| report.package.names.get(row.name) == "numeric")
        .ok_or("actual numeric owner")?;
    let definition = crate::resolve::model::DefId(u32::try_from(index)?);
    assert!(matches!(
        facts.pointer_lint_marker(report, definition)?,
        ContractKnowledge::KnownPresent(())
    ));
    let error = facts
        .reject_function_advisory_transport(report, definition)
        .err()
        .ok_or("lost marker exported")?;
    assert!(
        error
            .to_string()
            .contains("marker transport is not implemented")
    );
    let mut changed = test_source::source_provider(
        &(test_source::fixture("pa31/tests/spec/source-functions-transport/minimal-provider.rs")?
            + "#[rustc_as_ptr]fn numeric()->u8{7}"),
        "marker_owner_change",
    )?;
    let checked = crate::pipeline::analyze_source(&changed)?.facts;
    let (index, key) = changed
        .resolution
        .definitions
        .iter()
        .enumerate()
        .find_map(|(index, row)| {
            (changed.package.names.get(row.name) == "numeric").then_some((index, row.node?))
        })
        .ok_or("actual changed owner")?;
    let other = changed
        .resolution
        .definitions
        .iter()
        .find_map(|row| (changed.package.names.get(row.name) == "identity").then_some(row.node?))
        .ok_or("other actual source owner")?;
    let foreign = changed
        .package
        .unit(other.file)
        .configured
        .ast
        .node(other.node)
        .kind;
    changed.package.units[key.file.0 as usize]
        .configured
        .ast
        .nodes[key.node.get() as usize]
        .kind = foreign;
    assert!(
        checked
            .pointer_lint_marker(
                &changed,
                crate::resolve::model::DefId(u32::try_from(index)?)
            )
            .is_err()
    );
    Ok(())
}

fn publication_profiles() -> Result {
    use crate::lower::instances::SourcePublication;
    let source =
        test_source::fixture("pa31/tests/spec/source-functions-transport/minimal-provider.rs")?
            .replace("pub fn identity<T>", "#[rustc_as_ptr]pub fn identity<T>");
    for profile in [
        SourcePublication::LegacyFunctions,
        SourcePublication::FunctionContracts,
    ] {
        let error =
            crate::module_packets::exported_profile(&source, profile, "raw_marker_publication")
                .err()
                .ok_or("marker profile silently dropped diagnostic data")?;
        assert!(
            error
                .to_string()
                .contains("marker transport is not implemented"),
            "{error}"
        );
    }
    Ok(())
}

fn missing_marker_owner() -> Result {
    let mut report = test_source::resolve(&fixture("valid")?, true)?;
    let row = report
        .resolution
        .definitions
        .iter_mut()
        .find(|row| report.package.names.get(row.name) == "numeric")
        .ok_or("actual marker source")?;
    row.node = None;
    let error = crate::pipeline::analyze_source(&report)
        .err()
        .ok_or("unbound marker accepted")?;
    assert!(
        error
            .to_string()
            .contains("marker lacks its actual function definition"),
        "{error}"
    );
    Ok(())
}
