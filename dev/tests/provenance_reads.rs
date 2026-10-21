//! Qualified raw reads preserve storage identity, physical alignment and lifetime.
#[path = "harness/native_references.rs"]
mod native;
use std::{path::PathBuf, process::Command};
const CASES: &[(&str, bool, bool)] = &[
    ("packed-base-copy", true, true),
    ("packed-base-field-pointer", false, false),
    ("packed-nested-base-copy", true, true),
    ("packed-nested-field-pointer", false, false),
    ("packed-pointer-chain", true, true),
    ("packed-pointer-chain-unaligned", false, false),
    ("packed-nested-offset", false, false),
    ("direct-static", true, true),
    ("direct-packed-static", false, false),
    ("direct-cancellation", true, true),
    ("array-bounds", false, false),
    ("static-mutable-indirect", false, false),
    ("local", true, true),
    ("mutable-pointer", true, true),
    ("field", true, true),
    ("array", true, true),
    ("function", true, true),
    ("static", true, true),
    ("static-projected", true, true),
    ("packed-nested-aligned", true, true),
    ("raw-base-field", true, true),
    ("raw-base-array", true, true),
    ("static-indirect", true, true),
    ("address-cancellation", true, true),
    ("packed-unaligned", false, false),
    ("null", false, false),
    ("expired", false, false),
    ("uninitialized", false, false),
    ("cast-address-cancellation", false, false),
    ("mutable-static", false, false),
    ("static-cycle", false, false),
    ("nonnull-numeric", false, false),
    ("reinterpret-boundary", true, false),
    ("raw-write-boundary", true, true),
    ("raw-borrow-boundary", true, true),
];
fn root() -> Result<PathBuf, Box<dyn std::error::Error>> {
    Ok(PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?
        .to_owned())
}
fn rejection(name: &str) -> Result<&'static str, Box<dyn std::error::Error>> {
    Ok(match name {
        "packed-unaligned"
        | "direct-packed-static"
        | "packed-nested-offset"
        | "packed-base-field-pointer"
        | "packed-nested-field-pointer"
        | "packed-pointer-chain-unaligned" => "constant raw pointer read is unaligned",
        "null" | "nonnull-numeric" => "constant pointer has no allocation provenance",
        "expired" | "cast-address-cancellation" => "constant pointer refers to expired storage",
        "array-bounds" => "constant index is out of bounds",
        "uninitialized" => "borrowed place may be uninitialized or moved",
        "mutable-static" | "static-mutable-indirect" => {
            "constant execution cannot read mutable static storage"
        }
        "static-cycle" => "cycle in ",
        "reinterpret-boundary" => {
            "raw pointer type reinterpretation in constant execution is not implemented"
        }
        _ => return Err(format!("missing expected diagnostic for {name}").into()),
    })
}
#[test]
fn symbolic_raw_reads_match_independent_objects() -> Result<(), Box<dyn std::error::Error>> {
    let root = root()?;
    let evidence = root.join(format!(
        "obj/parallel/pa27/constant-raw-reads-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&evidence)?;
    for &(name, rust, frontend) in CASES {
        let source = root.join(format!("pa27/tests/spec/raw-constant-reads/{name}.rs"));
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
        assert_eq!(independent.status.code(), Some(i32::from(!rust)), "{name}");
        for mode in ["--emit-types", "--emit-lowir"] {
            let result = Command::new(env!("CARGO_BIN_EXE_rustgm"))
                .arg(mode)
                .arg(&source)
                .output()?;
            std::fs::write(
                evidence.join(format!("{name}.{mode}.stdout")),
                &result.stdout,
            )?;
            std::fs::write(
                evidence.join(format!("{name}.{mode}.stderr")),
                &result.stderr,
            )?;
            assert_eq!(
                result.status.code(),
                Some(i32::from(!frontend)),
                "{name}/{mode}: {}",
                String::from_utf8_lossy(&result.stderr)
            );
            assert_eq!(result.stderr.is_empty(), frontend, "{name}/{mode}");
            if !frontend {
                assert!(result.stdout.is_empty(), "{name}/{mode}");
                let reason = rejection(name)?;
                assert!(
                    String::from_utf8_lossy(&result.stderr).contains(reason),
                    "{name}/{mode}: {}",
                    String::from_utf8_lossy(&result.stderr)
                );
            }
        }
    }
    Ok(())
}
#[test]
#[ignore = "requires the qualified external CPPGM LowIR backend"]
fn typed_raw_reads_execute() -> Result<(), Box<dyn std::error::Error>> {
    for name in [
        "local-products",
        "function-static",
        "packed-physical",
        "packed-origin",
    ] {
        let source = std::fs::read_to_string(
            root()?.join(format!("pa27/tests/spec/raw-read-native/{name}.rs")),
        )?;
        native::check_cases("parallel/pa27/raw-read-native", &[(name, &source, false)])?;
    }
    Ok(())
}
