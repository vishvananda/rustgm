//! Reference Self retains its nested receiver while the vtable targets the captured leaf.
use super::{checked, native};
use std::path::PathBuf;

const ACCEPTED: &[&str] = &[
    "shared",
    "shared-weaker",
    "mutable",
    "nested-shared",
    "nested-mutable",
    "mixed",
    "generic",
    "quantified",
    "outer-result",
    "once-view",
];
fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../pa24/tests/spec/reference-objects/{name}.rs"))
}
#[test]
fn reference_objects_preserve_independent_outcomes() -> Result<(), Box<dyn std::error::Error>> {
    let evidence = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!(
        "../obj/parallel/pa24/reference-objects/source/{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&evidence)?;
    for &name in ACCEPTED {
        checked::check(&fixture(name), &evidence, name, (0, 0, ""))?;
    }
    for (name, types, reason) in [
        (
            "mutable-as-shared",
            1,
            "trait obligation requires an unambiguous",
        ),
        (
            "shared-mutable-leaf",
            1,
            "associated type requires an unambiguous validated implementation",
        ),
        (
            "loan-conflict",
            0,
            "access conflicts with a live reference loan",
        ),
        (
            "local-escape",
            0,
            "access conflicts with a live reference loan",
        ),
    ] {
        checked::check_modes(&fixture(name), &evidence, name, (1, [types, 1, 1], reason))?;
    }
    Ok(())
}
#[test]
#[ignore = "requires the qualified CPPGM backend; run make test-maintainer-native"]
fn reference_objects_execute() -> Result<(), Box<dyn std::error::Error>> {
    for &name in ACCEPTED {
        let source = std::fs::read_to_string(fixture(name))?;
        let body = source
            .strip_prefix(native::CHECK)
            .ok_or("missing check prefix")?;
        native::check_cases(
            "parallel/pa24/reference-objects/native",
            &[(name, body, false)],
        )?;
    }
    Ok(())
}
