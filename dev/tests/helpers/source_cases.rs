//! Independently qualified outcomes pass through the complete command-line pipeline.
#[path = "qualify.rs"]
mod rustc;
pub use rustc::{qualify, qualify_at};
use std::{path::Path, process::Command};

pub fn check(suite: &str, cases: &[(&str, bool, &str)]) -> Result<(), Box<dyn std::error::Error>> {
    let base = if suite.starts_with("parallel/") {
        "../obj"
    } else {
        "../obj/language-qualification"
    };
    let folder = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("{base}/{suite}-{}", std::process::id()));
    std::fs::create_dir_all(&folder)?;
    for &(name, expected, source) in cases {
        if suite.starts_with("parallel/") {
            qualify_at(&folder.join("rustc"), name, source, expected)?;
        } else {
            qualify(&format!("{suite}-{name}"), source, expected)?;
        }
        let input = folder.join(format!("{name}.rs"));
        std::fs::write(&input, format!("{source}fn main(){{}}"))?;
        let result = Command::new(env!("CARGO_BIN_EXE_rustgm"))
            .arg("--emit-lowir")
            .arg(&input)
            .output()?;
        std::fs::write(input.with_extension("stderr"), &result.stderr)?;
        assert_eq!(
            result.status.code(),
            Some(i32::from(!expected)),
            "{name}: {}",
            String::from_utf8_lossy(&result.stderr)
        );
        assert_eq!(result.stderr.is_empty(), expected, "{name}");
    }
    Ok(())
}
