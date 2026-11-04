//! Independently predicted unsized parameter permission and ownership outcomes.
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
fn source_parameters_retain_ordinary_sized_move_borrow_and_capture_requirements() -> Result {
    let inventory = crate::test_source::fixture("pa26/tests/spec/unsized-parameters/cases-v2.tsv")?;
    let mut count = 0;
    let mut mismatches = Vec::new();
    for row in inventory.lines() {
        let fields: Vec<_> = row.split('\t').collect();
        let [name, path, status] = fields.as_slice() else {
            return Err("invalid unsized parameter inventory".into());
        };
        let source = crate::test_source::fixture(path)?;
        for edition in [Edition::Rust2021, Edition::Rust2024] {
            let mut config = Config::in_edition(edition);
            config.set_lint_cap(LintCap::Allow);
            let actual = (|| -> Result {
                let input = InputFile {
                    identity: "unsized.rs".into(),
                    path: "unsized.rs".into(),
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
            if *name == "reference-control" {
                // This independent Rust-positive API remains an explicit required boundary.
                // It is separate from the by-value parameter permission and ownership rules.
                let error = actual
                    .err()
                    .ok_or("review the str::len required boundary after implementation")?;
                assert!(
                    error
                        .to_string()
                        .contains("receiver has no inherent nominal method")
                );
                continue;
            }
            if actual.is_ok() != (*status == "0") {
                mismatches.push(format!("{name} {edition:?}: {:?}", actual.err()));
            }
        }
        count += 1;
    }
    assert_eq!(count, 32);
    assert!(mismatches.is_empty(), "{}", mismatches.join("\n"));
    Ok(())
}
