//! Resolver history stays paid; diagnostic allocation is independently prepaid.
use super::*;
use crate::{
    expand::config::Config,
    resolve::{
        expansion,
        load::{InputFile, Sources},
    },
    support::{diagnostics::LintCap, source::SourceFile},
};
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
struct NoFiles;
impl Sources for NoFiles {
    fn read(&mut self, _: &std::path::Path) -> Result<Option<InputFile>, String> {
        Ok(None)
    }
}
fn expanded(source: &str, cap: LintCap) -> TestResult<Report> {
    let mut config = Config::default();
    config.set_lint_cap(cap);
    Ok(expansion::analyze(
        InputFile {
            identity: "lib.rs".into(),
            path: "lib.rs".into(),
            source: SourceFile::new(source.as_bytes().to_vec())?,
        },
        &mut NoFiles,
        &config,
    )?)
}
fn source() -> std::io::Result<String> {
    crate::test_source::fixture("pa40/tests/spec/hidden-glob-reexports/warning.rs")
}
fn root(report: &Report) -> NodeKey {
    NodeKey {
        file: report.package.modules[0].file,
        node: report.package.modules[0].root,
    }
}
#[test]
fn distinct_paid_histories_do_not_become_one_smaller_budget() -> TestResult {
    let mut report = expanded(&source()?, LintCap::Uncapped)?;
    let key = root(&report);
    let mut work = report
        .imports
        .take()
        .ok_or("actual completed import work")?;
    let producer = report.lint_producer().ok_or("source policy owner")?;
    producer
        .diagnostic_read(key, 40 * 1024 * 1024)
        .map_err(Error::from)?;
    let before = producer.work();
    work.lineage_budget
        .charge(key, 3_000_000, 40 * 1024 * 1024)
        .map_err(Error::from)?;
    let lineage = work.lineage_budget;
    let candidates = work.hidden_candidates(&report.resolution, key)?;
    let causes = work.hidden_causes(
        &report.package,
        &report.resolution,
        key,
        candidates,
        producer,
    )?;
    assert_eq!(causes.len(), 2);
    assert!(work.lineage_budget.steps > lineage.steps);
    assert!(work.lineage_budget.bytes >= lineage.bytes);
    let after = producer.work();
    assert!(after.bytes > before.bytes && after.bytes - before.bytes < 4096);
    assert!(after.steps > before.steps && after.steps - before.steps < 100);
    for cause in causes {
        cause.site(&report, producer)?;
    }
    assert!(producer.hidden_globs.is_none());
    Ok(())
}
#[test]
fn failed_new_diagnostic_allocations_are_sticky_and_publish_no_prefix() -> TestResult {
    let mut report = expanded(&source()?, LintCap::Uncapped)?;
    let key = root(&report);
    let mut work = report.imports.take().ok_or("actual import work")?;
    let producer = report.lint_producer().ok_or("source policy owner")?;
    producer
        // Amended: the lint limits include the admitted-source grant.
        .diagnostic_read(key, producer.work().limits().1 - producer.work().bytes)
        .map_err(Error::from)?;
    for _ in 0..2 {
        let before = producer.work();
        let candidates = work.hidden_candidates(&report.resolution, key)?;
        assert!(
            work.hidden_causes(
                &report.package,
                &report.resolution,
                key,
                candidates,
                producer
            )
            .is_err()
        );
        assert!(producer.work().bytes > before.bytes);
        assert!(producer.hidden_globs.is_none());
    }
    assert!(producer.diagnostic_read(key, 0).is_err());
    Ok(())
}
#[test]
fn completion_checks_both_failed_owners_and_the_actual_cap_before_empty_results() -> TestResult {
    for failed_imports in [false, true] {
        let mut report = expanded("#[doc=\"retained\"]fn ordinary(){}", LintCap::Allow)?;
        let key = root(&report);
        let mut work = Work::new(&report.resolution);
        if failed_imports {
            assert!(
                work.lineage_budget
                    .charge(key, 1, 64 * 1024 * 1024 + 1)
                    .is_err()
            );
        } else {
            // Amended: the lint limits include the admitted-source grant.
            let producer = report.lint_producer().ok_or("source policy")?;
            assert!(
                producer
                    .diagnostic_read(key, producer.work().limits().1 + 1)
                    .is_err()
            );
        }
        assert!(report.complete_hidden_globs(work).is_err());
    }
    let mut report = expanded(&source()?, LintCap::Allow)?;
    let work = report.imports.take().ok_or("actual import work")?;
    report.package.lint_cap = LintCap::Uncapped;
    let error = report
        .complete_hidden_globs(work)
        .err()
        .ok_or("changed cap accepted")?;
    assert!(
        error
            .to_string()
            .contains("changed its actual compilation cap")
    );
    Ok(())
}
