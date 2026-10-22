//! Frozen backend controls are separate from source-language qualification.
use std::{path::Path, process::Command};
#[test]
#[ignore = "requires the qualified CPPGM native backend"]
fn overlapping_byte_loop_backend_controls() -> Result<(), Box<dyn std::error::Error>> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?;
    let folder = root.join(format!(
        "obj/parallel/pa28/overlapping-memory/backend-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&folder)?;
    for name in ["left", "right", "identical", "disjoint", "zero", "zst-max"] {
        let input = root.join(format!(
            "pa28/tests/backend/overlapping-memory/{name}.lowir"
        ));
        let output = folder.join(name);
        let build = Command::new("python3")
            .arg(root.join("scripts/backend.py"))
            .arg(input)
            .arg("-o")
            .arg(&output)
            .output()?;
        std::fs::write(folder.join(format!("{name}.build.stderr")), &build.stderr)?;
        assert!(
            build.status.success() && build.stdout.is_empty() && build.stderr.is_empty(),
            "{name}: {}",
            String::from_utf8_lossy(&build.stderr)
        );
        let run = Command::new(&output).output()?;
        std::fs::write(folder.join(format!("{name}.run.stderr")), &run.stderr)?;
        assert_eq!(run.status.code(), Some(42), "{name}");
        assert!(run.stdout.is_empty() && run.stderr.is_empty(), "{name}");
    }
    Ok(())
}
