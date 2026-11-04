//! CLI applicability comes from actual source policy and the checked Min rules.
#[test]
fn source_trait_contracts_match_the_pinned_outcomes() -> Result<(), Box<dyn std::error::Error>> {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("repository root")?;
    let cases = std::fs::read_to_string(root.join("pa31/tests/spec/applicable-traits/cases.tsv"))?;
    for line in cases.lines() {
        let (path, expected) = line.split_once('\t').ok_or("applicability case")?;
        let output = std::process::Command::new(env!("CARGO_BIN_EXE_rustgm"))
            .args([
                "--emit-constants",
                "--cap-lints=allow",
                "--source-core",
                "probe@applicable_headers",
                "--crate-name",
                "probe",
                "--crate-disambiguator",
                "applicable_headers",
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
    assert_eq!(cases.lines().count(), 43);
    Ok(())
}
