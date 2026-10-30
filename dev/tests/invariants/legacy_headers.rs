//! A source index contract never rewrites same-crate calls or grants const permission.
use super::*;
use crate::{pipeline, test_source};
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
#[path = "legacy_replays.rs"]
mod replays;
fn input(name: &str) -> TestResult<String> {
    Ok(test_source::fixture(&format!(
        "pa31/tests/spec/source-legacy-const/{name}.rs"
    ))?)
}
fn checked(name: &str) -> TestResult<(Report, pipeline::Analysis)> {
    let report = test_source::resolve(&input(name)?, true)?;
    let analysis = pipeline::analyze_source(&report).map_err(|e| format!("{name}: {e}"))?;
    Ok((report, analysis))
}
fn lower(
    (report, analysis): (Report, pipeline::Analysis),
) -> TestResult<crate::lowir::program::Program> {
    let pipeline::Analysis {
        mut facts,
        prepared,
    } = analysis;
    Ok(crate::lower::instances::build_roots_prepared(
        &report,
        &mut facts,
        crate::lower::Roots {
            entry: None,
            exports: &[],
        },
        prepared,
    )?)
}
fn function(report: &Report, name: &str) -> TestResult<DefId> {
    let (index, _) = report
        .resolution
        .definitions
        .iter()
        .enumerate()
        .find(|(_, row)| {
            row.kind == DefKind::Function && report.package.names.get(row.name) == name
        })
        .ok_or("actual free function")?;
    Ok(DefId(u32::try_from(index)?))
}
#[test]
fn genuine_local_headers_preserve_modern_const_and_function_item_calls() -> TestResult {
    for (name, positions) in [
        ("local-modern", &[0][..]),
        ("const-modern", &[0][..]),
        ("multiple", &[0, 2][..]),
        ("duplicate-positions", &[0, 0][..]),
        ("reordered-positions", &[2, 0][..]),
        ("wrapped-meta-index", &[0][..]),
        ("radix-trailing", &[0][..]),
        ("boolean-formal", &[0][..]),
        ("static-reference", &[1][..]),
        ("nested-higher-ranked", &[1][..]),
        ("alias-modern", &[0][..]),
        ("fnptr-modern", &[0][..]),
        ("unsafe-modern", &[0][..]),
        ("macro-allowance", &[0][..]),
    ] {
        let (report, analysis) = checked(name)?;
        let id = function(&report, "f")?;
        assert_eq!(
            analysis.facts.legacy_const_positions(&report, id)?,
            positions,
            "{name}"
        );
        lower((report, analysis)).map_err(|e| format!("{name}: {e}"))?;
    }
    lower(checked("cfg-removed")?)?;
    Ok(())
}
#[test]
fn invalid_header_shapes_owners_forms_and_gates_remain_hard_errors() -> TestResult {
    for name in [
        "wrong-type-formal",
        "wrong-lifetime-formal",
        "elided-lifetime",
        "wrong-count",
        "wrong-index",
        "no-formals",
        "inherent-denied",
        "foreign-denied",
        "empty-list",
        "word",
        "value",
        "suffix",
        "negative",
        "noninteger",
        "missing-gate",
        "duplicate",
    ] {
        let error = checked(name)
            .err()
            .ok_or_else(|| format!("{name}: invalid declaration accepted"))?;
        let text = error.to_string();
        assert!(
            text.contains("legacy") || text.contains("rustc_attrs"),
            "{name}: {text}"
        );
    }
    Ok(())
}
#[test]
fn same_crate_legacy_alias_pointer_const_and_unsafe_calls_get_no_new_permission() -> TestResult {
    for name in [
        "local-legacy-denied",
        "alias-legacy-denied",
        "fnptr-legacy-denied",
        "explicit-generics-no-rewrite",
        "ordinary-not-const",
        "unsafe-not-granted",
    ] {
        let error = checked(name)
            .and_then(lower)
            .err()
            .ok_or_else(|| format!("{name}: forbidden call accepted"))?;
        let text = error.to_string();
        if name == "ordinary-not-const" {
            assert!(text.contains("const"), "{text}");
        } else if name == "unsafe-not-granted" {
            assert!(text.contains("unsafe"), "{text}");
        }
    }
    Ok(())
}
#[test]
fn legacy_indices_are_fenced_before_any_current_source_function_publication() -> TestResult {
    let report = test_source::source_provider(&input("module-provider")?, "legacy_header_fence")?;
    let analysis = pipeline::analyze_source(&report)?;
    let id = function(&report, "marked_legacy")?;
    let error = analysis
        .facts
        .reject_function_advisory_transport(&report, id)
        .err()
        .ok_or("legacy source word lost its transport fence")?;
    assert!(
        error.to_string().contains("legacy const generic indices"),
        "{error}"
    );
    for profile in [
        crate::lower::instances::SourcePublication::ModuleContracts,
        crate::lower::instances::SourcePublication::ModuleAvailability,
        crate::lower::instances::SourcePublication::ModuleDefinitions,
        crate::lower::instances::SourcePublication::ModuleCallables,
    ] {
        assert!(
            crate::module_packets::exported_profile(
                &input("module-provider")?,
                profile,
                "legacy_header_fence"
            )
            .is_err()
        );
    }
    Ok(())
}
