//! Authenticated const creation and checked child effects keep ordinary closure boundaries.
use rustgm::{expand::config::Config, pipeline, support::diagnostics::LintCap};
type Result = std::result::Result<(), Box<dyn std::error::Error>>;

fn analyze(input: &str) -> std::result::Result<(), String> {
    let mut config = Config::default();
    config.set_lint_cap(LintCap::Allow);
    let report = crate::resolve::source_provider_config(input, "constant_closures", &config)?;
    let pipeline::Analysis {
        mut facts,
        prepared,
    } = pipeline::analyze_source(&report).map_err(|error| error.to_string())?;
    rustgm::lower::instances::build_roots_prepared(
        &report,
        &mut facts,
        rustgm::lower::Roots {
            entry: None,
            exports: &[],
        },
        prepared,
    )
    .map(|_| ())
    .map_err(|error| error.to_string())
}

#[test]
fn actual_wrapper_and_child_preserve_creation_effect_and_nested_contexts() -> Result {
    let input = crate::resolve::fixture("pa31/tests/spec/inherited-overflow/const-closure-v3.rs")?;
    analyze(&input)?;
    let runtime = input.replace("pub const fn original_word", "pub fn original_word");
    let error = analyze(&runtime)
        .err()
        .ok_or("runtime const creation accepted")?;
    assert!(error.contains("outside of const contexts"), "{error}");
    let nested = input.clone() + "pub const fn nested(){let _=||{let _=const || 23usize;};}";
    let error = analyze(&nested)
        .err()
        .ok_or("ordinary child inherited const creation")?;
    assert!(error.contains("outside of const contexts"), "{error}");
    Ok(())
}

#[test]
fn unused_const_body_checks_calls_without_changing_ordinary_child_effects() -> Result {
    let input = crate::resolve::fixture("pa31/tests/spec/inherited-overflow/const-closure-v3.rs")?;
    let source = input + "fn runtime()->usize{23}\npub const fn nested(){let _=||runtime();}";
    analyze(&source)?;
    let constant = source.replace("let _=||runtime()", "let _=const ||runtime()");
    let error = analyze(&constant)
        .err()
        .ok_or("unused const body skipped call legality")?;
    assert!(error.contains("non-const"), "{error}");
    Ok(())
}

#[test]
fn const_qualification_keeps_existing_hot_record_widths() {
    use std::mem::size_of;
    assert_eq!(size_of::<rustgm::types::closure::Declaration>(), 1);
    assert_eq!(size_of::<rustgm::types::Kind>(), 24);
    assert_eq!(size_of::<rustgm::body::ir::Body>(), 456);
    assert_eq!(size_of::<rustgm::body::ir::Owner>(), 20);
    assert_eq!(size_of::<rustgm::body::ir::Value>(), 24);
    assert_eq!(size_of::<rustgm::syntax::ast::Node>(), 36);
    assert_eq!(size_of::<rustgm::sema::closures::Body>(), 76);
}
