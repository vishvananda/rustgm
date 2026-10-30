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
            let error = result
                .err()
                .ok_or("capped invalid replacement hint accepted")?;
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

#[test]
fn capped_macro_deprecation_work_is_linear_in_headers_and_calls() -> Result {
    let mut previous = None;
    for width in [8, 32, 128] {
        let mut source = String::new();
        for _ in 0..width {
            source.push_str("#![doc=\"retained owner data\"]\n");
        }
        source.push_str("macro_rules! plain{()=>{0u32}}pub fn observed()->u32{");
        for _ in 0..width {
            source.push_str("let _=plain!();");
        }
        source.push_str("7}");
        let report = resolved(&source, LintCap::Allow, false)?;
        let work = report.lint_producer().ok_or("actual producer")?.work();
        println!(
            "capped macro width={width} work={} bytes={} owners={} calls={}",
            work.steps, work.bytes, work.owners, work.calls
        );
        if let Some(before) = previous {
            assert!(
                work.steps < before * 6,
                "repeated owner validation is quadratic"
            );
        }
        previous = Some(work.steps);
        assert!(report.macro_deprecation_warnings()?.is_empty());
        assert!(
            crate::pipeline::analyze_source(&report)?
                .facts
                .warnings()
                .next()
                .is_none()
        );
    }
    Ok(())
}

#[test]
fn capped_macro_calls_keep_expectations_pending_and_strict_events_real() -> Result {
    for name in ["macro-invocation", "macro-reexport"] {
        let source =
            crate::test_source::fixture(&format!("pa40/tests/spec/lint-expectations/{name}.rs"))?;
        let capped = resolved(&source, LintCap::Allow, false)?;
        assert!(capped.macro_deprecation_warnings()?.is_empty());
        let analysis = crate::pipeline::analyze_source(&capped)?;
        assert!(analysis.facts.warnings().next().is_none());
        let progress = analysis
            .facts
            .safety_policies
            .progress
            .as_deref()
            .ok_or("actual expectation progress")?;
        assert_eq!(
            capped
                .lint_producer()
                .ok_or("actual producer")?
                .pending(&capped.package, progress)
                .map_err(Error::from)?
                .len(),
            1,
            "a suppressed macro event cannot fulfill a strict expectation"
        );
        let strict = resolved(&source, LintCap::Uncapped, false)?;
        assert!(
            crate::pipeline::analyze_source(&strict)?
                .facts
                .warnings()
                .next()
                .is_none(),
            "the genuine strict event must still fulfill its expectation"
        );
    }
    Ok(())
}

#[test]
fn suppressed_macro_events_keep_deprecation_attribute_errors_hard() -> Result {
    for name in [
        "macro-non-string-note",
        "macro-unsafe-wrapper",
        "macro-duplicate-attributes",
    ] {
        for cap in [LintCap::Allow, LintCap::Uncapped] {
            let error = resolved(&fixture(name)?, cap, false)
                .err()
                .ok_or_else(|| format!("{name}: malformed deprecated attribute accepted"))?;
            let expected = if name == "macro-non-string-note" {
                "string"
            } else {
                "deprecated"
            };
            assert!(error.to_string().contains(expected), "{name}: {error}");
        }
    }
    Ok(())
}
