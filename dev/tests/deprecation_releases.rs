//! Actual staged release metadata is independent of cap, body permission and expectation debt.
use rustgm::{
    expand::config::Config,
    pipeline,
    support::diagnostics::{LintCap, WarningCode},
};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const ROOT: &str = "pa40/tests/spec/future-deprecation";
fn report(name: &str, cap: bool) -> Result<rustgm::resolve::expansion::Report> {
    let mut config = Config::default();
    if cap {
        config.set_lint_cap(LintCap::Allow);
    }
    Ok(crate::resolve::source_provider_config(
        &crate::resolve::fixture(&format!("{ROOT}/{name}.rs"))?,
        "future_deprecation",
        &config,
    )?)
}
#[test]
fn future_and_numeric_since_follow_actual_staged_profile() -> Result {
    for name in [
        "ordinary-tbd",
        "staged-tbd",
        "staged-1-98-1",
        "staged-99-0-0",
        "active-allow",
        "active-expect",
        "macro-TBD",
    ] {
        let analysis = pipeline::analyze_source(&report(name, false)?)?;
        assert_eq!(
            analysis
                .facts
                .warnings()
                .filter(|warning| warning.code == WarningCode::Deprecated)
                .count(),
            0,
            "{name}"
        );
    }
    let analysis = pipeline::analyze_source(&report("active-warn", false)?)?;
    assert_eq!(
        analysis
            .facts
            .warnings()
            .filter(|warning| warning.code == WarningCode::Deprecated)
            .count(),
        1
    );
    Ok(())
}
#[test]
fn actual_active_deprecations_and_unfulfilled_expectations_remain() {
    let mut failures = Vec::new();
    for name in [
        "ordinary-1-0-0",
        "ordinary-1-98-0",
        "ordinary-1-98-1",
        "ordinary-99-0-0",
        "ordinary-current_rustc_version",
        "ordinary-soon",
        "staged-1-0-0",
        "staged-1-98-0",
        "staged-current_rustc_version",
        "active-deny",
        "macro-1-0-0",
        "future-expect-unfulfilled",
    ] {
        let result = report(name, false).and_then(|report| {
            pipeline::analyze_source(&report)
                .map(|_| ())
                .map_err(Into::into)
        });
        let message = result
            .err()
            .map_or_else(|| "ACCEPTED".to_owned(), |error| error.to_string());
        eprintln!("{name}: {message}");
        if !(message.contains("deprecated") || message.contains("unfulfilled")) {
            failures.push(format!("{name}: {message}"));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
#[test]
fn staged_invalid_metadata_is_rejected_before_optional_use_and_cap() -> Result {
    for name in ["future-deny-cap-allow", "active-deny-cap-allow"] {
        pipeline::analyze_source(&report(name, true)?)?;
    }
    let mut failures = Vec::new();
    for (name, cap, cause) in [
        (
            "invalid-staged-cap-allow",
            true,
            "valid version or exact TBD",
        ),
        ("staged-soon", false, "valid version or exact TBD"),
        ("staged-missing-since", false, "since field"),
        ("staged-missing-note", false, "note field"),
        ("duplicate-key", false, "unique named string"),
        ("nonstring", false, "string literal"),
        (
            "invalid-version-components",
            false,
            "valid version or exact TBD",
        ),
        ("lowercase-tbd", false, "valid version or exact TBD"),
        ("future-body-type-error", true, "incompatible types"),
        ("future-unsafe-call", true, "unsafe context"),
    ] {
        let result = report(name, cap).and_then(|report| {
            pipeline::analyze_source(&report)
                .map(|_| ())
                .map_err(Into::into)
        });
        let message = result
            .err()
            .map_or_else(|| "ACCEPTED".to_owned(), |error| error.to_string());
        eprintln!("{name}: {message}");
        if !message.contains(cause) || message.contains("not implemented") {
            failures.push(format!("{name}: {message}"));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}
#[test]
fn unused_staged_annotations_validate_even_without_optional_event() -> Result {
    for name in [
        "staged-tbd-unused",
        "staged-99-0-0-unused",
        "ordinary-soon-unused",
    ] {
        pipeline::analyze_source(&report(name, true)?)?;
    }
    for (name, cause) in [
        ("staged-soon-unused", "valid version or exact TBD"),
        ("staged-missing-since-unused", "since field"),
        ("staged-missing-note-unused", "note field"),
    ] {
        let error = report(name, true)
            .and_then(|report| {
                pipeline::analyze_source(&report)
                    .map(|_| ())
                    .map_err(Into::into)
            })
            .err()
            .ok_or_else(|| format!("invalid unused annotation accepted: {name}"))?
            .to_string();
        assert!(error.contains(cause), "{name}: {error}");
    }
    Ok(())
}
#[test]
#[ignore = "requires pinned CPPGM object backend and a C compiler"]
fn future_metadata_preserves_actual_ordinary_native_body() -> Result {
    let report = report("native-future", false)?;
    let pipeline::Analysis {
        mut facts,
        prepared,
    } = pipeline::analyze_source(&report)?;
    let program = rustgm::lower::instances::build_roots_prepared(
        &report,
        &mut facts,
        rustgm::lower::Roots {
            entry: None,
            exports: &[],
        },
        prepared,
    )?;
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let folder = root.join(format!(
        "obj/parallel/future-deprecation/native-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&folder)?;
    crate::ffi_program::check(
        &root,
        &folder,
        "future-deprecation",
        &program,
        &crate::resolve::fixture(&format!("{ROOT}/observer.c"))?,
        "future-deprecation 23\n",
    )?;
    Ok(())
}
