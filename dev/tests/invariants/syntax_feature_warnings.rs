//! The pinned direct diagnostic is one primary warning, irrespective of lint levels.
use super::*;
#[test]
fn unvisited_default_syntax_preserves_exact_unconditional_and_mixed_diagnostics() -> TestResult {
    for name in [
        "inactive-root-allow",
        "inactive-root-warn",
        "inactive-root-deny",
        "inactive-root-forbid",
        "inactive-root-deny-local-allow",
        "inactive-root-allow-local-warn",
        "inactive-root-forbid-local-allow",
    ] {
        let text = fixture("soft-authority", name)?;
        let found = report(&text)?;
        let analysis = pipeline::analyze_source(&found)?;
        let warnings: Vec<_> = analysis.facts.warnings().collect();
        assert_eq!(warnings.len(), 1, "{name}");
        let warning = warnings[0];
        assert_eq!(warning.code, WarningCode::UnstableSyntaxPreExpansion);
        assert_eq!(warning.diagnostic.message, MESSAGE);
        assert_eq!(
            warning.diagnostic.span.start as usize,
            text.find("default fn").ok_or("actual keyword")?
        );
        assert_eq!(
            format!("warning[{}]: {}\n", warning.code.name(), warning.diagnostic),
            format!(
                "warning[unstable_syntax_pre_expansion]: file 0: {}: specialization is experimental\n",
                text.find("default fn").ok_or("keyword")?
            )
        );
        assert!(
            found
                .resolution
                .definitions
                .iter()
                .all(|definition| definition.node.is_none_or(|key| found
                    .package
                    .unit(key.file)
                    .configured
                    .ast
                    .defaultness
                    .as_ref()
                    .is_none_or(|entries| entries
                        .entries()
                        .iter()
                        .all(|row| row.node != key.node))))
        );
        let again = pipeline::analyze_source(&found)?;
        assert_eq!(
            again.facts.warnings().count(),
            1,
            "read-only replay duplicated a warning"
        );
    }
    for (name, count) in [
        ("inactive-min-literal-allowance", 0),
        ("inactive-min-forwarded-allowance", 1),
    ] {
        let found = resolved("soft-edges", name)?;
        let analysis = pipeline::analyze_source(&found)?;
        let warnings: Vec<_> = analysis.facts.warnings().collect();
        assert_eq!(warnings.len(), count, "{name}");
        for warning in warnings {
            assert_eq!(warning.code, WarningCode::UnstableSyntaxPreExpansion);
            assert_eq!(warning.diagnostic.message, MESSAGE);
        }
    }
    let text = fixture("oracle-v1", "unexecuted-no-feature-demand")?;
    let found = report(&text)?;
    assert!(
        found
            .package
            .units
            .iter()
            .all(|unit| unit.configured.ast.defaultness.is_none())
    );
    assert!(
        pipeline::analyze_source(&found)?
            .facts
            .warnings()
            .next()
            .is_none(),
        "unused_macros is a separately retained coverage gap"
    );
    mixed_failures_retain_warning()
}
fn mixed_failures_retain_warning() -> TestResult {
    let text = fixture("soft-authority", "inactive-root-allow")?;
    let source = format!("{text}\nfn bad(){{missing();}}\n");
    let error = report(&source)
        .err()
        .ok_or("unresolved body name accepted")?;
    assert!(error.message.contains("missing"));
    assert_eq!(error.warnings().len(), 1);
    assert_eq!(
        error.warnings()[0].code,
        WarningCode::UnstableSyntaxPreExpansion
    );
    let source = format!("{text}\nfn bad()->u8{{false}}\n");
    let found = report(&source)?;
    let error = pipeline::analyze_source(&found)
        .err()
        .ok_or("bad body accepted")?;
    assert_eq!(error.warnings().len(), 1);
    assert_eq!(error.warnings()[0].diagnostic.message, MESSAGE);
    Ok(())
}
