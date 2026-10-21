//! Scalar union storage is a byte image, independent of selected field identity.
#[path = "harness/native_references.rs"]
mod native;
use std::{path::PathBuf, process::Command};
const CASES: &[(&str, i32, i32, i32)] = &[
    ("scalar-union/word-byte", 0, 0, 0),
    ("scalar-union/bool-zero", 0, 0, 0),
    ("scalar-union/bool-one", 0, 0, 0),
    ("scalar-union/invalid-alternative", 0, 0, 0),
    ("scalar-union/char-valid", 0, 0, 0),
    ("scalar-union/char-surrogate", 1, 1, 1),
    ("scalar-union/char-above-unicode", 1, 1, 1),
    ("scalar-union/float-from-bits", 0, 0, 0),
    ("scalar-union/negative-zero-bits", 0, 0, 0),
    ("scalar-union/wide-low-byte", 0, 0, 0),
    ("scalar-union/partial-preserve", 0, 0, 0),
    ("scalar-union/partial-uninitialized", 1, 1, 1),
    ("scalar-union/copy-snapshot", 0, 0, 0),
    ("scalar-union/const-function", 0, 0, 0),
    ("scalar-union/nested-product", 0, 0, 0),
    ("scalar-union/static-partial", 0, 0, 0),
    ("scalar-union/pointer-family", 0, 1, 1),
    ("scalar-union/array-family", 0, 1, 1),
    ("scalar-union/field-address", 1, 1, 1),
    ("scalar-union/ordinary-scalar", 0, 0, 0),
    ("unions/invalid-bool-const", 1, 1, 1),
    ("unions/unused-invalid-bool-const", 1, 1, 1),
    ("unions/uninitialized-byte-const", 1, 1, 1),
    ("unions/unused-uninitialized-byte-const", 1, 1, 1),
    ("unions/valid-bool-const-unsupported", 0, 0, 0),
    ("unions/uninit-field-write", 1, 0, 1),
    ("scalar-union/static-field-address", 0, 0, 0),
    ("scalar-union/reference-field", 0, 0, 0),
    ("scalar-union/reference-function", 0, 0, 0),
    ("scalar-union/atomic-context", 0, 0, 0),
    ("scalar-union/signed-bits", 0, 0, 0),
    ("scalar-union/mutable-reference", 0, 0, 0),
    ("scalar-union/image-size-limit", 0, 1, 1),
    ("scalar-union/generic-scalar", 0, 0, 0),
    ("scalar-union/zero-image", 0, 0, 0),
];
fn root() -> Result<PathBuf, Box<dyn std::error::Error>> {
    Ok(PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?
        .to_owned())
}
#[test]
fn scalar_union_objects_and_execution_contracts() -> Result<(), Box<dyn std::error::Error>> {
    let root = root()?;
    let folder = root.join(format!(
        "obj/parallel/pa27/union-images-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&folder)?;
    for &(name, rust, types, execution) in CASES {
        let path = root.join(format!("pa27/tests/spec/{name}.rs"));
        let stem = name.replace('/', "-");
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
            .arg(folder.join(format!("{stem}.o")))
            .arg(&path)
            .output()?;
        std::fs::write(
            folder.join(format!("{stem}.rustc.stderr")),
            &independent.stderr,
        )?;
        assert_eq!(independent.status.code(), Some(rust), "{name}");
        for (mode, status) in [
            ("--emit-types", types),
            ("--emit-lowir", execution),
            ("--emit-constants", execution),
        ] {
            let result = Command::new(env!("CARGO_BIN_EXE_rustgm"))
                .arg(mode)
                .arg(&path)
                .output()?;
            std::fs::write(folder.join(format!("{stem}.{mode}.stdout")), &result.stdout)?;
            std::fs::write(folder.join(format!("{stem}.{mode}.stderr")), &result.stderr)?;
            assert_eq!(
                result.status.code(),
                Some(status),
                "{name}/{mode}: {}",
                String::from_utf8_lossy(&result.stderr)
            );
            assert_eq!(result.stderr.is_empty(), status == 0, "{name}/{mode}");
        }
    }
    Ok(())
}
#[test]
fn image_views_preserve_unknown_bytes_and_validity() -> Result<(), Box<dyn std::error::Error>> {
    let root = root()?;
    for (name, reason) in [
        (
            "unions/unused-invalid-bool-const",
            "invalid bool representation",
        ),
        (
            "unions/unused-uninitialized-byte-const",
            "uninitialized bytes",
        ),
        ("scalar-union/char-surrogate", "invalid char representation"),
        (
            "scalar-union/char-above-unicode",
            "invalid char representation",
        ),
        (
            "scalar-union/pointer-family",
            "all-fields primitive scalar family",
        ),
    ] {
        let result = Command::new(env!("CARGO_BIN_EXE_rustgm"))
            .arg("--emit-constants")
            .arg(root.join(format!("pa27/tests/spec/{name}.rs")))
            .output()?;
        assert_eq!(result.status.code(), Some(1), "{name}");
        assert!(
            String::from_utf8_lossy(&result.stderr).contains(reason),
            "{name}: {}",
            String::from_utf8_lossy(&result.stderr)
        );
    }
    let path = root.join("pa27/tests/spec/scalar-union/static-partial.rs");
    let run = || {
        Command::new(env!("CARGO_BIN_EXE_rustgm"))
            .arg("--emit-constants")
            .arg(&path)
            .output()
    };
    let first = run()?;
    let second = run()?;
    assert!(first.status.success());
    assert_eq!(first.stdout, second.stdout);
    let view = String::from_utf8(first.stdout)?;
    assert!(
        view.contains("image size=4 bytes=25?????? initialized=0000000000000001"),
        "{view}"
    );
    Ok(())
}
#[test]
#[ignore = "requires the qualified external CPPGM LowIR backend"]
fn scalar_images_execute() -> Result<(), Box<dyn std::error::Error>> {
    for name in ["numeric", "overlap", "storage", "references"] {
        let text = std::fs::read_to_string(
            root()?.join(format!("pa27/tests/spec/scalar-union-native/{name}.rs")),
        )?;
        native::check_cases("parallel/pa27/union-images-native", &[(name, &text, false)])?;
    }
    Ok(())
}
