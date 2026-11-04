//! Documentation and optional lint requests never authorize pointer operations.
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
struct NoFiles;
impl Sources for NoFiles {
    fn read(&mut self, _: &Path) -> std::result::Result<Option<InputFile>, String> {
        Ok(None)
    }
}
fn report(source: &str, config: &Config, provider: bool) -> Result<Report> {
    if provider {
        return Ok(crate::test_source::source_provider_config(
            source, "advisory", config,
        )?);
    }
    Ok(resolve::analyze_expanded(resolve::expansion::analyze(
        InputFile {
            identity: "advisory.rs".into(),
            path: "advisory.rs".into(),
            source: SourceFile::new(source.as_bytes().to_vec())?,
        },
        &mut NoFiles,
        config,
    )?)?)
}
#[test]
fn advisory_requests_preserve_independent_source_outcomes_and_strict_boundaries() -> Result {
    let cases = crate::test_source::fixture("pa40/tests/spec/advisory-requests/cases-v2.tsv")?;
    let mut count = 0;
    for row in cases.lines() {
        let columns: Vec<_> = row.split('\t').collect();
        let [name, path, status, cap, provider] = columns.as_slice() else {
            return Err("malformed advisory request case".into());
        };
        let source = crate::test_source::fixture(path)?;
        for edition in [Edition::Rust2021, Edition::Rust2024] {
            let mut config = Config::in_edition(edition);
            if *cap == "allow" {
                config.set_lint_cap(LintCap::Allow);
            }
            let actual = (|| -> Result {
                let report = report(&source, &config, *provider == "1")?;
                crate::pipeline::analyze_source(&report)?;
                Ok(())
            })();
            assert_eq!(
                actual.is_ok(),
                *status == "0",
                "{name} {edition:?}: {:?}",
                actual.err()
            );
        }
        count += 1;
    }
    assert_eq!(count, 36);
    Ok(())
}
#[test]
fn advisory_requests_leave_language_feature_word_and_binding_mode_unchanged() -> Result {
    assert_eq!(size_of::<super::Features>(), 8);
    let source = "#![feature(intra_doc_pointers,strict_provenance_lints)]pub fn answer()->u8{7}";
    let report = report(source, &Config::default(), false)?;
    assert!(!report.package.uses_source_bindings());
    assert!(report.package.macro_context()?.is_none());
    let facts = crate::sema::declarations::analyze(&report)?;
    crate::pipeline::analyze(&report, facts)?;
    Ok(())
}
