#[path = "harness/crate_artifacts.rs"]
mod artifacts;
#[path = "crate_linking/borrowed.rs"]
mod borrowed;
#[path = "crate_linking/convention_transport.rs"]
mod convention_transport;
#[path = "crate_linking/direct_calls.rs"]
mod direct_calls;
#[path = "crate_linking/exports.rs"]
mod exports;
#[path = "crate_linking/imported_records.rs"]
mod imported_records;
#[path = "crate_linking/imported_regions.rs"]
mod imported_regions;
#[path = "crate_linking/imports.rs"]
mod imports;
#[path = "crate_linking/instantiable.rs"]
mod instantiable;
#[path = "crate_linking/named_values.rs"]
mod named_values;
#[path = "crate_linking/records.rs"]
mod records;
#[path = "crate_linking/reexports.rs"]
mod reexports;
#[path = "crate_linking/representations.rs"]
mod representations;
#[path = "crate_linking/static_transport.rs"]
mod static_transport;
#[path = "crate_linking/transitive.rs"]
mod transitive;
#[path = "crate_linking/variadic_transport.rs"]
mod variadic_transport;
use rustgm::support::crates;
use std::process::Command;

#[test]
fn cli_libraries_and_imports_are_reproducible_and_reject_incomplete_artifacts()
-> Result<(), Box<dyn std::error::Error>> {
    let build = artifacts::build("portable")?;
    let first_bytes = std::fs::read(&build.first)?;
    let first = crates::decode(&first_bytes)?;
    let second = crates::decode(&std::fs::read(&build.second)?)?;
    assert_eq!(second.dependencies(), &[first.identity().clone()]);
    assert_eq!(first.identity().name(), second.identity().name());
    assert_ne!(first.identity(), second.identity());
    let normal = artifacts::checked("client", &build.folder, &mut build.client("client", false))?;
    let reordered = artifacts::checked(
        "reordered",
        &build.folder,
        &mut build.client("client", true),
    )?;
    assert_eq!(normal.stdout, reordered.stdout);
    let copy = artifacts::build("repeat")?;
    assert_eq!(first_bytes, std::fs::read(copy.first)?);
    assert_eq!(
        std::fs::read(build.folder.join("first.lowir"))?,
        std::fs::read(copy.folder.join("first.lowir"))?
    );
    let missing = Command::new(env!("CARGO_BIN_EXE_rustgm"))
        .arg("--emit-lowir")
        .arg(build.fixture("client"))
        .arg("--extern")
        .arg(format!("two={}", build.second.display()))
        .output()?;
    assert!(!missing.status.success());
    assert!(String::from_utf8_lossy(&missing.stderr).contains("missing crate dependency"));
    let malformed = build.folder.join("truncated.rgm");
    std::fs::write(&malformed, &first_bytes[..7])?;
    let rejected = Command::new(env!("CARGO_BIN_EXE_rustgm"))
        .arg("--emit-lowir")
        .arg(build.fixture("client"))
        .arg("--extern")
        .arg(format!("one={}", malformed.display()))
        .output()?;
    assert!(!rejected.status.success());
    assert!(String::from_utf8_lossy(&rejected.stderr).contains("truncated"));
    Ok(())
}

#[test]
#[ignore = "requires configured CPPGM native backend and pinned Rust toolchain"]
fn scalar_crates_link_with_duplicate_private_names_and_imported_rust_main()
-> Result<(), Box<dyn std::error::Error>> {
    let build = artifacts::build("native")?;
    build.qualify()?;
    for name in ["client", "imported-main"] {
        build.execute(name)?;
    }
    Ok(())
}

#[test]
fn proof_failures_publish_no_artifact_and_crate_name_alone_keeps_legacy_output()
-> Result<(), Box<dyn std::error::Error>> {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?;
    let folder = root.join(format!(
        "obj/parallel/pa29-crate-linking/proof-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&folder)?;
    let path = folder.join("invalid.rs");
    let metadata = folder.join("invalid.rgm");
    std::fs::write(
        &path,
        "fn bad(){let n:u8;let _=n+1;}pub fn f(n:u32)->u32{n}",
    )?;
    let invalid = Command::new(env!("CARGO_BIN_EXE_rustgm"))
        .args([
            "--emit-lowir",
            "--crate-type=lib",
            "--crate-name",
            "invalid",
            "--crate-disambiguator",
            "proof",
            "--metadata-output",
        ])
        .arg(&metadata)
        .arg(&path)
        .output()?;
    assert!(!invalid.status.success());
    assert!(!metadata.exists());
    assert!(String::from_utf8_lossy(&invalid.stderr).contains("uninitialized"));
    std::fs::write(&path, "fn main(){}")?;
    let ordinary = artifacts::checked(
        "ordinary",
        &folder,
        Command::new(env!("CARGO_BIN_EXE_rustgm"))
            .arg("--emit-lowir")
            .arg(&path),
    )?;
    let named = artifacts::checked(
        "named",
        &folder,
        Command::new(env!("CARGO_BIN_EXE_rustgm"))
            .args(["--emit-lowir", "--crate-name", "different"])
            .arg(path),
    )?;
    assert_eq!(ordinary.stdout, named.stdout);
    Ok(())
}

#[test]
fn entry_free_escaping_closure_reports_lifetime_failure_without_publishing_metadata()
-> Result<(), Box<dyn std::error::Error>> {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("missing root")?;
    let folder = root.join(format!(
        "obj/parallel/pa24-library-proofs/escape-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&folder)?;
    let metadata = folder.join("escaping.rgm");
    let fixture = root.join("pa24/tests/spec/output-interfaces/escape-outer.rs");
    for publish in [false, true] {
        let mut command = Command::new(env!("CARGO_BIN_EXE_rustgm"));
        command.args([
            "--emit-lowir",
            "--crate-type=lib",
            "--crate-name",
            "escaping",
            "--crate-disambiguator",
            "closure",
        ]);
        if publish {
            command.arg("--metadata-output").arg(&metadata);
        }
        let output = command.arg(&fixture).output()?;
        assert_eq!(output.status.code(), Some(1));
        assert!(output.stdout.is_empty());
        assert!(!metadata.exists());
        assert!(
            String::from_utf8_lossy(&output.stderr).contains("required lifetime does not outlive"),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    Ok(())
}
