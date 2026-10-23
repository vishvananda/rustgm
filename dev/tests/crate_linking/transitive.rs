//! Complete metadata inventory grants visibility only through explicit aliases.
use std::{path::Path, process::Command};
type Result = std::result::Result<(), Box<dyn std::error::Error>>;

fn qualify(native: bool) -> Result {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("missing repository root")?;
    let folder = root.join(format!(
        "obj/parallel/pa29-transitive-metadata/maintainer-{native}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos()
    ));
    let mut command = Command::new("python3");
    command
        .current_dir(root)
        .arg(root.join("scripts/check_transitive_metadata.py"))
        .arg("--app")
        .arg(env!("CARGO_BIN_EXE_rustgm"))
        .arg("--evidence-root")
        .arg(&folder);
    if native {
        command.arg("--native");
    }
    super::artifacts::checked("harness", &folder, &mut command)?;
    Ok(())
}

#[test]
fn transitive_metadata_inventory_keeps_direct_alias_visibility() -> Result {
    qualify(false)
}

#[test]
#[ignore = "requires configured CPPGM object backend and pinned Rust toolchain"]
fn transitive_native_calls_link_without_exposing_dependency_names() -> Result {
    qualify(true)
}
