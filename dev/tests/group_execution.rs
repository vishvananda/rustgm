//! Actual implicit const members keep constant and ordinary trait consumers separate.
#[test]
fn const_views_preserve_all_independent_inherent_block_outcomes()
-> Result<(), Box<dyn std::error::Error>> {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("repository root")?;
    let cases =
        std::fs::read_to_string(root.join("pa31/tests/spec/const-inherent-policy/cases-v4.tsv"))?;
    let mut count = 0;
    for line in cases.lines() {
        let (path, expected) = line.split_once('\t').ok_or("source case fields")?;
        let output = std::process::Command::new(env!("CARGO_BIN_EXE_rustgm"))
            .args(["--emit-constants", "--cap-lints=allow"])
            .arg(root.join(path))
            .output()?;
        assert_eq!(
            output.status.code(),
            Some(expected.parse::<i32>()?),
            "{path}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        if expected == "1" {
            assert!(
                !String::from_utf8_lossy(&output.stderr).contains("not implemented"),
                "{path}: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
        count += 1;
    }
    assert_eq!(count, 51);
    Ok(())
}

#[test]
fn runtime_specializations_preserve_defining_and_reached_const_checks()
-> Result<(), Box<dyn std::error::Error>> {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("repository root")?;
    for (path, expected) in [
        (
            "pa25/tests/spec/const-inherent/conditional-runtime-bound.rs",
            0,
        ),
        ("pa31/tests/spec/const-inherent-policy/native.rs", 0),
        (
            "pa25/tests/spec/const-inherent/conditional-nonconst-impl.rs",
            1,
        ),
        ("pa25/tests/spec/const-inherent/unused-runtime-body.rs", 1),
        (
            "pa25/tests/spec/const-inherent/missing-inherited-bound.rs",
            1,
        ),
    ] {
        let output = std::process::Command::new(env!("CARGO_BIN_EXE_rustgm"))
            .args(["--emit-lowir", "--cap-lints=allow"])
            .arg(root.join(path))
            .output()?;
        assert_eq!(
            output.status.code(),
            Some(expected),
            "{path}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(
            !String::from_utf8_lossy(&output.stderr).contains("not implemented"),
            "{path}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    Ok(())
}

#[test]
#[ignore = "requires configured CPPGM backend and pinned independent Rust"]
fn inherited_const_members_pay_const_and_ordinary_bounds_at_their_real_calls()
-> Result<(), Box<dyn std::error::Error>> {
    crate::observed_program::compare(
        "const-inherent-runtime",
        "pa31/tests/spec/const-inherent-policy",
        b"7\n9\n",
    )
}
