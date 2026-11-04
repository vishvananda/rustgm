//! Field availability is checked on actual typed aggregates and independent uses.
#[test]
fn actual_field_cli_outcomes_match_the_pinned_language_contract()
-> Result<(), Box<dyn std::error::Error>> {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("repository root")?;
    let cases =
        std::fs::read_to_string(root.join("pa31/tests/spec/aggregate-field-policy/cases.tsv"))?;
    let mut count = 0;
    for line in cases.lines() {
        let (path, status) = line.split_once('\t').ok_or("field case")?;
        let output = std::process::Command::new(env!("CARGO_BIN_EXE_rustgm"))
            .args(["--emit-constants", "--cap-lints=allow"])
            .arg(root.join(path))
            .output()?;
        assert_eq!(
            output.status.code(),
            Some(status.parse::<i32>()?),
            "{path}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(
            !String::from_utf8_lossy(&output.stderr).contains("not implemented"),
            "{path}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        count += 1;
    }
    assert_eq!(count, 24);
    Ok(())
}

#[test]
#[ignore = "requires configured CPPGM backend and pinned independent Rust"]
fn annotated_fields_preserve_actual_construction_pattern_and_access_execution()
-> Result<(), Box<dyn std::error::Error>> {
    crate::observed_program::compare(
        "field-policy-runtime",
        "pa31/tests/spec/aggregate-field-policy",
        b"7\n9\n",
    )
}
