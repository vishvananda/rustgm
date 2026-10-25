//! Local openness is validated without granting the closed artifact profile.
use rustgm::sema::non_exhaustive::Target;
use std::path::Path;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

fn source(name: &str) -> Result<String> {
    Ok(std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../pa29/tests/spec/non-exhaustive-local")
            .join(format!("{name}.rs")),
    )?)
}

#[test]
fn local_non_exhaustive_contracts_preserve_construction_and_complete_matches() -> Result {
    for name in [
        "same-crate-structs",
        "same-crate-enum",
        "same-crate-variants",
        "same-crate-empty-enum",
        "disabled-invalid",
    ] {
        let report = crate::resolve::resolve(&source(name)?, true)?;
        let analysis = rustgm::pipeline::analyze_source(&report)
            .map_err(|error| format!("{name}: {error}"))?;
        assert_eq!(analysis.facts.warnings().count(), 0, "{name}");
        for definition in &report.resolution.definitions {
            let Some(key) = definition.node else { continue };
            if report.package.names.get(definition.name) != "Never" {
                continue;
            }
            let contract = analysis
                .facts
                .non_exhaustive_contract(&report, key)?
                .ok_or("empty enum lost its local contract")?;
            assert_eq!(contract.target, Target::Enum);
        }
    }
    Ok(())
}

#[test]
fn non_exhaustive_owners_forms_duplicates_and_unknown_attributes_stay_checked() -> Result {
    for name in [
        "wrong-function",
        "wrong-union",
        "wrong-trait",
        "wrong-field",
        "wrong-crate",
    ] {
        let report = crate::resolve::resolve(&source(name)?, true)?;
        let error = rustgm::pipeline::analyze_source(&report)
            .err()
            .ok_or("invalid owner accepted")?;
        assert!(
            error
                .to_string()
                .contains("non_exhaustive attribute requires a struct, enum or enum variant"),
            "{name}: {error}"
        );
    }
    for name in [
        "empty-list",
        "argument-list",
        "name-value",
        "unsafe-wrapper",
    ] {
        let report = crate::resolve::resolve(&source(name)?, true)?;
        let error = rustgm::pipeline::analyze_source(&report)
            .err()
            .ok_or("invalid form accepted")?;
        assert!(
            error
                .to_string()
                .contains("non_exhaustive attribute requires word form"),
            "{name}: {error}"
        );
    }
    let report = crate::resolve::resolve(&source("duplicate-default")?, true)?;
    let analysis = rustgm::pipeline::analyze_source(&report)?;
    let warnings: Vec<_> = analysis.facts.warnings().collect();
    assert_eq!(warnings.len(), 1);
    assert_eq!(warnings[0].code.name(), "unused_attributes");
    assert_eq!(
        warnings[0].diagnostic.message,
        "duplicate non_exhaustive attribute"
    );
    let report = crate::resolve::resolve(&source("duplicate-deny")?, true)?;
    let error = rustgm::pipeline::analyze_source(&report)
        .err()
        .ok_or("denied duplicate accepted")?;
    assert!(
        error
            .to_string()
            .contains("duplicate non_exhaustive attribute")
    );
    let error = crate::resolve::resolve(&source("unknown-attribute")?, true)
        .err()
        .ok_or("unknown attribute accepted")?;
    assert!(error.contains("unresolved attribute macro"), "{error}");
    Ok(())
}

#[test]
fn non_exhaustive_contracts_reject_a_different_current_source_image() -> Result {
    let text = source("same-crate-structs")?;
    let report = crate::resolve::resolve(&text, true)?;
    let analysis = rustgm::pipeline::analyze_source(&report)?;
    let other = crate::resolve::resolve(&text, true)?;
    let definition = report
        .resolution
        .definitions
        .iter()
        .find(|definition| report.package.names.get(definition.name) == "Named")
        .and_then(|definition| definition.node)
        .ok_or("missing actual Named declaration")?;
    assert!(
        analysis
            .facts
            .non_exhaustive_contract(&report, definition)?
            .is_some()
    );
    let error = analysis
        .facts
        .non_exhaustive_contract(&other, definition)
        .err()
        .ok_or("foreign report image accepted")?;
    assert!(error.to_string().contains("actual source owner changed"));
    Ok(())
}
