//! The real driver selects a source client from authenticated artifact identities.
use super::artifacts;
use std::{
    path::{Path, PathBuf},
    process::Command,
};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const PROVIDER: &str = "minicore@source_cli";
fn command() -> Command {
    Command::new(env!("CARGO_BIN_EXE_rustgm"))
}
fn output() -> Result<PathBuf> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?;
    let folder = root
        .join("obj/parallel/pa29-source-client-staging/lib007/packet-cli")
        .join(std::process::id().to_string());
    std::fs::create_dir_all(&folder)?;
    Ok(folder)
}
#[test]
fn cli_source_client_flags_reject_missing_identity_and_conflicting_provider_mode() -> Result {
    let folder = output()?;
    for (name, args, message) in [
        (
            "missing-value",
            vec!["--source-client"],
            "--source-client requires NAME@DISAMBIGUATOR",
        ),
        (
            "missing-current",
            vec!["--emit-lowir", "--source-client", PROVIDER],
            "complete current crate identity",
        ),
        (
            "conflicting",
            vec![
                "--emit-lowir",
                "--source-core",
                PROVIDER,
                "--source-client",
                PROVIDER,
            ],
            "mutually exclusive",
        ),
        (
            "same-current",
            vec![
                "--emit-lowir",
                "--crate-name",
                "minicore",
                "--crate-disambiguator",
                "source_cli",
                "--source-client",
                PROVIDER,
            ],
            "must differ",
        ),
    ] {
        let result = command().args(args).output()?;
        std::fs::write(folder.join(format!("{name}.stderr")), &result.stderr)?;
        assert!(!result.status.success(), "{name}");
        assert!(
            String::from_utf8_lossy(&result.stderr).contains(message),
            "{name}: {}",
            String::from_utf8_lossy(&result.stderr)
        );
    }
    Ok(())
}
fn build(folder: &Path) -> Result {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?;
    let fixtures = root.join("pa31/tests/spec/source-functions-transport");
    let metadata = folder.join("provider.rgm");
    artifacts::checked(
        "provider",
        folder,
        command()
            .args([
                "--emit-lowir",
                "--crate-type=lib",
                "--crate-name",
                "minicore",
                "--crate-disambiguator",
                "source_cli",
                "--source-core",
                PROVIDER,
                "--metadata-output",
            ])
            .arg(&metadata)
            .arg("-o")
            .arg(folder.join("provider.lowir"))
            .arg(fixtures.join("minimal-provider.rs")),
    )?;
    let bytes = std::fs::read(&metadata)?;
    let decoded = rustgm::support::crates::decode(&bytes)?;
    assert_eq!(bytes, rustgm::support::crates::encode(&decoded)?);
    assert_eq!(
        decoded
            .source_descriptor()
            .ok_or("source descriptor")?
            .profile(),
        rustgm::support::crates::SourceProfile::Source2024Functions
    );
    artifacts::checked(
        "client",
        folder,
        command()
            .args([
                "--emit-lowir",
                "--crate-type=lib",
                "--crate-name",
                "client",
                "--crate-disambiguator",
                "source_cli_client",
                "--source-client",
                PROVIDER,
                "--extern",
            ])
            .arg(format!("base={}", metadata.display()))
            .arg("-o")
            .arg(folder.join("client.lowir"))
            .arg(fixtures.join("runtime-client.rs")),
    )?;
    Ok(())
}
#[test]
#[ignore = "requires configured pinned object backend and host C compiler; fixture already independently Rust-qualified"]
fn cli_source_artifact_to_fresh_native_consumer_returns_seven() -> Result {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?;
    let folder = output()?;
    build(&folder)?;
    artifacts::checked(
        "backend",
        &folder,
        Command::new("python3")
            .arg(root.join("scripts/backend.py"))
            .arg("--emit-object")
            .arg(folder.join("client.lowir"))
            .arg("-o")
            .arg(folder.join("client.o")),
    )?;
    artifacts::checked(
        "host-link",
        &folder,
        Command::new("cc")
            .args(["-std=c11", "-Wall", "-Wextra", "-Werror"])
            .arg(root.join("pa31/tests/spec/source-functions-transport/native-driver.c"))
            .arg(folder.join("client.o"))
            .arg("-o")
            .arg(folder.join("client.native")),
    )?;
    let result = artifacts::checked(
        "execute",
        &folder,
        &mut Command::new(folder.join("client.native")),
    )?;
    std::fs::write(folder.join("execute.stdout"), &result.stdout)?;
    assert_eq!(result.stdout, b"7\n");
    Ok(())
}
