//! CLI declarations use actual source-provider authority, never bootstrap path spellings.
#[test]
fn exact_cli_header_outcomes_match_the_independently_declared_contract()
-> Result<(), Box<dyn std::error::Error>> {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("repository root")?;
    let mut cases =
        std::fs::read_to_string(root.join("pa28/tests/spec/intrinsic-declarations/cases.tsv"))?;
    cases.push_str(&std::fs::read_to_string(
        root.join("pa28/tests/spec/intrinsic-declarations/named-headers.tsv"),
    )?);
    let mut count = 0;
    for line in cases.lines() {
        let (path, status) = line.split_once('\t').ok_or("intrinsic case")?;
        let output = std::process::Command::new(env!("CARGO_BIN_EXE_rustgm"))
            .args([
                "--emit-constants",
                "--cap-lints=allow",
                "--source-core",
                "probe@intrinsic_headers",
                "--crate-name",
                "probe",
                "--crate-disambiguator",
                "intrinsic_headers",
            ])
            .arg(root.join(path))
            .output()?;
        assert_eq!(
            output.status.code(),
            Some(status.parse::<i32>()?),
            "{path}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        count += 1;
    }
    assert_eq!(count, 659);
    Ok(())
}
