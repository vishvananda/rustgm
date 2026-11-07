//! The real const wrapper is an attribute target, never const or arithmetic permission.
use rustgm::{
    expand::config::Config, pipeline, resolve::model::DefKind, support::diagnostics::LintCap,
};
type Result = std::result::Result<(), Box<dyn std::error::Error>>;
const ROOT: &str = "pa31/tests/spec/inherited-overflow";
fn report(name: &str) -> std::result::Result<rustgm::resolve::expansion::Report, String> {
    let input = crate::resolve::fixture(&format!("{ROOT}/{name}.rs")).map_err(|e| e.to_string())?;
    let mut config = Config::default();
    config.set_lint_cap(LintCap::Allow);
    crate::resolve::source_provider_config(&input, "overflow_word", &config)
}
#[test]
fn original_const_closure_word_consumes_its_actual_wrapper_and_child() -> Result {
    for name in ["const-closure-v3", "ordinary-closure-v3"] {
        rustgm::sema::declarations::analyze(&report(name)?)?;
    }
    let report = report("const-closure-v3")?;
    let pipeline::Analysis {
        mut facts,
        prepared,
    } = pipeline::analyze_source(&report)?;
    assert!(
        facts
            .closures
            .all()
            .iter()
            .any(|body| body.constant.is_some())
    );
    rustgm::lower::instances::build_roots_prepared(
        &report,
        &mut facts,
        rustgm::lower::Roots {
            entry: None,
            exports: &[],
        },
        prepared,
    )?;
    Ok(())
}
#[test]
fn inherited_word_does_not_supply_target_or_feature_authority() -> Result {
    for (name, expected) in [
        ("missing-closure-feature-v3", "const_closures"),
        ("missing-internal-feature-v3", "rustc_attrs"),
        (
            "wrong-target-v3",
            "inherited overflow policy requires a function body or closure",
        ),
    ] {
        let rejected = report(name).and_then(|report| {
            rustgm::sema::declarations::analyze(&report)
                .map(|_| ())
                .map_err(|e| e.to_string())
        });
        let error = rejected.err().ok_or_else(|| format!("accepted {name}"))?;
        assert!(error.contains(expected), "{name}: {error}");
    }
    Ok(())
}
#[test]
fn actual_runtime_checks_and_const_overflow_remain_independent() -> Result {
    let report = report("checked-function")?;
    let pipeline::Analysis {
        mut facts,
        prepared,
    } = pipeline::analyze_source(&report)?;
    let definition = report
        .resolution
        .definitions
        .iter()
        .enumerate()
        .find(|(_, d)| d.kind == DefKind::Function && report.package.names.get(d.name) == "checked")
        .map(|(index, _)| u32::try_from(index).map(rustgm::resolve::model::DefId))
        .transpose()?
        .ok_or("missing checked function")?;
    let program = rustgm::lower::instances::build_roots_prepared(
        &report,
        &mut facts,
        rustgm::lower::Roots {
            entry: None,
            exports: &[definition],
        },
        prepared,
    )?;
    assert!(
        program
            .symbols
            .iter()
            .filter_map(|s| s.body.as_ref())
            .flat_map(|b| &b.blocks)
            .any(|b| matches!(
                b.terminator,
                rustgm::lowir::program::Terminator::Branch { .. }
            ))
    );
    let error = pipeline::analyze_source(&self::report("const-overflow")?)
        .err()
        .ok_or("const arithmetic overflow accepted")?;
    assert!(error.to_string().contains("overflow"), "{error}");
    Ok(())
}
