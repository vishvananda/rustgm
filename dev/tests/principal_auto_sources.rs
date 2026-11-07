//! Source auto declarations, not principal spellings, classify object bounds.
use rustgm::{expand::config::Config, pipeline};
type Result = std::result::Result<(), Box<dyn std::error::Error>>;
const ROOT: &str = "pa31/tests/spec/source-object-autos";
/// Pinned Rust outcomes from `predictions-v1/v2.json`; `None` accepts.
const CASES: &[(&str, Option<&str>)] = &[
    ("principal-send.rs", None),
    ("v2/auto-first.rs", None),
    ("v2/auto-only.rs", None),
    ("v2/unpin-object.rs", None),
    ("demand-send.rs", None),
    ("supertrait-send.rs", None),
    ("v2/inherent-send.rs", None),
    (
        "two-principals.rs",
        Some("object requires exactly one principal trait"),
    ),
    (
        "ordinary-named-send.rs",
        Some("object requires exactly one principal trait"),
    ),
    (
        "generic-auto.rs",
        Some("auto traits do not accept generic arguments"),
    ),
    (
        "demand-missing.rs",
        Some("trait obligation requires an unambiguous matching assumption or implementation"),
    ),
    (
        "duplicate-object-impl.rs",
        Some("overlapping trait implementation headers require coherence proof"),
    ),
    (
        "v2/inherent-auto-only.rs",
        Some("inherent object impl requires a principal trait"),
    ),
];

fn analyze(report: &rustgm::resolve::expansion::Report) -> std::result::Result<(), String> {
    let pipeline::Analysis {
        mut facts,
        prepared,
        ..
    } = pipeline::analyze_source(report).map_err(|e| e.to_string())?;
    rustgm::lower::instances::build_roots_prepared(
        report,
        &mut facts,
        rustgm::lower::Roots {
            entry: None,
            exports: &[],
        },
        prepared,
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

fn provider(name: &str) -> std::result::Result<(), String> {
    let input = crate::resolve::fixture(&format!("{ROOT}/{name}")).map_err(|e| e.to_string())?;
    analyze(&crate::resolve::source_provider_config(
        &input,
        "object_autos",
        &Config::default(),
    )?)
}

#[test]
fn source_object_autos_match_pinned_outcomes() {
    let mut errors = Vec::new();
    for &(name, cause) in CASES {
        match (provider(name), cause) {
            (Ok(()), None) => {}
            (Err(error), Some(cause)) if error.contains(cause) => {}
            (outcome, _) => errors.push(format!("{name}: {outcome:?}")),
        }
    }
    assert!(errors.is_empty(), "{}", errors.join("\n"));
}

#[test]
fn custom_source_auto_beside_a_principal_is_a_typed_gap() -> Result {
    // Pinned Rust accepts this input; no custom marker is represented yet.
    let error = provider("v2/custom-auto.rs")
        .err()
        .ok_or("custom auto object accepted without a represented marker")?;
    assert!(
        error.contains("object auto traits without a represented marker are not implemented"),
        "{error}"
    );
    Ok(())
}

#[test]
fn ordinary_objects_keep_builtin_markers_and_one_principal() -> Result {
    let ordinary = |source: &str| {
        crate::resolve::resolve_config(source, true, &Config::default()).and_then(|r| analyze(&r))
    };
    ordinary("trait Any{}fn f(_:&(dyn Any+Send+Sync)){}fn g(_:&(dyn Send+Any)){}")?;
    let error = ordinary("trait T{}trait U{}fn check(_: &(dyn T+U)){}")
        .err()
        .ok_or("two ordinary principals accepted")?;
    assert!(
        error.contains("object requires exactly one principal trait"),
        "{error}"
    );
    Ok(())
}
