//! Actual Field headers, selected evidence and source result projections.
use crate::resolve as source;
use rustgm::{expand::config::Config, pipeline, support::diagnostics::LintCap};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const PACKET: &str = "pa31/tests/spec/field-source";

fn analyze(name: &str) -> Result<pipeline::Analysis> {
    analyze_in(PACKET, name)
}
fn analyze_in(packet: &str, name: &str) -> Result<pipeline::Analysis> {
    let mut config = Config::default();
    config.set_lint_cap(LintCap::Allow);
    let report = source::source_provider_config(
        &source::fixture(&format!("{packet}/{name}.rs"))?,
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

#[test]
fn actual_field_proof_and_both_projections_share_the_selected_application() -> Result {
    analyze_in("pa31/tests/spec/field-operations", "proof-only")?;
    analyze_in("pa31/tests/spec/field-operations", "proof-demand-only")?;
    Ok(())
}

#[test]
fn builtin_field_evidence_rejects_ineligible_bases_fields_and_impostors() -> Result {
    for name in [
        "packed",
        "sized-field-unsized-base",
        "unsized-field",
        "enum-demand",
        "union-demand",
        "nominal-impostor",
    ] {
        let error = analyze_in("pa31/tests/spec/field-operations", name)
            .err()
            .ok_or_else(|| format!("{name} unexpectedly accepted"))?
            .to_string();
        assert!(
            error.contains(
                "trait obligation requires an unambiguous matching assumption or implementation"
            ),
            "{name}: {error}"
        );
    }
    Ok(())
}

#[test]
fn selected_results_preserve_full_generic_arguments_and_payload_lifetimes() -> Result {
    for name in ["generic-premises", "lifetime-positive"] {
        analyze_in("pa31/tests/spec/field-operations", name)
            .map_err(|error| format!("{name}: {error}"))?;
    }
    let error = analyze_in("pa31/tests/spec/field-operations", "wrong-full-application")
        .err()
        .ok_or("distinct generic field applications were equated")?
        .to_string();
    assert!(error.contains("incompatible types"), "{error}");
    Ok(())
}

#[test]
fn actual_field_source_is_reauthenticated_before_instance_publication() -> Result {
    let text = source::fixture("pa31/tests/spec/field-operations/proof-only.rs")?;
    let altered = text.replacen("pub unsafe trait Field", "pub        trait Field", 1);
    assert_ne!(text, altered);
    let mut config = Config::default();
    config.set_lint_cap(LintCap::Allow);
    let report = source::source_provider_config(&text, "field_source", &config)?;
    let changed = source::source_provider_config(&altered, "field_source", &config)?;
    pipeline::analyze_source(&changed)?;
    let definition = report
        .resolution
        .definitions
        .iter()
        .enumerate()
        .find(|(_, d)| report.package.names.get(d.name) == "proof")
        .map(|(index, _)| u32::try_from(index).map(rustgm::resolve::model::DefId))
        .transpose()?
        .ok_or("actual proof root")?;
    let pipeline::Analysis {
        mut facts,
        prepared,
    } = pipeline::analyze_source(&report)?;
    rustgm::lower::instances::build_roots_prepared(
        &report,
        &mut facts,
        rustgm::lower::Roots {
            entry: None,
            exports: &[definition],
        },
        prepared,
    )?;
    let pipeline::Analysis {
        mut facts,
        prepared,
    } = pipeline::analyze_source(&report)?;
    let error = rustgm::lower::instances::build_roots_prepared(
        &changed,
        &mut facts,
        rustgm::lower::Roots {
            entry: None,
            exports: &[definition],
        },
        prepared,
    )
    .err()
    .ok_or("Field evidence accepted a different actual source header")?
    .to_string();
    assert!(
        error.contains("source") || error.contains("Field"),
        "{error}"
    );
    Ok(())
}

#[test]
#[ignore = "requires the pinned CPPGM object backend and a C compiler"]
fn genuine_field_projections_and_actual_callback_execute() -> Result {
    let input = source::fixture("pa31/tests/spec/field-operations/projection-native.rs")?;
    let mut config = Config::default();
    config.set_lint_cap(LintCap::Allow);
    let report = source::source_provider_config(&input, "field_projection", &config)?;
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
        "obj/parallel/field-projections/native-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&folder)?;
    crate::ffi_program::check(
        &root,
        &folder,
        "field_projection",
        &program,
        &source::fixture("pa31/tests/spec/field-operations/projection-native.c")?,
        "projection 23 29 1 1\n",
    )?;
    Ok(())
}

#[test]
fn reflected_results_keep_ordinary_move_lifetime_and_parent_requirements() -> Result {
    let mut failures = Vec::new();
    for (name, cause) in [
        (
            "generic-missing",
            "associated type requires an unambiguous implementation",
        ),
        (
            "foreign-members-demand",
            "associated type requires an unambiguous implementation",
        ),
        (
            "noncopy-result",
            "cannot move a non-Copy value out of a reference",
        ),
        (
            "lifetime-escape",
            "required lifetime does not outlive its target",
        ),
    ] {
        let mut config = Config::default();
        config.set_lint_cap(LintCap::Allow);
        let input = source::fixture(&format!("pa31/tests/spec/field-operations/{name}.rs"))?;
        let report = source::source_provider_config(&input, "field_opponent", &config)?;
        let result = pipeline::analyze_source(&report).and_then(|analysis| {
            let pipeline::Analysis {
                mut facts,
                prepared,
            } = analysis;
            rustgm::lower::instances::build_roots_prepared(
                &report,
                &mut facts,
                rustgm::lower::Roots {
                    entry: None,
                    exports: &[],
                },
                prepared,
            )
        });
        match result {
            Err(error) if error.to_string().contains(cause) => {}
            Err(error) => failures.push(format!("{name}: {error}")),
            Ok(_) => failures.push(format!("{name} unexpectedly accepted")),
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}
