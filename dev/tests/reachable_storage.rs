//! Result reachability differs from evaluating a repeat operand and from type inspection.
use crate::native_references as native;
use std::{path::PathBuf, process::Command};

const CASES: &[(&str, bool, bool)] = &[
    ("zero-raw-repeat-local", true, true),
    ("zero-raw-repeat-function", true, true),
    ("zero-raw-repeat-nested", true, true),
    ("transient-returned-raw", true, true),
    ("transient-local-raw", true, true),
    ("integer-raw-address", true, true),
    ("packed-copy-compound", true, true),
    ("packed-value-pattern", true, true),
    ("packed-raw-address", true, true),
    ("dangling-raw-repeat", false, false),
    ("expired-raw-read", false, false),
    ("packed-reference-pattern", false, true),
    ("packed-match-reference", false, true),
];
fn root() -> Result<PathBuf, Box<dyn std::error::Error>> {
    Ok(PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("repository root")?
        .to_owned())
}
#[test]
fn logical_result_storage_matches_independent_objects() -> Result<(), Box<dyn std::error::Error>> {
    let root = root()?;
    let evidence = root.join(format!(
        "obj/parallel/pa27/reachable-storage-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&evidence)?;
    for &(name, accepted, inspection) in CASES {
        let source = root.join(format!("pa27/tests/spec/validity-liveness/{name}.rs"));
        let independent = Command::new("rustc")
            .args([
                "+1.98.0",
                "--edition=2024",
                "--crate-name=probe",
                "--emit=obj",
                "-C",
                "panic=abort",
                "-o",
            ])
            .arg(evidence.join(format!("{name}.o")))
            .arg(&source)
            .output()?;
        std::fs::write(
            evidence.join(format!("{name}.rustc.stderr")),
            &independent.stderr,
        )?;
        assert_eq!(
            independent.status.code(),
            Some(i32::from(!accepted)),
            "{name}"
        );
        for (mode, expected) in [("--emit-types", inspection), ("--emit-lowir", accepted)] {
            let output = Command::new(env!("CARGO_BIN_EXE_rustgm"))
                .arg(mode)
                .arg(&source)
                .output()?;
            std::fs::write(
                evidence.join(format!("{name}.{mode}.stderr")),
                &output.stderr,
            )?;
            assert_eq!(
                output.status.code(),
                Some(i32::from(!expected)),
                "{name}/{mode}: {}",
                String::from_utf8_lossy(&output.stderr)
            );
            assert_eq!(output.stderr.is_empty(), expected, "{name}/{mode}");
            if !expected {
                assert!(output.stdout.is_empty(), "{name}/{mode}");
                let reason = match name {
                    "dangling-raw-repeat" | "expired-raw-read" => {
                        "constant pointer refers to expired storage"
                    }
                    _ => "reference to an unaligned packed field",
                };
                assert!(
                    String::from_utf8_lossy(&output.stderr).contains(reason),
                    "{name}/{mode}"
                );
            }
        }
    }
    Ok(())
}
#[test]
#[ignore = "requires the qualified external CPPGM LowIR backend"]
fn empty_storage_and_packed_copies_execute() -> Result<(), Box<dyn std::error::Error>> {
    for name in [
        "empty-pointer-storage",
        "transient-pointer-values",
        "packed-copies",
    ] {
        let source = std::fs::read_to_string(
            root()?.join(format!("pa27/tests/spec/validity-native/{name}.rs")),
        )?;
        native::check_cases("parallel/pa27/reachable-native", &[(name, &source, false)])?;
    }
    Ok(())
}
