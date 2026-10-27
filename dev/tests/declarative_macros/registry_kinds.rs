//! Registry kind joins preserve source syntax while selecting the actual macro namespace.
use crate::provider_traits::{TestResult, report_in_edition};
use rustgm::{expand::builtin::Builtin, resolve, support::edition::Edition};
fn source(name: &str) -> TestResult<String> {
    Ok(std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../pa31/tests/spec/scoped-builtin-declarations")
            .join(format!("{name}.rs")),
    )?)
}
#[test]
fn source_form_and_registry_kind_join_without_lending_execution_or_feature_authority() -> TestResult
{
    for edition in [Edition::Rust2021, Edition::Rust2024] {
        for name in [
            "legacy-test",
            "legacy-attribute-shadow",
            "legacy-bang-shadow",
            "registry-matcher",
            "iter-declaration",
        ] {
            let report = report_in_edition(&source(name)?, edition)?;
            if name == "iter-declaration" {
                assert!(report.definitions.iter().any(|definition| matches!(
                    definition.body,
                    resolve::expansion::Body::Builtin(Builtin::Iter)
                )));
            }
            rustgm::pipeline::analyze_source(&report)
                .map_err(|error| format!("{name}/{edition:?}: {error}"))?;
        }
        for (name, message) in [
            ("missing-declaration-feature", "decl_macro"),
            ("malformed-source-matcher", "fragment"),
            ("unknown-source-role", "builtin"),
        ] {
            let error = report_in_edition(&source(name)?, edition)
                .err()
                .ok_or_else(|| format!("{name}: invalid declaration accepted"))?;
            assert!(
                error.to_string().contains(message),
                "{name}/{edition:?}: {error}"
            );
        }
        let original = source("iter-declaration")?;
        let error = report_in_edition(&(original + "\nfn unavailable(){iter!();}\n"), edition)
            .err()
            .ok_or("unimplemented Iter expansion accepted")?;
        assert!(error.to_string().contains("not implemented"), "{error}");
    }
    Ok(())
}
