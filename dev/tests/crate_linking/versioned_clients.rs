//! The actual driver publishes complete contracts before fresh CLI consumers run.
use std::{path::Path, process::Command};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

const DEPRECATED_PROVIDER: &str = r#"
#[stable(feature="deprecation_contract",since="1.0.0")]
#[deprecated(since="1.0.0",note="use the replacement")]
pub fn removed() {}
#[stable(feature="deprecation_contract",since="1.0.0")]
#[deprecated(since="1.0.0",note="use the current module")]
pub mod old {
    #[stable(feature="deprecation_contract",since="1.0.0")]
    pub fn inside() {}
}
#[stable(feature="deprecation_contract",since="1.0.0")]
#[deprecated(since="TBD",note="scheduled")]
pub fn scheduled() {}
#[stable(feature="deprecation_contract",since="1.0.0")]
#[deprecated(since="1.99.0",note="next release")]
pub fn next_release() {}
"#;

const RESERVATION_PROVIDER: &str = r#"
#[stable(feature="reservation_transport",since="1.0.0")]
#[rustc_const_unstable(feature="reservation_effect",issue="none")]
#[lang="From"]
pub const trait From<T>: Sized {
    #[stable(feature="reservation_transport",since="1.0.0")]
    fn from(value: T) -> Self;
}
#[stable(feature="reservation_transport",since="1.0.0")]
#[rustc_const_unstable(feature="reservation_effect",issue="none")]
#[rustc_reservation_impl="preserve future implementation"]
const impl<T> From<!> for T {
    fn from(value: !) -> T { value }
}
"#;

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

fn availability_foundation(root: &Path) -> Result<String> {
    let source = std::fs::read_to_string(
        root.join("pa31/tests/spec/source-availability-transport/provider.rs"),
    )?;
    Ok(source
        .split_once("\n#[unstable(feature=\"experimental_identity\"")
        .ok_or("source availability fixture lost its role-only prefix")?
        .0
        .to_owned())
}

fn source_client(source: &Path, metadata: &Path, lowir: &Path) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_rustgm"));
    command
        .args([
            "--emit-lowir",
            "--edition=2024",
            "--crate-name",
            "client",
            "--crate-disambiguator",
            "exact_cli",
            "--source-client",
            "base@exact_cli",
            "--extern",
        ])
        .arg(format!("base={}", metadata.display()))
        .arg("-o")
        .arg(lowir)
        .arg(source);
    command
}

#[test]
fn cli_version_fourteen_preserves_actual_deprecation_and_client_policy() -> Result {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?;
    let folder = root.join(format!(
        "obj/parallel/deprecation-metadata-v14/cli-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&folder)?;
    let foundation = availability_foundation(root)?;
    let source = folder.join("provider.rs");
    std::fs::write(&source, format!("{foundation}\n{DEPRECATED_PROVIDER}"))?;
    let metadata = folder.join("provider.rgcrate");
    let mut command = source_export(
        "module-body-operations-v14",
        &source,
        &metadata,
        &folder.join("provider.lowir"),
        true,
    );
    command.arg("--cap-lints=allow");
    super::artifacts::checked("provider", &folder, &mut command)?;
    let bytes = std::fs::read(&metadata)?;
    assert_eq!(
        rustgm::support::crates::encode(&rustgm::support::crates::decode(&bytes)?)?,
        bytes
    );
    for (case, policy, callee, diagnostic) in [
        ("active", "deny", "removed", Some("use the replacement")),
        (
            "inherited",
            "deny",
            "old::inside",
            Some("use the current module"),
        ),
        ("tbd", "deny", "scheduled", None),
        ("next", "deny", "next_release", None),
        ("allow", "allow", "removed", None),
        ("expect", "expect", "removed", None),
    ] {
        let source = folder.join(format!("{case}.rs"));
        let text = format!(
            "#![feature(no_core)]#![no_core]#![deny(unfulfilled_lint_expectations)]\n\
             extern crate base;#[{policy}(deprecated)]fn main(){{base::{callee}();}}"
        );
        std::fs::write(&source, text)?;
        let output =
            source_client(&source, &metadata, &folder.join(format!("{case}.lowir"))).output()?;
        let stderr = String::from_utf8(output.stderr)?;
        assert_eq!(
            output.status.code(),
            Some(i32::from(diagnostic.is_some())),
            "{case}: {stderr}"
        );
        if let Some(note) = diagnostic {
            assert!(
                stderr.contains(&format!("use of deprecated declaration: {note}")),
                "{case}: {stderr}"
            );
        } else {
            assert!(stderr.is_empty(), "{case}: {stderr}");
        }
    }
    Ok(())
}

#[test]
fn cli_version_fourteen_preserves_const_reservation_for_coherence() -> Result {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?;
    let folder = root.join(format!(
        "obj/parallel/reservation-polarity-v14/cli-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&folder)?;
    let foundation = availability_foundation(root)?;
    let provider = folder.join("provider.rs");
    std::fs::write(
        &provider,
        format!("#![feature(never_type,const_trait_impl)]\n{foundation}\n{RESERVATION_PROVIDER}"),
    )?;
    let metadata = folder.join("provider.rgcrate");
    let mut command = source_export(
        "module-body-operations-v14",
        &provider,
        &metadata,
        &folder.join("provider.lowir"),
        true,
    );
    command.arg("--cap-lints=allow");
    super::artifacts::checked("provider", &folder, &mut command)?;
    let bytes = std::fs::read(&metadata)?;
    assert_eq!(
        rustgm::support::crates::encode(&rustgm::support::crates::decode(&bytes)?)?,
        bytes
    );
    for (case, body, diagnostic) in [
        ("unused", "fn main() {}", None),
        (
            "use",
            "struct Local; fn require<T: base::From<!>>() {} fn main() { require::<Local>(); }",
            Some("trait obligation requires an unambiguous matching assumption or implementation"),
        ),
        (
            "overlap",
            "struct Local; trait Marker {} impl<T: base::From<!>> Marker for T {} impl Marker for Local {} fn main() {}",
            Some("overlapping trait implementation headers require coherence proof"),
        ),
    ] {
        let client = folder.join(format!("{case}.rs"));
        std::fs::write(
            &client,
            format!("#![feature(never_type,no_core)]#![no_core]\n{body}"),
        )?;
        let output =
            source_client(&client, &metadata, &folder.join(format!("{case}.lowir"))).output()?;
        let stderr = String::from_utf8(output.stderr)?;
        assert_eq!(
            output.status.code(),
            Some(i32::from(diagnostic.is_some())),
            "{case}: {stderr}"
        );
        if let Some(message) = diagnostic {
            assert!(stderr.contains(message), "{case}: {stderr}");
        } else {
            assert!(stderr.is_empty(), "{case}: {stderr}");
        }
    }
    Ok(())
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

#[test]
fn cli_version_seven_keeps_written_effects_and_old_profile_boundaries() -> Result {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?;
    let folder = root.join(format!(
        "obj/parallel/source-callable-effects-v1/cli-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&folder)?;
    let fixture = root.join("pa31/tests/spec/source-callable-effects/provider.rs");
    let metadata = folder.join("provider.rgcrate");
    let lowir = folder.join("provider.lowir");
    let output = super::artifacts::checked(
        "version-seven",
        &folder,
        &mut source_export("module-callables-v7", &fixture, &metadata, &lowir, true),
    )?;
    assert!(output.stderr.is_empty());
    let bytes = std::fs::read(&metadata)?;
    assert_eq!(bytes.get(13..15), Some([7, 0].as_slice()));
    assert_eq!(
        rustgm::support::crates::encode(&rustgm::support::crates::decode(&bytes)?)?,
        bytes
    );
    let rejected = folder.join("older.rgcrate");
    let output = source_export(
        "module-compatibility-v6",
        &fixture,
        &rejected,
        &folder.join("older.lowir"),
        true,
    )
    .output()?;
    assert_eq!(
        output.status.code(),
        Some(1),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(!rejected.exists());
    Ok(())
}

#[test]
fn cli_source_client_feature_requests_keep_plain_library_exports() -> Result {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?;
    let folder = root.join(format!(
        "obj/parallel/source-callable-effects-v1/consumer-cli-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&folder)?;
    let fixtures = root.join("pa31/tests/spec/source-callable-effects");
    let provider = folder.join("provider.rgcrate");
    let output = source_export(
        "module-callables-v7",
        &fixtures.join("combined-provider.rs"),
        &provider,
        &folder.join("provider.lowir"),
        true,
    )
    .output()?;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let metadata = folder.join("client.rgcrate");
    let output = Command::new(env!("CARGO_BIN_EXE_rustgm"))
        .args([
            "--emit-lowir",
            "--crate-type=lib",
            "--crate-name",
            "observer",
            "--crate-disambiguator",
            "native",
            "--source-client",
            "base@exact_cli",
            "--extern",
        ])
        .arg(format!("base={}", provider.display()))
        .arg("--metadata-output")
        .arg(&metadata)
        .arg("-o")
        .arg(folder.join("client.lowir"))
        .arg(fixtures.join("native-client.rs"))
        .output()?;
    std::fs::write(folder.join("client.stderr"), &output.stderr)?;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let warnings = String::from_utf8(output.stderr)?;
    assert_eq!(warnings.matches("warning[deprecated]:").count(), 2);
    let bytes = std::fs::read(metadata)?;
    assert_eq!(
        rustgm::support::crates::encode(&rustgm::support::crates::decode(&bytes)?)?,
        bytes
    );
    Ok(())
}

#[test]
fn cli_version_eight_retains_actual_unsafe_calls_and_legacy_denials() -> Result {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?;
    let folder = root.join(format!(
        "obj/parallel/source-callable-effects-v1/safety-cli-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&folder)?;
    let fixture = root.join("pa31/tests/spec/source-call-safety/preserved-provider.rs");
    let metadata = folder.join("provider.rgcrate");
    let lowir = folder.join("provider.lowir");
    let output = super::artifacts::checked(
        "version-eight",
        &folder,
        &mut source_export("module-safety-v8", &fixture, &metadata, &lowir, true),
    )?;
    assert!(output.stderr.is_empty());
    let bytes = std::fs::read(&metadata)?;
    assert_eq!(bytes.get(13..15), Some([8, 0].as_slice()));
    assert_eq!(
        rustgm::support::crates::encode(&rustgm::support::crates::decode(&bytes)?)?,
        bytes
    );
    let rejected = folder.join("older.rgcrate");
    let output = source_export(
        "module-callables-v7",
        &fixture,
        &rejected,
        &folder.join("older.lowir"),
        true,
    )
    .output()?;
    assert_eq!(
        output.status.code(),
        Some(1),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(!rejected.exists());
    Ok(())
}
