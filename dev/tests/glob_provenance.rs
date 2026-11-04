//! Import causes retain real source policy independently of earlier lineage debt.
use crate::resolve as test_source;
use rustgm::{pipeline, support::diagnostics::WarningCode};
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
fn fixture(name: &str) -> std::io::Result<String> {
    std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../pa40/tests/spec/hidden-glob-reexports")
            .join(format!("{name}.rs")),
    )
}
#[test]
fn real_shadow_namespace_and_import_owner_protocol() -> TestResult {
    for (name, expected) in [
        ("warning", vec![WarningCode::HiddenGlobReexports; 2]),
        ("expect", vec![]),
        ("single-import", vec![]),
        ("same-target", vec![]),
        ("original-namespaces", vec![]),
        (
            "wrong-owner",
            vec![
                WarningCode::HiddenGlobReexports,
                WarningCode::UnfulfilledLintExpectations,
            ],
        ),
        ("all-shadow-warning", vec![]),
        ("all-shadow-deny", vec![]),
        (
            "all-shadow-expect",
            vec![WarningCode::UnfulfilledLintExpectations],
        ),
        (
            "all-shadow-import",
            vec![WarningCode::UnfulfilledLintExpectations],
        ),
        ("no-glob", vec![WarningCode::UnfulfilledLintExpectations]),
        (
            "private-glob",
            vec![WarningCode::UnfulfilledLintExpectations],
        ),
        (
            "public-local",
            vec![WarningCode::UnfulfilledLintExpectations],
        ),
        ("macro-only", vec![WarningCode::UnfulfilledLintExpectations]),
        (
            "no-type-counterpart",
            vec![WarningCode::UnfulfilledLintExpectations],
        ),
        (
            "overridden-allow",
            vec![WarningCode::UnfulfilledLintExpectations],
        ),
    ] {
        let report = test_source::resolve(&fixture(name)?, true)?;
        let analysis = pipeline::analyze_source(&report)?;
        let codes: Vec<_> = analysis
            .facts
            .warnings()
            .map(|warning| warning.code)
            .collect();
        assert_eq!(codes, expected, "{name}");
    }
    for (name, message) in [
        ("deny", "private item shadows public glob"),
        ("forbid", "forbidden hidden_glob_reexports"),
    ] {
        let report = test_source::resolve(&fixture(name)?, true)?;
        let Err(error) = pipeline::analyze_source(&report) else {
            return Err(format!("accepted {name}").into());
        };
        assert!(error.to_string().contains(message), "{name}: {error}");
    }
    Ok(())
}
#[test]
fn effective_export_follows_selected_chains_not_ultimate_targets() -> TestResult {
    for (name, count) in [
        ("private-container", 1),
        ("exposed-import", 0),
        ("unrelated-export", 1),
        ("chain", 0),
    ] {
        let report = test_source::resolve(&fixture(name)?, true)?;
        let analysis = pipeline::analyze_source(&report)?;
        assert_eq!(analysis.facts.warnings().count(), count, "{name}");
    }
    Ok(())
}
#[test]
fn actual_large_lineage_keeps_its_owner_under_uncapped_and_allow_policies() -> TestResult {
    use rustgm::{
        expand::config::Config,
        resolve::model::{DefKind, Origin},
        support::diagnostics::LintCap,
    };
    use std::fmt::Write as _;
    let mut source = String::from("#![expect(hidden_glob_reexports)]\nmod base{");
    for index in 0..512 {
        write!(source, "pub mod leaf{index}{{}}")?;
    }
    source.push_str("}\n");
    for index in 0..512 {
        writeln!(
            source,
            "pub mod fanout{index}{{pub use super::base::*;mod leaf0{{}}}}"
        )?;
    }
    for index in 0..16_384 {
        writeln!(source, "#[doc=\"retained\"]pub fn owner{index}(){{}}")?;
    }
    for cap in [LintCap::Uncapped, LintCap::Allow] {
        let mut config = Config::default();
        config.set_lint_cap(cap);
        let report = test_source::resolve_config(&source, true, &config)?;
        assert_eq!(report.resolution.imports.len(), 512);
        for (index, import) in report.resolution.imports.iter().enumerate() {
            assert!(import.glob);
            let origin = Origin::Glob(u32::try_from(index)?);
            assert_eq!(
                report
                    .resolution
                    .bindings
                    .scope(import.scope)
                    .filter(|(_, _, binding)| binding.origin == origin)
                    .count(),
                511
            );
        }
        assert_eq!(
            report
                .resolution
                .definitions
                .iter()
                .filter(|d| d.kind == DefKind::Function)
                .count(),
            16_384
        );
        // This known producer keeps its original expectation identity. Only
        // unsupported optional producers enter the unnumbered deferred count.
        assert_eq!(report.deferred_lint_expectations()?, 0);
        let facts = pipeline::analyze_source(&report)?.facts;
        assert!(facts.warnings().next().is_none());
    }
    Ok(())
}
