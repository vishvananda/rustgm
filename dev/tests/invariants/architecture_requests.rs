//! Foreign language requests cannot authorize selected-target attributes or operations.
use crate::{
    expand::config::Config,
    resolve::{
        self,
        expansion::Report,
        load::{InputFile, Sources},
    },
    support::{diagnostics::LintCap, edition::Edition, source::SourceFile},
};
use std::path::Path;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
struct EmptySources;
impl Sources for EmptySources {
    fn read(&mut self, _: &Path) -> std::result::Result<Option<InputFile>, String> {
        Ok(None)
    }
}
fn report(source: &str, edition: Edition) -> Result<Report> {
    let mut config = Config::in_edition(edition);
    config.set_lint_cap(LintCap::Allow);
    let input = InputFile {
        identity: "target-requests.rs".into(),
        path: "target-requests.rs".into(),
        source: SourceFile::new(source.as_bytes().to_vec())?,
    };
    Ok(resolve::analyze_expanded(resolve::expansion::analyze(
        input,
        &mut EmptySources,
        &config,
    )?)?)
}
#[test]
fn request_family_preserves_selected_target_safety_and_original_body_checks() -> Result {
    let inventory =
        crate::test_source::fixture("pa41/tests/spec/architecture-requests/cases-v1.tsv")?;
    let mut count = 0;
    let mut mismatches = Vec::new();
    for row in inventory.lines() {
        let fields: Vec<_> = row.split('\t').collect();
        let [name, path, status] = fields.as_slice() else {
            return Err("invalid architecture request inventory".into());
        };
        let source = crate::test_source::fixture(path)?;
        for edition in [Edition::Rust2021, Edition::Rust2024] {
            let actual = (|| -> Result {
                let report = report(&source, edition)?;
                let mut analysis = crate::pipeline::analyze_source(&report)?;
                crate::lower::instances::inspect_prepared(
                    &report,
                    &mut analysis.facts,
                    analysis.prepared,
                    |_, _| (),
                )?;
                Ok(())
            })();
            if actual.is_ok() != (*status == "0") {
                mismatches.push(format!("{name} {edition:?}: {:?}", actual.err()));
            }
        }
        count += 1;
    }
    assert_eq!(count, 56);
    assert!(mismatches.is_empty(), "{}", mismatches.join("\n"));
    Ok(())
}
#[test]
fn foreign_requests_retain_no_cpu_permissions_or_ordinary_binding_owner() -> Result {
    let source =
        crate::test_source::fixture("pa41/tests/spec/architecture-requests/all-foreign.rs")?;
    let ordinary = report(&source, Edition::Rust2024)?;
    assert!(!ordinary.package.uses_source_bindings());
    assert!(ordinary.package.macro_context()?.is_none());
    assert!(ordinary.package.library_policy().is_none());
    let provider = crate::test_source::source_provider(
        &format!("#![no_core]#![feature(no_core)]{source}"),
        "foreign_requests",
    )?;
    assert!(provider.package.source_core().is_some());
    assert!(provider.package.library_policy().is_none());
    for report in [&ordinary, &provider] {
        for gate in crate::support::cpu_features::Gate::ALL {
            assert!(!report.package.requested_cpu_gate(gate)?);
        }
    }
    assert_eq!(size_of::<super::Features>(), 16);
    assert_eq!(size_of::<super::MacroFeatures>(), 1);
    assert_eq!(size_of::<crate::resolve::bindings::SourceCore>(), 200);
    assert_eq!(size_of::<crate::resolve::package::Package>(), 256);
    Ok(())
}
