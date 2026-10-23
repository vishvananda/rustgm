//! Actual provider originals survive the artifact boundary and ordinary instance proof.
use super::{artifacts, exports, imports};
use rustgm::{
    lower, pipeline,
    support::crates::{self, Dependencies, Dependency},
};
use std::{
    path::{Path, PathBuf},
    process::Command,
    sync::Arc,
};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
fn fixture(name: &str) -> Result<String> {
    Ok(std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join(format!("../pa29/tests/spec/generic-contracts/{name}.rs")),
    )?)
}
fn provider() -> Result<crates::Interface> {
    let (_, interface) = exports::compile(&fixture("minimal-id-library")?, "generic_originals")?;
    let payload = interface
        .generic_payload()
        .ok_or("generic original export was omitted")?;
    assert_eq!(payload.rows().functions.len(), 1);
    assert_eq!(payload.rows().parameters.len(), 1);
    assert!(interface.functions().is_empty());
    let encoded = crates::encode(&interface)?;
    let decoded = crates::decode(&encoded)?;
    assert_eq!(encoded, crates::encode(&decoded)?);
    Ok(decoded)
}
#[test]
fn actual_generic_originals_execute_runtime_and_constant_instances_after_roundtrip() -> Result {
    let interface = provider()?;
    let identity = interface.identity().clone();
    let dependencies = Arc::new(Dependencies::new(
        vec![interface],
        vec![Dependency::new("base".into(), identity)?],
    )?);
    for name in ["pass-id", "pass-imported-id-const"] {
        let mut report = imports::resolve(&fixture(name)?, Arc::clone(&dependencies))?;
        let pipeline::Analysis {
            mut facts,
            prepared,
        } = pipeline::analyze_source(&report)?;
        let entry = rustgm::sema::entry::select(&mut report, &facts)?;
        lower::instances::build_prepared(&report, &mut facts, entry, prepared)?;
        let imported = report
            .resolution
            .crates
            .as_ref()
            .ok_or("missing actual imported declarations")?;
        for (external, definition) in imported.definitions() {
            if external.path().last().is_some_and(|name| name == "id") {
                assert!(
                    report.resolution.definitions[definition.0 as usize]
                        .node
                        .is_none()
                );
                assert!(!facts.functions.iter().any(|f| f.definition == definition));
            }
        }
    }
    let rejected = imports::resolve(&fixture("reject-copy")?, dependencies)?;
    let outcome = pipeline::analyze_source(&rejected);
    assert!(
        outcome.is_err(),
        "imported declared Copy input was not enforced"
    );
    assert!(exports::compile(&fixture("reject-unused-source")?, "invalid_unused").is_err());
    Ok(())
}
#[test]
fn actual_generic_artifacts_are_deterministic_and_source_only_export_cannot_drop_requests() -> Result
{
    let first = provider()?;
    assert_eq!(crates::encode(&first)?, crates::encode(&provider()?)?);
    let report = crate::resolve::resolve(&fixture("minimal-id-library")?, true)?;
    let mut facts = pipeline::analyze_source(&report)?.facts;
    let result = rustgm::sema::crates::export(&report, &mut facts, first.identity().clone());
    assert!(
        result.is_err(),
        "missing original provider silently emitted an empty interface"
    );
    Ok(())
}
#[test]
fn imported_formal_flags_pay_unused_sizedness_and_preserve_relaxation() -> Result {
    for (name, relaxed) in [
        ("minimal-unused-library", false),
        ("minimal-unsized-library", true),
    ] {
        let (_, interface) = exports::compile(&fixture(name)?, "formal_flags")?;
        let interface = crates::decode(&crates::encode(&interface)?)?;
        let identity = interface.identity().clone();
        let dependencies = Arc::new(Dependencies::new(
            vec![interface],
            vec![Dependency::new("base".into(), identity)?],
        )?);
        let mut report = imports::resolve(&fixture("unused-unsized-argument")?, dependencies)?;
        let result = pipeline::analyze_source(&report);
        if relaxed {
            let pipeline::Analysis {
                mut facts,
                prepared,
            } = result?;
            let entry = rustgm::sema::entry::select(&mut report, &facts)?;
            lower::instances::build_prepared(&report, &mut facts, entry, prepared)?;
        } else {
            assert!(
                result.is_err(),
                "unused default Sized argument was accepted"
            );
        }
    }
    Ok(())
}
fn output_root() -> Result<PathBuf> {
    let folder = Path::new(env!("CARGO_MANIFEST_DIR")).join(format!(
        "../obj/parallel/pa29-crate-linking/actual-generics-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&folder)?;
    Ok(folder)
}
#[test]
#[ignore = "requires configured CPPGM native backend; original sources independently Rust-qualified in generic-id-outcomes.json"]
fn imported_generic_identity_runs_native_runtime_and_constant_calls() -> Result {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?;
    let folder = output_root()?;
    let metadata = folder.join("base.rgm");
    artifacts::checked(
        "generic-provider",
        &folder,
        Command::new(env!("CARGO_BIN_EXE_rustgm"))
            .args([
                "--emit-lowir",
                "--crate-type=lib",
                "--crate-name",
                "base",
                "--crate-disambiguator",
                "generic_native",
                "--metadata-output",
            ])
            .arg(&metadata)
            .arg("-o")
            .arg(folder.join("base.lowir"))
            .arg(root.join("pa29/tests/spec/generic-contracts/minimal-id-library.rs")),
    )?;
    for name in ["pass-id", "pass-imported-id-const"] {
        let lowir = folder.join(format!("{name}.lowir"));
        artifacts::checked(
            name,
            &folder,
            Command::new(env!("CARGO_BIN_EXE_rustgm"))
                .arg("--emit-lowir")
                .arg(root.join(format!("pa29/tests/spec/generic-contracts/{name}.rs")))
                .arg("--extern")
                .arg(format!("base={}", metadata.display()))
                .arg("-o")
                .arg(&lowir),
        )?;
        let native = folder.join(format!("{name}.native"));
        artifacts::checked(
            &format!("{name}.backend"),
            &folder,
            Command::new("python3")
                .arg(root.join("scripts/backend.py"))
                .arg(lowir)
                .arg("-o")
                .arg(&native),
        )?;
        artifacts::checked(
            &format!("{name}.execute"),
            &folder,
            &mut Command::new(native),
        )?;
    }
    Ok(())
}
