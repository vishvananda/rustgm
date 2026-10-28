//! Explicit caps suppress diagnostics while the ordinary checker remains active.
use super::*;
use crate::{
    expand::config::Config,
    resolve::{
        self,
        load::{InputFile, Sources},
    },
    support::{diagnostics::LintCap, source::SourceFile},
};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
struct NoFiles;
impl Sources for NoFiles {
    fn read(&mut self, _: &std::path::Path) -> std::result::Result<Option<InputFile>, String> {
        Ok(None)
    }
}
fn fixture(name: &str) -> Result<String> {
    Ok(crate::test_source::fixture(&format!(
        "pa40/tests/spec/lint-cap-allow/{name}.rs"
    ))?)
}
fn resolved(source: &str, cap: LintCap, source_mode: bool) -> Result<Report> {
    let mut config = Config::default();
    config.set_lint_cap(cap);
    if source_mode {
        return Ok(crate::test_source::source_provider_config(
            source, "lint_cap", &config,
        )?);
    }
    let expanded = resolve::expansion::analyze(
        InputFile {
            identity: "lib.rs".into(),
            path: "lib.rs".into(),
            source: SourceFile::new(source.as_bytes().to_vec())?,
        },
        &mut NoFiles,
        &config,
    )?;
    Ok(resolve::analyze_expanded(expanded)?)
}
#[test]
fn capped_replacement_hints_still_require_the_actual_language_feature() -> Result {
    for name in [
        "accepted",
        "used",
        "missing-feature",
        "wrong-value",
        "duplicate",
        "macro-allowance",
    ] {
        let source = crate::test_source::fixture(&format!(
            "pa40/tests/spec/deprecation-suggestion/{name}.rs"
        ))?;
        let result = match resolved(&source, LintCap::Allow, false) {
            Ok(report) => crate::pipeline::analyze_source(&report)
                .map(|analysis| {
                    assert!(analysis.facts.warnings().next().is_none(), "{name}");
                })
                .map_err(|error| error.to_string()),
            Err(error) => Err(error.to_string()),
        };
        if matches!(name, "accepted" | "used") {
            result?;
        } else {
            let error = result.err().ok_or("capped invalid replacement hint accepted")?;
            let expected = match name {
                "wrong-value" => "string",
                "duplicate" => "unique",
                _ => "requires deprecated_suggestion",
            };
            assert!(error.contains(expected), "{name}: {error}");
        }
    }
    Ok(())
}
#[test]
fn explicit_cap_suppresses_only_lints_in_real_ordinary_and_source_inputs() -> Result {
    for source_mode in [false, true] {
        for name in [
            "unknown-expect",
            "unimplemented-expect",
            "known-empty",
            "known-event",
            "deny",
            "forbid",
            "forbid-weakening",
        ] {
            let name = if source_mode {
                format!("source-{name}")
            } else {
                name.into()
            };
            let source = fixture(&name)?;
            let report = resolved(&source, LintCap::Allow, source_mode)?;
            let analysis =
                crate::pipeline::analyze_source(&report).map_err(|e| format!("{name}: {e}"))?;
            assert!(analysis.facts.warnings().next().is_none(), "{name}");
            assert_eq!(
                report.deferred_lint_expectations()?,
                u32::from(
                    name.ends_with("unknown-expect") || name.ends_with("unimplemented-expect")
                ),
                "{name}"
            );
            if name.ends_with("known-event") || name.ends_with("known-empty") {
                let producer = report.lint_producer().ok_or("producer")?;
                let progress = analysis
                    .facts
                    .safety_policies
                    .progress
                    .as_deref()
                    .ok_or("request progress")?;
                // Effective Allow neither invents an event nor closes a strict expectation.
                assert_eq!(
                    producer
                        .pending(&report.package, progress)
                        .map_err(Error::from)?
                        .len(),
                    1
                );
            }
        }
    }
    for name in ["unknown-expect", "unimplemented-expect"] {
        assert!(
            resolved(&fixture(name)?, LintCap::Uncapped, false).is_err(),
            "strict {name}"
        );
    }
    for name in ["deny", "forbid", "forbid-weakening"] {
        let report = resolved(&fixture(name)?, LintCap::Uncapped, false)?;
        assert!(
            crate::pipeline::analyze_source(&report).is_err(),
            "strict {name}"
        );
    }
    let report = resolved(&fixture("unsafe-operation-lint")?, LintCap::Allow, false)?;
    assert!(
        crate::pipeline::analyze_source(&report)?
            .facts
            .warnings()
            .next()
            .is_none()
    );
    let source = fixture("default-static-reference")?;
    let report = resolved(&source, LintCap::Allow, false)?;
    assert!(report.lint_producer().is_none());
    let analysis = crate::pipeline::analyze_source(&report)?;
    assert!(analysis.facts.warnings().next().is_none());
    assert_eq!(analysis.facts.retained_policy_storage(), (0, 0, 0));
    Ok(())
}
#[test]
fn cap_keeps_hard_type_feature_syntax_tool_and_unsafe_errors() -> Result {
    for name in [
        "hard-type",
        "hard-unsafe",
        "hard-feature",
        "hard-feature-name",
        "hard-syntax",
        "hard-ctfe",
        "unknown-tool",
        "malformed-reason",
        "source-hard-type",
        "source-hard-unsafe",
    ] {
        let source = fixture(name)?;
        let rejected = match resolved(&source, LintCap::Allow, name.starts_with("source-")) {
            Ok(report) => crate::pipeline::analyze_source(&report).is_err(),
            Err(_) => true,
        };
        assert!(rejected, "cap granted hard semantic permission: {name}");
    }
    Ok(())
}
#[test]
fn capped_declaration_facts_and_progress_cannot_be_reused_with_strict_source() -> Result {
    let source = fixture("known-empty")?;
    let capped = resolved(&source, LintCap::Allow, false)?;
    let facts = crate::sema::declarations::analyze(&capped)?;
    let strict = resolved(&source, LintCap::Uncapped, false)?;
    let error = crate::sema::bodies::analyze(&strict, facts)
        .err()
        .ok_or("capped facts reused under strict source")?;
    assert!(error.to_string().contains("compilation cap"));
    Ok(())
}
