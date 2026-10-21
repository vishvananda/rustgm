//! Quantified object conversions retain caller captures and reject local lifetime escape.
use super::{checked, native};
use std::path::PathBuf;

const ACCEPTED: &[&str] = &[
    "mutable",
    "weaker",
    "identity-result",
    "two-inputs",
    "external-result",
    "nested-callable",
    "generic",
    "once-view",
    "captured-zst",
    "generic-consumer",
    "upcast",
    "projected",
    "projected-lifetime",
    "outer-first",
    "outer-second",
    "outer-upcast",
    "nominal-outer",
];
const REJECTED: &[(&str, i32, &str)] = &[
    (
        "local-as-static",
        0,
        "required lifetime does not outlive its target",
    ),
    (
        "capture-as-static",
        0,
        "required lifetime does not outlive its target",
    ),
    (
        "mutable-to-shared",
        1,
        "trait obligation requires an unambiguous",
    ),
    (
        "loan-conflict",
        0,
        "access conflicts with a live reference loan",
    ),
    (
        "late-permission",
        0,
        "required lifetime does not outlive its target",
    ),
    (
        "outer-wrong",
        0,
        "required lifetime does not outlive its target",
    ),
];
fn fixture(name: &str) -> PathBuf {
    let relative = if name == "nominal-outer" {
        "../pa23/tests/spec/conversions/quantified-outer-result.rs".to_owned()
    } else {
        format!("../pa24/tests/spec/quantified-dispatch/{name}.rs")
    };
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(relative)
}
#[test]
fn quantified_conversions_preserve_independent_source_outcomes()
-> Result<(), Box<dyn std::error::Error>> {
    let evidence = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!(
        "../obj/parallel/pa24/quantified-dispatch/source/{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&evidence)?;
    for &name in ACCEPTED {
        checked::check(&fixture(name), &evidence, name, (0, 0, ""))?;
    }
    for &(name, types, reason) in REJECTED {
        checked::check_modes(&fixture(name), &evidence, name, (1, [types, 1, 1], reason))?;
    }
    Ok(())
}
#[test]
#[ignore = "requires the qualified CPPGM backend; run make test-maintainer-native"]
fn quantified_captured_objects_execute() -> Result<(), Box<dyn std::error::Error>> {
    for &name in ACCEPTED {
        let source = std::fs::read_to_string(fixture(name))?;
        let body = source
            .strip_prefix(native::CHECK)
            .ok_or("missing check prefix")?;
        native::check_cases(
            "parallel/pa24/quantified-dispatch/native",
            &[(name, body, false)],
        )?;
    }
    Ok(())
}
