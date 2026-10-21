//! Direct capture endpoints retain ordinary body, borrow and cleanup proofs.
#[path = "environment_execution/adapters.rs"]
mod adapters;
#[path = "environment_execution/adjusted.rs"]
mod adjusted;
#[path = "environment_execution/chains.rs"]
mod chains;
#[path = "harness/checked_sources.rs"]
mod checked_sources;
#[path = "environment_execution/contextual.rs"]
mod contextual;
#[path = "environment_execution/hierarchy.rs"]
mod hierarchy;
#[path = "harness/native_references.rs"]
mod native;
#[path = "environment_execution/nested.rs"]
mod nested;
#[path = "environment_execution/conversion_context.rs"]
mod object_inputs;
#[path = "environment_execution/dependent.rs"]
mod projected;
#[path = "environment_execution/copy_inputs.rs"]
mod reference_copies;
#[path = "environment_execution/references.rs"]
mod references;
#[path = "helpers/qualify.rs"]
mod rustc;
#[path = "support/semantic.rs"]
mod semantic;
#[path = "environment_execution/signatures.rs"]
mod signatures;
#[path = "environment_execution/slot_ownership.rs"]
mod slot_ownership;
#[path = "environment_execution/specialization.rs"]
mod specialization;

const CASES: &[(&str, &str, bool)] = &[
    ("wildcard-reference-loan", "wildcard-reference-loan", false),
    ("mutable-reference", "mutable-reference", true),
    ("mutable-reference-field", "mutable-reference-field", true),
    (
        "shared-reference-mutation",
        "shared-reference-mutation",
        false,
    ),
    ("immutable-owned-field", "immutable-owned-field", false),
    ("wildcard-reference-ended", "wildcard-reference-ended", true),
    ("once-consumed-cleanup", "once-consumed-cleanup", true),
    ("wildcard-live-loan", "wildcard-live-loan", true),
    ("wildcard-moved", "wildcard-moved", false),
    (
        "wildcard-plain-uninitialized",
        "wildcard-plain-uninitialized",
        true,
    ),
    ("wildcard-initialized", "wildcard-initialized", true),
    ("wildcard-uninitialized", "wildcard-uninitialized", false),
    ("shared-capture", "shared-capture", true),
    ("mutable-capture", "mutable-capture", true),
    ("move-once", "move-once", true),
    ("move-still-fn", "move-still-fn", true),
    ("unique-immutable", "unique-immutable", true),
    ("precise-field-borrow", "precise-field-borrow", true),
    ("precise-field-move", "precise-field-move", true),
    ("borrow-ended", "borrow-ended", true),
    ("drop-capture", "drop-capture", true),
    ("borrow-conflict", "borrow-conflict", false),
    (
        "unique-immutable-conflict",
        "unique-immutable-conflict",
        false,
    ),
    ("move-twice", "move-twice", false),
    (
        "immutable-binding-cannot-call-mut",
        "immutable-binding-cannot-call-mut",
        false,
    ),
    ("captured-to-fn-pointer", "captured-to-fn-pointer", false),
    (
        "zst-captured-to-fn-pointer",
        "zst-captured-to-fn-pointer",
        false,
    ),
    ("returned-borrow", "returned-borrow", false),
];
fn fixture(stem: &str) -> Result<String, std::io::Error> {
    std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join(format!("../pa24/tests/spec/captures/{stem}.rs")),
    )
}
#[test]
fn declared_output_lifetime_results() -> Result<(), Box<dyn std::error::Error>> {
    let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(format!(
        "../obj/parallel/pa24/capture-execution/output-boundaries/{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&folder)?;
    for name in ["reference-return", "once-partial-cleanup"] {
        let source = fixture(name)?;
        rustc::qualify(
            &format!("parallel/pa24/capture-execution/qualification/{name}"),
            &source,
            true,
        )?;
        let path = folder.join(format!("{name}.rs"));
        std::fs::write(&path, &source)?;
        let output = std::process::Command::new(env!("CARGO_BIN_EXE_rustgm"))
            .arg("--emit-lowir")
            .arg(path)
            .output()?;
        std::fs::write(folder.join(format!("{name}.stderr")), &output.stderr)?;
        assert_eq!(
            output.status.code(),
            Some(0),
            "{name}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    Ok(())
}
#[test]
fn independent_direct_execution_outcomes() -> Result<(), Box<dyn std::error::Error>> {
    for &(name, stem, accepted) in CASES {
        let source = fixture(stem)?;
        rustc::qualify(
            &format!("parallel/pa24/capture-execution/qualification/{name}"),
            &source,
            accepted,
        )?;
    }
    Ok(())
}
#[test]
fn direct_execution_outcomes() -> Result<(), Box<dyn std::error::Error>> {
    let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(format!(
        "../obj/parallel/pa24/capture-execution/source/{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&folder)?;
    let mut failures = Vec::new();
    for &(name, stem, accepted) in CASES {
        let source = fixture(stem)?;
        let path = folder.join(format!("{name}.rs"));
        std::fs::write(&path, &source)?;
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
#[ignore = "requires the pinned CPPGM backend; run make test-maintainer-native"]
fn native_direct_execution() -> Result<(), Box<dyn std::error::Error>> {
    let sources = CASES
        .iter()
        .filter(|case| case.2)
        .map(|&(name, stem, _)| fixture(stem).map(|source| (name, source)))
        .chain(
            ["reference-return", "once-partial-cleanup"]
                .into_iter()
                .map(|name| fixture(name).map(|source| (name, source))),
        )
        .collect::<Result<Vec<_>, _>>()?;
    let cases = sources
        .iter()
        .map(|(name, source)| {
            (
                *name,
                source.strip_prefix(native::CHECK).unwrap_or(source),
                false,
            )
        })
        .collect::<Vec<_>>();
    native::check_cases("parallel/pa24/capture-execution/native", &cases)
}
