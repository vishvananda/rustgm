//! Named storage retains identity, safety and contents across actual object boundaries.
fn qualify(assignment: u8, native: bool) -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
    qualify_manifest(assignment, "storage-metadata-outcomes.json", native)
}
pub(super) fn qualify_manifest(
    assignment: u8,
    manifest: &str,
    native: bool,
) -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("missing repository")?;
    let folder = root.join(format!(
        "obj/parallel/pa30/storage-metadata/maintainer-{assignment}-{}-{native}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos(),
    ));
    let mut command = std::process::Command::new("python3");
    command
        .current_dir(root)
        .arg(root.join("scripts/check_storage_metadata.py"))
        .arg("--app")
        .arg(env!("CARGO_BIN_EXE_rustgm"))
        .arg("--manifest")
        .arg(root.join(format!("pa{assignment}/tests/{manifest}")))
        .arg("--evidence-root")
        .arg(&folder);
    if native {
        command.arg("--native");
    }
    let result = command.output()?;
    std::fs::write(folder.join("harness.stdout"), &result.stdout)?;
    std::fs::write(folder.join("harness.stderr"), &result.stderr)?;
    assert!(
        result.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&result.stdout),
        String::from_utf8_lossy(&result.stderr)
    );
    Ok(folder)
}

#[test]
fn stored_global_interfaces_are_reproducible_and_preserve_permissions()
-> Result<(), Box<dyn std::error::Error>> {
    qualify(30, false).map(|_| ())
}

#[test]
#[ignore = "requires configured CPPGM object backend, C compiler and pinned Rust toolchain"]
fn stored_globals_execute_across_crates_and_foreign_objects()
-> Result<(), Box<dyn std::error::Error>> {
    qualify(30, true).map(|_| ())
}

#[test]
fn hidden_and_cyclic_storage_retain_canonical_metadata() -> Result<(), Box<dyn std::error::Error>> {
    use rustgm::support::crates::{self, StaticRef, StoredKind};
    let folder = qualify(29, false)?;
    let mut type_count = None;
    for width in [8usize, 32, 128] {
        let bytes = std::fs::read(folder.join(format!("shared_roots_{width}.rgm")))?;
        let interface = crates::decode(&bytes)?;
        let storage = interface.storage().ok_or("missing shared-root storage")?;
        assert_eq!(storage.statics().len(), width + 1);
        assert_eq!(
            storage.statics().iter().filter(|s| s.public()).count(),
            width
        );
        let backing = storage
            .statics()
            .iter()
            .position(|s| !s.public())
            .ok_or("missing private backing")?;
        let backing = StaticRef(u32::try_from(backing)?);
        for value in storage.values() {
            if let StoredKind::Reference { target, fields } = &value.kind {
                assert_eq!(*target, backing);
                assert_eq!(fields.as_ref(), &[1]);
            }
        }
        assert_eq!(interface.records().len(), 1);
        let types = interface
            .nominals()
            .ok_or("missing graph")?
            .types()
            .nodes()
            .len();
        if let Some(previous) = type_count {
            assert_eq!(types, previous);
        }
        type_count = Some(types);
        assert!(storage.values().len() <= width + 3);
        assert!(bytes.len() <= 512 * width + 4096);
        eprintln!(
            "shared_roots={width} statics={} values={} types={types} bytes={}",
            storage.statics().len(),
            storage.values().len(),
            bytes.len()
        );
    }
    Ok(())
}

#[test]
#[ignore = "requires configured CPPGM object backend, C compiler and pinned Rust toolchain"]
fn hidden_and_cyclic_storage_execute_with_original_address_identity()
-> Result<(), Box<dyn std::error::Error>> {
    qualify(29, true).map(|_| ())
}

#[test]
fn stored_arrays_preserve_compact_recipes_and_typed_addresses()
-> Result<(), Box<dyn std::error::Error>> {
    qualify_manifest(29, "array-storage-outcomes.json", false).map(|_| ())
}
#[test]
#[ignore = "requires configured CPPGM object backend, C compiler and pinned Rust toolchain"]
fn stored_arrays_execute_across_actual_object_boundaries() -> Result<(), Box<dyn std::error::Error>>
{
    qualify_manifest(29, "array-storage-outcomes.json", true).map(|_| ())
}

#[test]
fn version_fourteen_shares_restricted_array_storage() -> super::fresh_packets::Result {
    use super::fresh_packets::Packet;
    use rustgm::support::crates;
    let packet = Packet::new(
        "v14-static-arrays",
        "pa29/tests/spec/static-sync",
        "frontend",
        "pairs",
    )?;
    let foundation = std::fs::read_to_string(packet.fixture("provider"))?;
    let (foundation, _) = foundation
        .split_once("// Authored callable-role transport controls.")
        .ok_or("static provider lost its role foundation")?;
    let provider = packet.folder.join("provider.rs");
    let mut source = foundation.to_owned();
    source.push_str(
        r#"
#[lang = "sync"]
pub unsafe auto trait Sync {}
pub mod tables {
    pub(super) static PAIRS: [(u64, u64); 2] = [(3, 5), (7, 11)];
}
pub fn first<T>() -> &'static [(u64, u64); 2] { &tables::PAIRS }
pub fn same<T>() -> &'static [(u64, u64); 2] { &tables::PAIRS }
"#,
    );
    std::fs::write(&provider, source)?;
    super::artifacts::checked(
        "provider",
        &packet.folder,
        packet
            .export("module-body-operations-v14", &provider, &packet.metadata)
            .arg("--cap-lints=allow"),
    )?;
    let bytes = std::fs::read(&packet.metadata)?;
    assert_eq!(crates::encode(&crates::decode(&bytes)?)?, bytes);
    for (name, body, expected) in [
        (
            "allowed",
            r"
pub fn first() -> usize { base::first::<u8>() as *const [(u64, u64); 2] as usize }
pub fn same() -> usize { base::same::<u32>() as *const [(u64, u64); 2] as usize }
",
            None,
        ),
        (
            "private",
            "fn read() -> &'static [(u64, u64); 2] { &base::tables::PAIRS }",
            Some("private"),
        ),
        (
            "wrong",
            "fn read() -> &'static [(u64, u64); 3] { base::first::<u8>() }",
            Some("incompatible types"),
        ),
    ] {
        let input = packet.folder.join(format!("{name}.rs"));
        std::fs::write(
            &input,
            format!("#![feature(no_core)]#![no_core]\nextern crate base;\n{body}\n"),
        )?;
        let mut consumer = packet.consumer(&input, name, &packet.metadata, &packet.owner());
        consumer.arg("--cap-lints=allow");
        if let Some(diagnostic) = expected {
            packet.rejected(name, &mut consumer, diagnostic)?;
        } else {
            super::artifacts::checked(name, &packet.folder, &mut consumer)?;
        }
    }
    let provider_data = std::fs::read_to_string(packet.metadata.with_extension("lowir"))?;
    let consumer_data = std::fs::read_to_string(packet.folder.join("allowed.lowir"))?;
    assert!(provider_data.contains("static$"), "{provider_data}");
    assert!(consumer_data.contains("static$"), "{consumer_data}");
    let legacy = packet.folder.join("legacy.rgcrate");
    packet.rejected(
        "legacy",
        packet
            .export("module-bodies-v13", &provider, &legacy)
            .arg("--cap-lints=allow"),
        "source declaration kind is outside the initial source profile",
    )?;
    assert!(!legacy.exists());
    Ok(())
}
