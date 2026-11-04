//! Actual source owner and feature authority survive the public CLI boundary.
#[test]
fn fundamental_headers_match_the_independent_outcomes() -> Result<(), Box<dyn std::error::Error>> {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("repository root")?;
    let cases = std::fs::read_to_string(root.join("pa31/tests/spec/fundamental-policy/cases.tsv"))?;
    for row in cases.lines() {
        let (path, expected) = row.split_once('\t').ok_or("fundamental case")?;
        let output = std::process::Command::new(env!("CARGO_BIN_EXE_rustgm"))
            .args([
                "--emit-constants",
                "--cap-lints=allow",
                "--source-core",
                "probe@fundamental_headers",
                "--crate-name",
                "probe",
                "--crate-disambiguator",
                "fundamental_headers",
            ])
            .arg(root.join(path))
            .output()?;
        assert_eq!(
            output.status.code(),
            Some(expected.parse::<i32>()?),
            "{path}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    assert_eq!(cases.lines().count(), 34);
    Ok(())
}
