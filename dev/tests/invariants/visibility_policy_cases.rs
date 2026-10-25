//! The real lexical lint policy owns severity and the shared error carrier.
use super::*;
fn diagnostics<'a>(warnings: impl Iterator<Item = &'a Warning>) -> Vec<String> {
    warnings
        .map(|warning| format!("warning[{}]: {}\n", warning.code.name(), warning.diagnostic))
        .collect()
}
#[test]
fn visibility_levels_origins_and_mixed_failures_preserve_exact_warning_contracts() -> TestResult {
    for (name, count) in [
        ("pub-repeat", 2),
        ("visibility-allow", 0),
        ("visibility-warn", 1),
        ("visibility-root-deny-local-allow", 0),
        ("visibility-root-allow-local-warn", 1),
        ("visibility-macro-default", 1),
        ("visibility-macro-allow", 0),
        ("visibility-macro-forwarded", 1),
        ("literal-many-calls", 2),
        ("visibility-restricted", 1),
    ] {
        let source = fixture(25, name)?;
        let found = report(&source)?;
        let first = pipeline::analyze_source(&found)?;
        let warnings: Vec<_> = first.facts.warnings().collect();
        assert_eq!(warnings.len(), count, "{name}");
        assert!(
            warnings
                .iter()
                .all(|warning| warning.code == WarningCode::UnusedVisibilities
                    && warning.diagnostic.message == MESSAGE)
        );
        let before = found.lint_producer().map(Producer::work);
        let first_view = diagnostics(first.facts.warnings());
        assert_eq!(first_view, diagnostics(first.facts.warnings()));
        assert_eq!(
            before.map(|work| work.steps),
            found.lint_producer().map(|producer| producer.work().steps)
        );
        let again = pipeline::analyze_source(&found)?;
        assert_eq!(first_view, diagnostics(again.facts.warnings()), "{name}");
        if name == "pub-repeat" {
            let expected: Vec<_> = source
                .match_indices("pub const")
                .map(|(offset, _)| {
                    format!("warning[unused_visibilities]: file 0: {offset}: {MESSAGE}\n")
                })
                .collect();
            assert_eq!(first_view, expected);
        }
        if name.starts_with("visibility-macro") || name == "literal-many-calls" {
            assert!(
                warnings
                    .iter()
                    .all(|warning| !warning.diagnostic.expansions.is_empty())
            );
        }
    }
    for name in [
        "visibility-deny",
        "visibility-forbid",
        "visibility-forbid-weaken",
    ] {
        let error = report(&fixture(25, name)?)
            .err()
            .ok_or("denied visibility accepted")?;
        assert!(error.warnings().is_empty());
        assert!(
            error
                .message
                .contains(if name == "visibility-forbid-weaken" {
                    "cannot be weakened"
                } else {
                    MESSAGE
                })
        );
    }
    let source = fixture(25, "named-privacy")?;
    let error = report(&source)
        .err()
        .ok_or("private constant became visible")?;
    assert_eq!(error.message, "private value name `NAMED`");
    assert_eq!(error.warnings().len(), 1);
    assert_eq!(
        diagnostics(error.warnings().iter()),
        vec![format!(
            "warning[unused_visibilities]: file 0: {}: {MESSAGE}\n",
            source.find("pub const").ok_or("pub token")?
        )]
    );
    for name in ["visibility-invalid-type", "visibility-invalid-ctfe"] {
        let source = fixture(25, name)?;
        let found = report(&source)?;
        let error = pipeline::analyze_source(&found)
            .err()
            .ok_or("invalid initializer accepted")?;
        assert_eq!(error.warnings().len(), 1, "{name}");
        assert_eq!(
            diagnostics(error.warnings().iter()),
            vec![format!(
                "warning[unused_visibilities]: file 0: {}: {MESSAGE}\n",
                source.find("pub const").ok_or("pub token")?
            )]
        );
    }
    Ok(())
}
