//! Symbolic-length builtin indexes share the same checked classification as fixed arrays.
use crate::resolve;
use rustgm::{expand::config::Config, pipeline, support::diagnostics::LintCap};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const PACKET: &str = "pa31/tests/spec/symbolic-index";

fn analyzed(name: &str) -> Result<(rustgm::resolve::expansion::Report, pipeline::Analysis)> {
    let mut config = Config::default();
    config.set_lint_cap(LintCap::Allow);
    let text = resolve::fixture(&format!("{PACKET}/{name}.rs"))?;
    let report = resolve::source_provider_config(&text, "symbolic_index", &config)?;
    let analysis = pipeline::analyze_source(&report)?;
    Ok((report, analysis))
}

#[test]
fn symbolic_builtin_places_keep_const_policy_and_custom_call_opponents() -> Result {
    let mut failures = Vec::new();
    for (name, accepted) in [
        ("runtime-symbolic", true),
        ("fixed-stable", true),
        ("symbolic-stable", true),
        ("symbolic-mutable", true),
        ("symbolic-raw", true),
        ("symbolic-raw-mut", true),
        ("custom-key", true),
        ("changed-output", true),
        ("missing-impl", false),
        ("nonconst-impl", false),
        ("explicit-stable", false),
        ("native", true),
        ("native-generic", true),
        ("out-of-bounds", false),
    ] {
        let result = analyzed(name);
        if result.is_ok() != accepted {
            failures.push(format!("{name}: {:?}", result.err()));
        } else if let Err(error) = result {
            let error = error.to_string();
            if name == "out-of-bounds" && !error.contains("out of bounds") {
                failures.push(format!("{name}: wrong cause: {error}"));
            }
            if [
                "budget",
                "exhausted",
                "lost its",
                "changed its",
                "semantic contract is not implemented",
            ]
            .iter()
            .any(|cause| error.contains(cause))
            {
                failures.push(format!("{name}: implementation failure: {error}"));
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    for name in ["read", "write", "reference"] {
        crate::semantic::analyze(&resolve::fixture(&format!(
            "pa18/tests/spec/operands/{name}.rs"
        ))?)?;
    }
    Ok(())
}

#[test]
fn symbolic_builtin_mutation_and_read_produce_a_real_constant() -> Result {
    let (report, analysis) = analyzed("native")?;
    let initializer = analysis
        .facts
        .initializers
        .iter()
        .find(|initializer| {
            report
                .package
                .names
                .get(report.resolution.definitions[initializer.definition.0 as usize].name)
                == "VALUE"
        })
        .ok_or("actual symbolic VALUE initializer")?;
    let reader = rustgm::sema::values::Reader::new(&analysis.facts);
    let value = reader
        .definition(initializer.definition)
        .ok_or("completed symbolic array constant")?;
    assert!(matches!(
        reader.view(value)?,
        rustgm::sema::values::View::Scalar(rustgm::sema::model::Scalar::Integer(47))
    ));
    Ok(())
}
