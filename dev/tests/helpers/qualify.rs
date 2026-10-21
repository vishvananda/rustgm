//! Independent language checks with the repository-pinned Rust toolchain.
use std::{path::Path, process::Command};
pub fn qualify(name: &str, source: &str, accepted: bool) -> Result<(), Box<dyn std::error::Error>> {
    let folder = Path::new(env!("CARGO_MANIFEST_DIR")).join(format!(
        "../obj/language-qualification/{}",
        std::process::id()
    ));
    qualify_at(&folder, name, source, accepted)
}

pub fn qualify_at(
    folder: &Path,
    name: &str,
    source: &str,
    accepted: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    std::fs::create_dir_all(folder)?;
    let input = folder.join(format!("{name}.rs"));
    let metadata = folder.join(format!("{name}.rmeta"));
    std::fs::write(&input, source)?;
    let output = Command::new("rustc")
        .args([
            "--edition=2024",
            "--crate-type=lib",
            "--crate-name=drop_identity",
            "--emit=metadata",
            "-o",
        ])
        .arg(&metadata)
        .arg(input)
        .output()?;
    std::fs::write(folder.join(format!("{name}.stderr")), &output.stderr)?;
    assert_eq!(
        output.status.code(),
        Some(i32::from(!accepted)),
        "{name}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    if accepted {
        std::fs::remove_file(metadata)?;
    }
    Ok(())
}
