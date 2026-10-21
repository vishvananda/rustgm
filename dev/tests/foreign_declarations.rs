//! Foreign ownership survives expansion and produces genuine runtime declarations.
#[path = "harness/foreign_frontend.rs"]
mod frontend;
#[cfg(feature = "object-interop-tests")]
#[path = "harness/host_objects.rs"]
mod objects;
use frontend::checked;
use std::{
    path::{Path, PathBuf},
    process::Command,
};
const CASES: &[(&str, &str, bool)] = &[
    (
        "generic-caller",
        "pa30/tests/spec/imports/generic-caller.rs",
        true,
    ),
    (
        "constant-pointer",
        "pa30/tests/spec/imports/constant-pointer.rs",
        true,
    ),
    ("scalars", "pa30/tests/spec/imports/scalars.rs", true),
    ("callback", "pa30/tests/spec/imports/callback.rs", true),
    ("pointer", "pa30/tests/spec/imports/pointer.rs", true),
    ("default", "pa30/tests/spec/imports/default.rs", true),
    ("safe", "pa30/tests/spec/imports/safe.rs", true),
    (
        "unsafe-pointer",
        "pa30/tests/spec/imports/unsafe-pointer.rs",
        true,
    ),
    ("rust-abi", "pa30/tests/spec/imports/rust-abi.rs", true),
    (
        "expanded-item",
        "pa30/tests/spec/imports/expanded-item.rs",
        true,
    ),
    (
        "expanded-block",
        "pa30/tests/spec/imports/expanded-block.rs",
        true,
    ),
    (
        "default-unsafe",
        "pa30/tests/spec/imports/default-unsafe.rs",
        false,
    ),
    (
        "rust-unsafe",
        "pa30/tests/spec/imports/rust-unsafe.rs",
        false,
    ),
    (
        "safety-narrowing",
        "pa30/tests/spec/imports/safety-narrowing.rs",
        false,
    ),
    (
        "abi-identity",
        "pa30/tests/spec/imports/abi-identity.rs",
        false,
    ),
    (
        "constant-call",
        "pa30/tests/spec/imports/constant-call.rs",
        false,
    ),
    (
        "type-parameter",
        "pa30/tests/spec/imports/type-parameter.rs",
        false,
    ),
    (
        "const-function",
        "pa30/tests/spec/imports/const-function.rs",
        false,
    ),
    (
        "missing-unsafe",
        "pa30/tests/spec/imports/missing-unsafe.rs",
        false,
    ),
    (
        "ordinary-safe",
        "pa30/tests/spec/imports/ordinary-safe.rs",
        false,
    ),
    (
        "foreign-body",
        "pa30/tests/spec/imports/foreign-body.rs",
        false,
    ),
];
fn fixture(path: &str) -> Result<String, std::io::Error> {
    std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join(path))
}
fn directory() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?;
    let path = root.join(format!(
        "obj/parallel/foreign-declarations/cases/{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&path)?;
    Ok(path)
}
#[test]
fn foreign_function_outcomes_are_independently_qualified() -> Result<(), Box<dyn std::error::Error>>
{
    let folder = directory()?;
    for &(name, path, accepted) in CASES {
        let source = fixture(path)?;
        let input = folder.join(format!("{name}.rs"));
        let object = folder.join(format!("{name}.o"));
        std::fs::write(&input, &source)?;
        let output = Command::new("rustc")
            .args([
                "+1.98.0",
                "--edition=2024",
                "--crate-type=lib",
                "--crate-name=foreign_case",
                "--cap-lints=allow",
                "--emit=obj",
                "-o",
            ])
            .arg(&object)
            .arg(input)
            .output()?;
        std::fs::write(folder.join(format!("{name}.stderr")), &output.stderr)?;
        assert_eq!(
            output.status.code(),
            Some(i32::from(!accepted)),
            "rustc {name}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        let result = checked(&source);
        assert_eq!(
            result.is_ok(),
            accepted,
            "{name}: {:?}",
            result.as_ref().err()
        );
    }
    Ok(())
}
#[test]
fn demanded_imports_have_declarations_without_bodies() -> Result<(), Box<dyn std::error::Error>> {
    for &(_, path, accepted) in CASES.iter().skip(5).take(6) {
        let source = fixture(path)?;
        assert!(accepted);
        let program = checked(&source)?;
        let imported: Vec<_> = program
            .symbols
            .iter()
            .filter(|s| s.name == "imported")
            .collect();
        assert_eq!(imported.len(), 1);
        assert!(imported[0].body.is_none());
        assert!(program.symbols.iter().any(|s| s.body.is_some()));
    }
    Ok(())
}
#[test]
#[cfg(feature = "object-interop-tests")]
#[ignore = "requires the optional pinned CPPGM object profile and host C compiler"]
fn imports_and_callbacks_link_with_independent_c() -> Result<(), Box<dyn std::error::Error>> {
    let cases: Vec<_> = CASES
        .iter()
        .take(5)
        .map(|&(name, path, _)| (name, path))
        .collect();
    objects::check_cases(
        "foreign-declarations",
        "pa30/tests/spec/imports/provider.c",
        &cases,
    )
}
