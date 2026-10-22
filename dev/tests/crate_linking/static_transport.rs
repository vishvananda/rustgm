//! Named storage retains identity, safety and contents across actual object boundaries.
fn qualify(assignment: u8, native: bool) -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
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
        .arg(root.join(format!(
            "pa{assignment}/tests/storage-metadata-outcomes.json"
        )))
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
