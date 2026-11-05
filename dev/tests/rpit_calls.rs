//! Required trait results retain their declared equalities and opaque identities.
use crate::{resolve as test_source, semantic};
type TestResult = Result<(), Box<dyn std::error::Error>>;
#[test]
fn required_result_equalities_keep_identity_and_lifetime_opponents() -> TestResult {
    let manifest = test_source::fixture("pa21/tests/spec/opaque/required-results/cases.tsv")?;
    let mut count = 0;
    let mut failures = Vec::new();
    for line in manifest.lines() {
        let (path, status) = line.split_once('\t').ok_or("case columns")?;
        let result = semantic::resolve(&test_source::fixture(path)?).and_then(|mut report| {
            let rustgm::pipeline::Analysis {
                mut facts,
                prepared,
            } = rustgm::pipeline::analyze_source(&report).map_err(|e| e.to_string())?;
            let entry =
                rustgm::sema::entry::select(&mut report, &facts).map_err(|e| e.to_string())?;
            rustgm::lower::instances::build_roots_prepared(
                &report,
                &mut facts,
                rustgm::lower::Roots {
                    entry: Some(entry),
                    exports: &[],
                },
                prepared,
            )
            .map_err(|e| e.to_string())
        });
        match (status, result) {
            ("0", Ok(_)) => {}
            ("0", Err(error)) => failures.push(format!("{path}: {error}")),
            ("1", Ok(_)) => failures.push(format!("accepted {path}")),
            ("1", Err(error)) => {
                if [
                    "invalid or exhausted",
                    "requires normalization",
                    "not implemented",
                ]
                .iter()
                .any(|fence| error.contains(fence))
                {
                    failures.push(format!("{path}: {error}"));
                }
            }
            _ => return Err("invalid outcome".into()),
        }
        count += 1;
    }
    assert_eq!(count, 6);
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}
