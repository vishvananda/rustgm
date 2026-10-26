//! Real v2 metadata is written by one process and consumed by a fresh source client.
use super::artifacts;
use std::{
    path::{Path, PathBuf},
    process::Command,
};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const PROVIDER: &str = "minicore@module_cli";
fn command() -> Command {
    Command::new(env!("CARGO_BIN_EXE_rustgm"))
}
fn root() -> Result<PathBuf> {
    Ok(Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?
        .to_owned())
}
fn provider(folder: &Path, name: &str, fixture: &str, version_two: bool) -> Result<PathBuf> {
    let metadata = folder.join(format!("{name}.rgm"));
    let mut cmd = command();
    cmd.args([
        "--emit-lowir",
        "--crate-type=lib",
        "--crate-name",
        "minicore",
        "--crate-disambiguator",
        "module_cli",
        "--source-core",
        PROVIDER,
        "--metadata-output",
    ])
    .arg(&metadata)
    .arg("-o")
    .arg(folder.join(format!("{name}.lowir")));
    if version_two {
        cmd.args(["--source-profile", "module-contracts-v2"]);
    }
    cmd.arg(root()?.join(fixture));
    artifacts::checked(name, folder, &mut cmd)?;
    let bytes = std::fs::read(&metadata)?;
    let interface = rustgm::support::crates::decode(&bytes)?;
    assert_eq!(rustgm::support::crates::encode(&interface)?, bytes);
    assert_eq!(
        bytes.get(13..15),
        Some([if version_two { 2 } else { 1 }, 0].as_slice())
    );
    Ok(metadata)
}
fn client(
    folder: &Path,
    metadata: &Path,
    name: &str,
    mode: &str,
    source: &str,
    expected_status: i32,
    message: &str,
) -> Result {
    let input = folder.join(format!("{name}.rs"));
    std::fs::write(&input, source)?;
    let mut cmd = command();
    cmd.args([
        mode,
        "--crate-type=lib",
        "--crate-name",
        "client",
        "--crate-disambiguator",
        "module_cli_client",
        "--source-client",
        PROVIDER,
        "--extern",
    ])
    .arg(format!("base={}", metadata.display()))
    .arg("-o")
    .arg(folder.join(format!(
        "{name}.{}",
        if mode == "--emit-types" {
            "types"
        } else {
            "lowir"
        }
    )))
    .arg(input);
    let output = cmd.output()?;
    std::fs::write(folder.join(format!("{name}.stderr")), &output.stderr)?;
    assert_eq!(
        output.status.code(),
        Some(expected_status),
        "{name}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    if !message.is_empty() {
        assert!(
            String::from_utf8_lossy(&output.stderr).contains(message),
            "{name}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    Ok(())
}
#[test]
fn actual_v2_artifacts_resolve_nested_aliases_and_separate_foreign_impl_permission() -> Result {
    let root = root()?;
    let folder = root.join(format!(
        "obj/parallel/pa31/module-cli/{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&folder)?;
    let plain = provider(
        &folder,
        "legacy-plain",
        "pa31/tests/spec/source-functions-transport/minimal-provider.rs",
        false,
    )?;
    client(
        &folder,
        &plain,
        "ordinary",
        "--emit-lowir",
        "#![feature(no_core)]#![no_core]extern crate base;pub fn keep()->u8{base::identity(7u8)}",
        0,
        "",
    )?;
    let restricted = provider(
        &folder,
        "restricted",
        "pa31/tests/spec/module-contracts-v2/restricted-provider.rs",
        true,
    )?;
    restricted_clients(&root, &folder, &restricted)?;
    let open = provider(
        &folder,
        "open",
        "pa31/tests/spec/module-contracts-v2/namespace-provider.rs",
        true,
    )?;
    let source = std::fs::read_to_string(
        root.join("pa31/tests/spec/module-contracts-v2/nested-fresh-impl.rs"),
    )?;
    client(
        &folder,
        &open,
        "alias-local-impl",
        "--emit-lowir",
        &source,
        0,
        "",
    )?;
    client(
        &folder,
        &open,
        "no-const",
        "--emit-lowir",
        "#![feature(no_core)]#![no_core]extern crate base;pub const BAD:u8=base::identity(7u8);",
        1,
        "const",
    )?;
    let nested = provider(
        &folder,
        "nested-call",
        "pa31/tests/spec/module-contracts-v2/nested-call-provider.rs",
        true,
    )?;
    client(
        &folder,
        &nested,
        "nested-original",
        "--emit-lowir",
        "#![feature(no_core)]#![no_core]extern crate base;pub fn keep()->u8{base::api::renamed(7u8)}",
        0,
        "",
    )?;
    rejected_publication_and_flags(&root, &folder)?;
    Ok(())
}

fn restricted_clients(root: &Path, folder: &Path, restricted: &Path) -> Result {
    for (name, file, mode, expected_status, message) in [
        (
            "provider-impl",
            "select-provider-impl",
            "--emit-lowir",
            0,
            "",
        ),
        (
            "public-bound",
            "sealed-public-bound",
            "--emit-types",
            2,
            "--source-client requires --emit-lowir",
        ),
        (
            "foreign-impl",
            "sealed-foreign-impl",
            "--emit-lowir",
            1,
            "restricted module",
        ),
        (
            "foreign-feature",
            "sealed-foreign-feature-does-not-override",
            "--emit-lowir",
            1,
            "restricted module",
        ),
        (
            "private-path",
            "sealed-private-path-denied",
            "--emit-lowir",
            1,
            "private",
        ),
    ] {
        let source = std::fs::read_to_string(
            root.join(format!("pa31/tests/spec/module-contracts-v2/{file}.rs")),
        )?;
        client(
            folder,
            restricted,
            name,
            mode,
            &source,
            expected_status,
            message,
        )?;
    }
    Ok(())
}

fn rejected_publication_and_flags(root: &Path, folder: &Path) -> Result {
    let denied = folder.join("legacy-restricted.rgm");
    let output = command()
        .args([
            "--emit-lowir",
            "--crate-type=lib",
            "--crate-name",
            "minicore",
            "--crate-disambiguator",
            "module_cli",
            "--source-core",
            PROVIDER,
            "--metadata-output",
        ])
        .arg(&denied)
        .arg(root.join("pa31/tests/spec/module-contracts-v2/restricted-provider.rs"))
        .output()?;
    assert!(!output.status.success());
    assert!(!denied.exists());
    for args in [
        vec!["--source-profile", "module-contracts-v2"],
        vec!["--source-profile", "unknown"],
        vec![
            "--source-profile",
            "module-contracts-v2",
            "--source-profile",
            "module-contracts-v2",
        ],
    ] {
        assert!(!command().args(args).output()?.status.success());
    }
    Ok(())
}
