//! Full code-generation qualification with the pinned independent compiler.
use std::{path::Path, process::Command};
pub fn qualify(
    name: &str,
    source: &str,
    accepted: bool,
    crate_type: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let folder =
        Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("../obj/{name}/{}", std::process::id()));
    std::fs::create_dir_all(&folder)?;
    let input = folder.join("case.rs");
    let object = folder.join("case.o");
    std::fs::write(&input, source)?;
    let output = Command::new("rustc")
        .args([
            "+1.98.0",
            "--edition=2024",
            "--crate-name=independent",
            "--emit=obj",
            "--crate-type",
            crate_type,
            "-o",
        ])
        .arg(&object)
        .arg(&input)
        .output()?;
    std::fs::write(folder.join("stderr"), &output.stderr)?;
    assert_eq!(
        output.status.code(),
        Some(i32::from(!accepted)),
        "{name}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    if accepted {
        std::fs::remove_file(object)?;
    }
    Ok(())
}
