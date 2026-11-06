//! Supported no-drop const paths stay distinct from active cleanup authority.
use crate::resolve as source;
use rustgm::{lowir::program::Program, pipeline};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
fn program(path: &str) -> Result<Program> {
    let text = source::fixture(path)?;
    let report = source::source_provider(&text, "const_cleanup_repair")?;
    let pipeline::Analysis {
        mut facts,
        prepared,
    } = pipeline::analyze_source(&report)?;
    Ok(rustgm::lower::instances::build_roots_prepared(
        &report,
        &mut facts,
        rustgm::lower::Roots {
            entry: None,
            exports: &[],
        },
        prepared,
    )?)
}
fn cleanup(name: &str) -> Result<Program> {
    program(&format!("pa31/tests/spec/source-const-cleanup/{name}.rs"))
}
#[test]
fn unchanged_no_drop_constructors_initializers_and_copy_bodies_compile() -> Result {
    for path in [
        "pa28/tests/spec/source-transmute/positive-owned-native.rs",
        "pa31/tests/spec/source-phantom/source-marker.rs",
        "pa31/tests/spec/contract-helpers/native-item.rs",
        "pa31/tests/spec/contract-helpers/native-closure.rs",
        "pa31/tests/spec/contract-helpers/captured-bool.rs",
        "pa31/tests/spec/contract-helpers/captured-nominal.rs",
    ] {
        program(path).map_err(|error| format!("{path}: {error}"))?;
    }
    Ok(())
}
#[test]
fn unchanged_comparison_bodies_retain_actual_precise_cleanup_request() -> Result {
    program("pa31/tests/spec/source-comparisons/ordering/const-source-comparisons-v3.rs")?;
    for (name, cause) in [
        ("missing-ord-impl", "can't compare"),
        ("wrong-ordering-tag", "Ordering"),
        ("wrong-intrinsic-copy", "Copy"),
    ] {
        let error = program(&format!(
            "pa31/tests/spec/source-comparisons/ordering/{name}-v3.rs"
        ))
        .err()
        .ok_or("invalid source comparison accepted")?;
        assert!(error.to_string().contains(cause), "{name}: {error}");
    }
    Ok(())
}
#[test]
fn precise_request_omits_only_non_access_cleanup_eligibility() -> Result {
    for name in ["prstr", "sealok"] {
        cleanup(name).map_err(|error| format!("{name}: {error}"))?;
    }
    for name in ["nrstr", "pract", "macreq", "sealno"] {
        let error = cleanup(name)
            .err()
            .ok_or("const cleanup authority changed")?;
        let message = error.to_string();
        assert!(
            message.contains("const Destruct")
                || message.contains("const Drop")
                || message.contains("can't drop"),
            "{name}: {message}"
        );
        assert!(!message.contains("not implemented"), "{name}: {message}");
    }
    Ok(())
}
#[test]
fn actual_known_enum_cleanup_definitions_are_separate_from_unknown_payloads() -> Result {
    for name in ["prknow", "prmove"] {
        cleanup(name).map_err(|e| format!("{name}: {e}"))?;
    }
    let error = cleanup("prunk")
        .err()
        .ok_or("unknown nonconst payload accepted")?;
    assert!(
        error.to_string().contains("const Drop")
            || error.to_string().contains("const Destruct")
            || error.to_string().contains("can't drop"),
        "{error}"
    );
    Ok(())
}

#[test]
fn no_drop_repair_keeps_reached_intrinsic_body_fences() -> Result {
    for name in ["const", "runtime", "reified"] {
        let error = program(&format!(
            "pa28/tests/spec/intrinsic-declarations/execution/{name}.rs"
        ))
        .err()
        .ok_or("authored unsupported intrinsic executed")?;
        let error = error.to_string();
        assert!(error.contains("source intrinsic"), "{name}: {error}");
        assert!(
            error.contains(if name == "reified" {
                "cannot be coerced or cast"
            } else {
                "not implemented"
            }),
            "{name}: {error}"
        );
    }
    Ok(())
}
#[test]
fn precise_cleanup_cannot_borrow_a_foreign_provider_report() -> Result {
    let text = source::fixture("pa31/tests/spec/source-const-cleanup/prstr.rs")?;
    let original = source::source_provider(&text, "actual_cleanup_owner")?;
    let pipeline::Analysis {
        mut facts,
        prepared,
    } = pipeline::analyze_source(&original)?;
    let foreign = source::source_provider(&text, "foreign_cleanup_owner")?;
    let error = rustgm::lower::instances::build_roots_prepared(
        &foreign,
        &mut facts,
        rustgm::lower::Roots {
            entry: None,
            exports: &[],
        },
        prepared,
    )
    .err()
    .ok_or("cleanup query accepted another provider report")?;
    assert!(error.to_string().contains("provider report"), "{error}");
    Ok(())
}
#[test]
fn precise_cleanup_cannot_borrow_a_same_provider_different_source_image() -> Result {
    let enabled = source::fixture("pa31/tests/spec/source-const-cleanup/prstr.rs")?;
    let disabled = enabled.replace(
        "const_precise_live_drops",
        &format!(
            "{:<width$}",
            "const_destruct",
            width = "const_precise_live_drops".len()
        ),
    );
    assert_eq!(enabled.len(), disabled.len());
    let original = source::source_provider(&disabled, "same_cleanup_owner")?;
    let analysis = match pipeline::analyze_source(&original) {
        Ok(analysis) => analysis,
        Err(error) => {
            assert!(error.to_string().contains("can't drop"), "{error}");
            println!(
                "source-image opponent rejected before prepared analysis publication: {error}"
            );
            return Ok(());
        }
    };
    let pipeline::Analysis {
        mut facts,
        prepared,
    } = analysis;
    let foreign = source::source_provider(&enabled, "same_cleanup_owner")?;
    let error = rustgm::lower::instances::build_roots_prepared(
        &foreign,
        &mut facts,
        rustgm::lower::Roots {
            entry: None,
            exports: &[],
        },
        prepared,
    )
    .err()
    .ok_or("cleanup query accepted another source image feature")?;
    println!("source-image opponent rejected at lowering: {error}");
    Ok(())
}
