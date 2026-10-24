//! Exact associated-subject destruction promises do not grant structural or const execution.
use super::{TestResult, lower};

fn fixture(name: &str) -> TestResult<String> {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../pa31/tests/spec/trait-default-effects/projections");
    let root = if name.starts_with("independent-") || name.starts_with("opening-") {
        root.join("body-opening").join(format!("{name}.rs"))
    } else if name.starts_with("provider-") {
        root.join("public-context")
            .join(format!("public-{name}.rs"))
    } else {
        root.join(format!("{name}.rs"))
    };
    Ok(std::fs::read_to_string(root)?)
}

#[test]
fn body_opening_never_pays_other_lifetimes_constants_or_inference_alternatives() -> TestResult {
    for name in [
        "independent-lifetimes-unpaid",
        "independent-lifetimes-paid",
        "independent-lifetime-owners-unpaid",
        "independent-const-arguments-unpaid",
        "independent-inference-alternatives",
    ] {
        assert!(
            lower(&fixture(name)?).is_err(),
            "{name}: unsupported or unpaid input accepted"
        );
    }
    Ok(())
}

#[test]
fn distinct_lifetime_owners_pay_both_actual_projected_cleanup_demands() -> TestResult {
    lower(&fixture("independent-lifetime-owners-paid")?)?;
    let error = lower(&fixture("independent-lifetime-owners-unpaid")?)
        .err()
        .ok_or("missing Right Destruct demand accepted")?;
    assert!(
        error
            .to_string()
            .contains("conditional Destruct obligation"),
        "{error}"
    );
    Ok(())
}

#[test]
#[ignore = "requires the pinned CPPGM backend and host C linker"]
fn distinct_lifetime_owners_execute_both_actual_destructors() -> TestResult {
    let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../obj/parallel/pa31/projected-native")
        .join(std::process::id().to_string());
    std::fs::create_dir_all(&folder)?;
    super::native::execute_observed(
        &folder,
        "distinct-lifetime-owners",
        &fixture("independent-lifetime-owners-paid")?,
        "extern void exercise(unsigned char*,unsigned char*);int main(void){unsigned char left=0,right=0;exercise(&left,&right);return left!=7||right!=7;}\n",
    )
}

#[test]
fn actual_alias_opening_width_sites_and_noise_reach_complete_original_proofs() -> TestResult {
    for (axis, count, noise) in [
        ("subjects", 8, 0),
        ("subjects", 32, 0),
        ("subjects", 128, 0),
        ("subjects", 8, 128),
        ("sites", 8, 0),
        ("sites", 32, 0),
        ("sites", 128, 0),
    ] {
        let name = format!("opening-{axis}-{count}-noise-{noise}");
        let report = super::report(&fixture(&name)?)?;
        let rustgm::pipeline::Analysis {
            mut facts,
            prepared,
        } = rustgm::pipeline::analyze_source(&report)?;
        let before = facts.body_work.canonical_steps;
        rustgm::lower::instances::build_roots_prepared(
            &report,
            &mut facts,
            rustgm::lower::Roots {
                entry: None,
                exports: &[],
            },
            prepared,
        )
        .map_err(|error| format!("{name}: {error}"))?;
        println!(
            "complete_alias_lower axis={axis} count={count} noise={noise} steps={}",
            facts.body_work.canonical_steps - before
        );
    }
    Ok(())
}

#[test]
#[ignore = "requires the pinned CPPGM backend and host C linker"]
fn actual_alias_opening_width_sites_and_noise_execute_native_seven() -> TestResult {
    let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../obj/parallel/pa31/projected-native")
        .join(std::process::id().to_string());
    std::fs::create_dir_all(&folder)?;
    for (axis, count, noise) in [
        ("subjects", 8, 0),
        ("subjects", 32, 0),
        ("subjects", 128, 0),
        ("subjects", 8, 128),
        ("sites", 8, 0),
        ("sites", 32, 0),
        ("sites", 128, 0),
    ] {
        let name = format!("opening-{axis}-{count}-noise-{noise}");
        super::native::execute(&folder, &name, &fixture(&name)?, 7)?;
    }
    Ok(())
}

#[test]
fn fully_live_projected_defaults_keep_exact_applications_and_move_opposites() -> TestResult {
    for name in [
        "provider-paid",
        "provider-trait-arguments",
        "provider-family-arguments",
        "provider-moved",
    ] {
        lower(&fixture(name)?).map_err(|error| format!("{name}: {error}"))?;
    }
    Ok(())
}

#[test]
fn projected_defaults_reject_missing_ordinary_or_other_application_promises() -> TestResult {
    for name in [
        "provider-missing",
        "provider-ordinary-only",
        "provider-wrong-member",
        "provider-trait-arguments-unpaid",
        "provider-family-arguments-unpaid",
    ] {
        let error = lower(&fixture(name)?)
            .err()
            .ok_or("unpaid projected cleanup accepted")?;
        assert!(
            error
                .to_string()
                .contains("conditional Destruct obligation"),
            "{name}: {error}"
        );
    }
    assert!(lower(&fixture("provider-unavailable-formation")?).is_err());
    Ok(())
}

#[test]
fn composite_and_partial_container_cleanup_stay_explicit_unsupported_requirements() -> TestResult {
    for name in ["provider-composite", "provider-partial-container"] {
        let error = lower(&fixture(name)?)
            .err()
            .ok_or("unsupported structural cleanup accepted")?;
        assert!(
            error.to_string().contains("not implemented"),
            "{name}: {error}"
        );
    }
    Ok(())
}

#[test]
fn source_projection_width_sites_and_noise_reach_complete_original_proofs() -> TestResult {
    for (axis, count, noise) in [
        ("subjects", 8, 0),
        ("subjects", 32, 0),
        ("subjects", 128, 0),
        ("subjects", 8, 128),
        ("sites", 8, 0),
        ("sites", 32, 0),
        ("sites", 128, 0),
    ] {
        let name = format!("{axis}-{count}-noise-{noise}");
        lower(&fixture(&name)?).map_err(|error| format!("{name}: {error}"))?;
    }
    Ok(())
}

#[test]
#[ignore = "requires the pinned CPPGM backend and host C linker"]
fn projected_originals_and_scaling_execute_native_seven() -> TestResult {
    let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../obj/parallel/pa31/projected-native")
        .join(std::process::id().to_string());
    std::fs::create_dir_all(&folder)?;
    for name in [
        "provider-paid",
        "provider-trait-arguments",
        "provider-family-arguments",
        "provider-moved",
    ] {
        super::native::execute(&folder, name, &fixture(name)?, 7)?;
    }
    for (axis, count, noise) in [
        ("subjects", 8, 0),
        ("subjects", 32, 0),
        ("subjects", 128, 0),
        ("subjects", 8, 128),
        ("sites", 8, 0),
        ("sites", 32, 0),
        ("sites", 128, 0),
    ] {
        let name = format!("{axis}-{count}-noise-{noise}");
        super::native::execute(&folder, &name, &fixture(&name)?, 7)?;
    }
    Ok(())
}

#[test]
fn materialized_alias_instances_reach_complete_source_lowering() -> TestResult {
    for name in [
        "instance-engine",
        "instance-arguments",
        "instance-engine-scoped",
        "instance-arguments-scoped",
    ] {
        let source = std::fs::read_to_string(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../pa31/tests/spec/trait-default-effects/projections/body-opening")
                .join(format!("{name}.rs")),
        )?;
        lower(&source)?;
    }
    Ok(())
}

#[test]
#[ignore = "requires the pinned CPPGM backend and host C linker"]
fn materialized_alias_instances_execute_native_seven() -> TestResult {
    let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../obj/parallel/pa31/projected-native")
        .join(std::process::id().to_string());
    std::fs::create_dir_all(&folder)?;
    for name in [
        "instance-engine",
        "instance-arguments",
        "instance-engine-scoped",
        "instance-arguments-scoped",
    ] {
        let source = std::fs::read_to_string(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../pa31/tests/spec/trait-default-effects/projections/body-opening")
                .join(format!("{name}.rs")),
        )?;
        super::native::execute(&folder, name, &source, 7)?;
    }
    Ok(())
}
