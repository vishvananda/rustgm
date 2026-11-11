//! Returned const closures keep the selected owner's defining requirements.
use rustgm::{expand::config::Config, pipeline, support::diagnostics::LintCap};
type Result = std::result::Result<(), Box<dyn std::error::Error>>;

#[test]
fn conditional_closure_results_pay_their_instantiated_defining_contracts() -> Result {
    let mut failures = Vec::new();
    for (name, accepted) in [
        ("self", true),
        ("binary", true),
        ("forward", true),
        ("direct", true),
        ("ordinary", true),
        ("const-factory", true),
        ("ordinary-bound-reject", false),
        ("ordinary-closure-reject", false),
        ("wrong-output-reject", false),
        ("runtime-creation-reject", false),
    ] {
        let input = crate::resolve::fixture(&format!("pa31/tests/spec/const-adapters/{name}.rs"))?;
        let mut config = Config::default();
        config.set_lint_cap(LintCap::Allow);
        let report = crate::resolve::source_provider_config(&input, "const_adapter", &config)?;
        let result = pipeline::analyze_source(&report);
        if result.is_ok() != accepted {
            failures.push(format!("{name}: {:?}", result.as_ref().err()));
        }
        if let Err(error) = result {
            let message = error.to_string();
            if [
                "not implemented",
                "scoped continuation",
                "budget",
                "invalid canonical",
            ]
            .iter()
            .any(|part| message.contains(part))
            {
                failures.push(format!("{name}: implementation failure: {message}"));
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}
