//! Mandatory raw-autoref lint policy consumes actual generated Field borrows.
use crate::resolve as source;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

#[test]
fn raw_field_lint_warn_expect_allow_and_caps_use_the_actual_lexical_policy() -> Result {
    let manifest = source::fixture("pa40/tests/spec/source-field-autorefs/cases.tsv")?;
    for line in manifest.lines() {
        let (name, status) = line.split_once('\t').ok_or("autoref case columns")?;
        let text = source::fixture(&format!("pa40/tests/spec/source-field-autorefs/{name}"))?;
        let result = source::source_provider(&text, "raw_policy").and_then(|report| {
            rustgm::pipeline::analyze_source(&report)
                .map(|_| ())
                .map_err(|e| e.to_string())
        });
        let cause = match name {
            "raw-deny.rs" => "implicit autoref creates a reference",
            "raw-forbid-allow.rs" => "forbidden dangerous_implicit_autorefs",
            "raw-expect-missing.rs" => "unfulfilled",
            _ => "positive",
        };
        match (status, result) {
            ("0", Ok(())) => {}
            ("1", Err(error)) if error.contains(cause) => {}
            (_, result) => return Err(format!("{name}: {result:?}").into()),
        }
    }
    for (name, warns) in [
        ("raw-warn", true),
        ("raw-allow", false),
        ("raw-expect", false),
    ] {
        let text = source::fixture(&format!("pa40/tests/spec/source-field-autorefs/{name}.rs"))?;
        let report = source::source_provider(&text, "raw_field_lint")?;
        let analysis = rustgm::pipeline::analyze_source(&report)?;
        assert_eq!(
            analysis
                .facts
                .warnings()
                .any(|warning| warning.code.name() == "dangerous_implicit_autorefs"),
            warns,
            "{name}"
        );
    }
    let text = source::fixture("pa31/tests/spec/source-field-deref/raw-explicit.rs")?;
    let mut config = rustgm::expand::config::Config::default();
    config.set_lint_cap(rustgm::support::diagnostics::LintCap::Allow);
    let report = source::source_provider_config(&text, "raw_capped", &config)?;
    let analysis = rustgm::pipeline::analyze_source(&report)?;
    assert!(
        !analysis
            .facts
            .warnings()
            .any(|warning| warning.code.name() == "dangerous_implicit_autorefs")
    );
    Ok(())
}
