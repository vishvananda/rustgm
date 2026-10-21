//! Owned receiver adaptations preserve the proved source and normal cleanup.
use super::{native, rustc};
const CASES: &[(&str, bool, bool)] = &[
    ("shared-to-mutable", true, true),
    ("shared-to-once", true, true),
    ("mutable-to-once", true, true),
    ("owned-shared-to-mutable", true, true),
    ("owned-mutable-to-once", true, true),
    ("shared-reference-to-once", true, true),
    ("mutable-reference-to-once", true, true),
    ("once-keeps-external-reference", true, true),
    ("once-drops-owned-environment", true, true),
    ("mutable-borrow-preserves-environment", true, true),
    ("contextual-quantified-identity", true, true),
    ("quantified-identity", false, false),
    ("shared-rejects-mutable", false, false),
    ("mutable-rejects-once", false, false),
    ("once-still-consumes", false, false),
    ("mutable-reference-rejects-shared-alias", false, false),
    ("once-rejects-owned-reference-escape", false, false),
    ("false-output", false, false),
    ("crossed-output", false, false),
    ("noninjective-output", true, true),
    ("unused-binder", true, true),
    ("unused-binder-escape", false, false),
    ("reference-shared-call", true, true),
    ("reference-shared-mut-call", true, true),
    ("reference-mut-shared-reject", false, false),
    ("reference-shared-mut-reject", false, false),
    ("reference-shared-once-reject", false, false),
    ("reference-mut-once-reject", false, false),
    ("reference-shared-external-result", true, true),
    ("reference-mut-external-result", true, true),
    ("reference-shared-owned-drop", true, true),
    ("reference-mut-owned-drop", true, true),
    ("reference-static-escape", false, false),
    ("reference-nested-boundary", true, false),
];
fn fixture(name: &str) -> Result<String, std::io::Error> {
    std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join(format!("../pa24/tests/spec/receiver-adapters/{name}.rs")),
    )
}
#[test]
fn independent_receiver_adaptations() -> Result<(), Box<dyn std::error::Error>> {
    for &(name, accepted, _) in CASES {
        rustc::qualify(
            &format!("parallel/pa24/receiver-adapters/qualification/{name}"),
            &fixture(name)?,
            accepted,
        )?;
    }
    Ok(())
}
#[test]
fn declared_receiver_adaptations() -> Result<(), Box<dyn std::error::Error>> {
    let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(format!(
        "../obj/parallel/pa24/receiver-adapters/source/{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&folder)?;
    let mut failures = Vec::new();
    for &(name, _, accepted) in CASES {
        let path = folder.join(format!("{name}.rs"));
        std::fs::write(&path, fixture(name)?)?;
        let output = std::process::Command::new(env!("CARGO_BIN_EXE_rustgm"))
            .arg("--emit-lowir")
            .arg(&path)
            .output()?;
        std::fs::write(folder.join(format!("{name}.stderr")), &output.stderr)?;
        if output.status.code() != Some(i32::from(!accepted)) {
            failures.push(format!(
                "{name}: {:?}: {}",
                output.status,
                String::from_utf8_lossy(&output.stderr)
            ));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}
#[test]
#[ignore = "requires the qualified external CPPGM LowIR backend"]
fn owned_receiver_adaptations_execute() -> Result<(), Box<dyn std::error::Error>> {
    let mut failures = Vec::new();
    for &(name, _, accepted) in CASES {
        if !accepted {
            continue;
        }
        let source = fixture(name)?;
        if let Err(error) = native::check_cases(
            "parallel/pa24/receiver-adapters/native",
            &[(
                name,
                source
                    .strip_prefix(native::CHECK)
                    .ok_or("native check prefix")?,
                false,
            )],
        ) {
            failures.push(format!("{name}: {error}"));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}
