//! Actual Field headers preserve ordinary source legality without field evidence.
use crate::resolve as source;
use rustgm::{expand::config::Config, pipeline, support::diagnostics::LintCap};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const PACKET: &str = "pa31/tests/spec/field-source";

fn analyze(name: &str) -> Result<pipeline::Analysis> {
    let mut config = Config::default();
    config.set_lint_cap(LintCap::Allow);
    let report = source::source_provider_config(
        &source::fixture(&format!("{PACKET}/{name}.rs"))?,
        "field_source",
        &config,
    )?;
    Ok(pipeline::analyze_source(&report)?)
}

#[test]
fn unused_alternative_headers_and_unrelated_trait_members_remain_ordinary() -> Result {
    for name in [
        "unused-alternative-v4",
        "unrelated-owner-v4",
        "inherent-owner-v4",
    ] {
        analyze(name).map_err(|error| format!("{name}: {error}"))?;
    }
    Ok(())
}

#[test]
fn exact_original_headers_and_provided_offset_body_are_checked() -> Result {
    analyze("original-headers-v4")?;
    Ok(())
}

#[test]
fn written_targets_and_parent_generic_counts_reject_before_completion() -> Result {
    for (name, cause) in [
        ("wrong-struct-target-v4", "required declaration target"),
        ("wrong-trait-target-v4", "required declaration target"),
        ("wrong-base-target-v4", "required declaration target"),
        ("wrong-type-target-v4", "required declaration target"),
        ("wrong-offset-target-v4", "required declaration target"),
        ("wrong-struct-arity-v4", "required generic arity"),
        ("wrong-trait-arity-v4", "required generic arity"),
        ("wrong-member-parent-arity-v4", "required generic arity"),
    ] {
        let error = analyze(name)
            .err()
            .ok_or_else(|| format!("{name} unexpectedly accepted"))?;
        assert!(error.to_string().contains(cause), "{name}: {error}");
    }
    Ok(())
}

#[test]
fn ordinary_constant_types_and_explicit_implementation_policy_remain() -> Result {
    for (name, causes) in [
        ("invalid-ordinary-body-v4", &["incompatible types"][..]),
        (
            "explicit-impl-v4",
            &["explicit implementation is prohibited by its source trait declaration"][..],
        ),
    ] {
        let error = analyze(name)
            .err()
            .ok_or_else(|| format!("{name} unexpectedly accepted"))?
            .to_string();
        assert!(
            causes.iter().any(|cause| error.contains(cause)),
            "{name}: {error}"
        );
        assert!(
            !error.contains("semantic contract is not implemented"),
            "{name}: {error}"
        );
    }
    Ok(())
}

#[test]
fn exact_original_nominal_trait_and_copy_clone_intervals_are_preserved() -> Result {
    let original = source::fixture(&format!("{PACKET}/original-field.rs"))?;
    let actual = source::fixture(&format!("{PACKET}/original-headers-v4.rs"))?;
    for (start, end) in [(104, 667), (5044, 6342), (2575, 2851)] {
        assert!(actual.contains(&original[start..end]), "{start}..{end}");
    }
    Ok(())
}
