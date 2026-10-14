//! Independent language checks with the repository-pinned Rust toolchain.
use std::{path::Path, process::Command};
pub fn qualify(name: &str, source: &str, accepted: bool) -> Result<(), Box<dyn std::error::Error>> {
    let folder = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../obj/drop-identity-{}", std::process::id()));
    std::fs::create_dir_all(&folder)?;
    let input = folder.join(format!("{name}.rs"));
    std::fs::write(&input, source)?;
    let output = Command::new("rustc")
        .args([
            "--edition=2024",
            "--crate-type=lib",
            "--crate-name=drop_identity",
            "--emit=metadata",
            "-o",
        ])
        .arg(folder.join(format!("{name}.rmeta")))
        .arg(input)
        .output()?;
    assert_eq!(
        output.status.code(),
        Some(i32::from(!accepted)),
        "{name}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(())
}
