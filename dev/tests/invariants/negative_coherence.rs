//! Actual negative evidence, frozen overlap inputs and strict-owner authority.
use crate::{
    expand::config::Config,
    resolve::{
        self,
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
#[test]
fn source_coherence_matches_independent_permission_and_refutation_outcomes() -> Result {
    let inventory = crate::test_source::fixture("pa31/tests/spec/negative-coherence/cases-v6.tsv")?;
    let mut count = 0;
    let mut mismatches = Vec::new();
    for row in inventory.lines() {
        let fields: Vec<_> = row.split('\t').collect();
        let [name, path, status] = fields.as_slice() else {
            return Err("invalid coherence inventory".into());
        };
        let source = crate::test_source::fixture(path)?;
        for edition in [Edition::Rust2021, Edition::Rust2024] {
            let mut config = Config::in_edition(edition);
            config.set_lint_cap(LintCap::Allow);
            let actual = (|| -> Result {
                let input = InputFile {
                    identity: "coherence.rs".into(),
                    path: "coherence.rs".into(),
                    source: SourceFile::new(source.as_bytes().to_vec())?,
                };
                let report = resolve::analyze_expanded(resolve::expansion::analyze(
                    input,
                    &mut NoFiles,
                    &config,
                )?)?;
                let analysis = crate::pipeline::analyze_source(&report)?;
                let mut facts = analysis.facts;
                crate::lower::instances::inspect_prepared(
                    &report,
                    &mut facts,
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
    assert_eq!(count, 41);
    assert!(mismatches.is_empty(), "{}", mismatches.join("\n"));
    Ok(())
}

#[test]
fn strict_owner_cannot_lose_its_policy_in_existing_metadata() -> Result {
    let source =
        crate::test_source::fixture("pa31/tests/spec/negative-coherence/strict-negative.rs")?;
    let report = crate::test_source::resolve(&source, true)?;
    let mut facts = crate::pipeline::analyze_source(&report)?.facts;
    let error = crate::sema::crates::export(
        &report,
        &mut facts,
        crate::support::crates::CrateIdentity::new("coherence", "owned")?,
    )
    .err()
    .ok_or("strict owner policy erased by metadata")?
    .to_string();
    assert!(
        error.contains("strict coherence owners require authenticated metadata transport"),
        "{error}"
    );
    Ok(())
}
