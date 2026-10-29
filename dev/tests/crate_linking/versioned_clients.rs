//! The actual driver publishes complete contracts before fresh CLI consumers run.
use std::{path::Path, process::Command};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

fn qualify(native: bool) -> Result {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?;
    let lane = if native { "native" } else { "frontend" };
    let evidence = root
        .join("obj/parallel/pa31/source-cli-contracts")
        .join(format!("{lane}-{}", std::process::id()));
    std::fs::create_dir_all(evidence.parent().ok_or("evidence parent")?)?;
    let mut command = Command::new("python3");
    command
        .arg(root.join("scripts/check_source_attributes.py"))
        .arg("--app")
        .arg(env!("CARGO_BIN_EXE_rustgm"))
        .arg("--evidence-root")
        .arg(&evidence);
    if native {
        command.arg("--native");
    }
    let result = command.output()?;
    std::fs::write(evidence.with_extension("stdout"), &result.stdout)?;
    std::fs::write(evidence.with_extension("stderr"), &result.stderr)?;
    if !result.status.success() {
        return Err(format!(
            "source CLI contracts: {:?}\n{}",
            result.status,
            String::from_utf8_lossy(&result.stderr)
        )
        .into());
    }
    if !result.stderr.is_empty() {
        return Err("source CLI contract harness emitted an undeclared diagnostic".into());
    }
    Ok(())
}

#[test]
fn cli_function_contracts_reach_fresh_consumers_with_exact_diagnostics() -> Result {
    qualify(false)
}

#[test]
#[ignore = "requires prebuilt locked object backend, pinned Rust and host C compiler"]
fn cli_function_contracts_reach_fresh_native_consumers_returning_seven() -> Result {
    qualify(true)
}

fn source_export(
    profile: &str,
    fixture: &Path,
    metadata: &Path,
    lowir: &Path,
    provider: bool,
) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_rustgm"));
    command.args([
        "--emit-lowir",
        "--crate-type=lib",
        "--crate-name",
        "base",
        "--crate-disambiguator",
        "exact_cli",
        "--source-profile",
        profile,
    ]);
    if provider {
        command.args(["--source-core", "base@exact_cli"]);
    }
    command
        .arg("--metadata-output")
        .arg(metadata)
        .arg("-o")
        .arg(lowir)
        .arg(fixture);
    command
}
#[test]
fn cli_version_five_keeps_exact_c_publication_and_source_provider_boundaries() -> Result {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?;
    let folder = root.join(format!(
        "obj/parallel/exact-source-linkage-v5/cli-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&folder)?;
    let fixture = root.join("pa31/tests/spec/exact-source-linkage/provider.rs");
    let metadata = folder.join("provider.rgcrate");
    let lowir = folder.join("provider.lowir");
    let output = super::artifacts::checked(
        "version-five",
        &folder,
        &mut source_export("module-definitions-v5", &fixture, &metadata, &lowir, true),
    )?;
    assert!(output.stderr.is_empty());
    let bytes = std::fs::read(&metadata)?;
    assert_eq!(bytes.get(13..15), Some([5, 0].as_slice()));
    assert_eq!(
        rustgm::support::crates::encode(&rustgm::support::crates::decode(&bytes)?)?,
        bytes
    );
    for (profile, provider, expected) in [
        ("module-definitions-v5", false, 2),
        ("module-availability-v3", true, 1),
    ] {
        let rejected_metadata = folder.join(format!("rejected-{expected}.rgcrate"));
        let output = source_export(
            profile,
            &fixture,
            &rejected_metadata,
            &folder.join(format!("rejected-{expected}.lowir")),
            provider,
        )
        .output()?;
        assert_eq!(
            output.status.code(),
            Some(expected),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(!rejected_metadata.exists());
    }
    Ok(())
}
