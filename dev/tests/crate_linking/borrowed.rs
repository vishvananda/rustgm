//! Reference signatures retain binder-local identities across separate compilations.
use super::{artifacts, crates};
use std::process::Command;

#[test]
fn borrowed_interfaces_preserve_output_slots_and_reject_invalid_clients()
-> Result<(), Box<dyn std::error::Error>> {
    let build = artifacts::build_borrowed("borrowed-portable")?;
    build.qualify()?;
    for metadata in [&build.first, &build.second] {
        let bytes = std::fs::read(metadata)?;
        assert_eq!(bytes[12], 2);
        let interface = crates::decode(&bytes)?;
        assert_eq!(bytes, crates::encode(&interface)?);
    }
    let ordinary = artifacts::checked("client", &build.folder, &mut build.client("client", false))?;
    let reverse = artifacts::checked(
        "client-reversed",
        &build.folder,
        &mut build.client("client", true),
    )?;
    assert_eq!(ordinary.stdout, reverse.stdout);
    for name in [
        "const-call",
        "dangling",
        "wrong-output-slot",
        "shared-to-mut",
        "unsafe-call",
        "exclusive-result",
    ] {
        let output = build.client(name, false).output()?;
        std::fs::write(build.folder.join(format!("{name}.stderr")), &output.stderr)?;
        assert_eq!(
            output.status.code(),
            Some(1),
            "{name}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        let diagnostic = String::from_utf8_lossy(&output.stderr);
        let reason = match name {
            "dangling" | "wrong-output-slot" => "required lifetime does not outlive its target",
            "shared-to-mut" => "cannot coerce a shared reference to a mutable reference",
            "unsafe-call" => "call to unsafe function requires an unsafe context",
            "exclusive-result" => "access conflicts with a live reference loan",
            _ => "non-const",
        };
        assert!(diagnostic.contains(reason), "{name}: {diagnostic}");
        let code = match name {
            "dangling" | "wrong-output-slot" => "E0515",
            "shared-to-mut" => "E0308",
            "unsafe-call" => "E0133",
            "exclusive-result" => "E0499",
            _ => "E0015",
        };
        build.qualify_client(name, Some(code))?;
    }
    Ok(())
}

#[test]
#[ignore = "requires configured CPPGM native backend and pinned Rust toolchain"]
fn borrowed_crate_values_execute() -> Result<(), Box<dyn std::error::Error>> {
    let build = artifacts::build_borrowed("borrowed-native")?;
    build.qualify()?;
    build.execute("client")
}

#[test]
fn explicit_early_bounds_remain_a_declared_profile_boundary()
-> Result<(), Box<dyn std::error::Error>> {
    let build = artifacts::build_borrowed("borrowed-early")?;
    for name in ["early-bound", "output-only"] {
        let independent = Command::new("rustc")
            .args([
                "+1.98.0",
                "--edition=2024",
                "--crate-type=lib",
                "--emit=obj",
                "-C",
                "panic=abort",
                "-C",
                "overflow-checks=yes",
            ])
            .arg(build.fixture(name))
            .arg("-o")
            .arg(build.folder.join(format!("{name}.o")))
            .output()?;
        std::fs::write(
            build.folder.join(format!("{name}.rustc.stderr")),
            &independent.stderr,
        )?;
        assert!(independent.status.success());
        let metadata = build.folder.join(format!("{name}.rgm"));
        let output = Command::new(env!("CARGO_BIN_EXE_rustgm"))
            .args([
                "--emit-lowir",
                "--crate-type=lib",
                "--crate-name",
                "early",
                "--crate-disambiguator",
                "bound",
                "--metadata-output",
            ])
            .arg(&metadata)
            .arg(build.fixture(name))
            .output()?;
        std::fs::write(build.folder.join(format!("{name}.stderr")), &output.stderr)?;
        assert_eq!(output.status.code(), Some(1));
        let diagnostic = String::from_utf8_lossy(&output.stderr);
        assert!(
            diagnostic.contains(
                "early lifetime parameters and explicit bounds are outside the metadata profile"
            ),
            "{name}: {diagnostic}"
        );
        assert!(!metadata.exists());
    }
    Ok(())
}
