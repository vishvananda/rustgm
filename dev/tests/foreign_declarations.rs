//! Foreign ownership survives expansion and produces genuine runtime declarations.
use rustgm::{
    expand::config::Config,
    resolve::{self, load::InputFile},
    sema,
    support::source::SourceFile,
};
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
struct NoFiles;
impl resolve::load::Sources for NoFiles {
    fn read(&mut self, _: &Path) -> Result<Option<InputFile>, String> {
        Ok(None)
    }
}
fn checked(source: &str) -> Result<rustgm::lowir::program::Program, String> {
    let input = InputFile {
        identity: "lib.rs".into(),
        path: "lib.rs".into(),
        source: SourceFile::new(source.as_bytes().to_vec())?,
    };
    let expanded = resolve::expansion::analyze(input, &mut NoFiles, &Config::default())
        .map_err(|e| e.to_string())?;
    let mut report = resolve::analyze_expanded(expanded).map_err(|e| e.to_string())?;
    let rustgm::pipeline::Analysis {
        mut facts,
        prepared,
    } = rustgm::pipeline::analyze_source(&report).map_err(|e| e.to_string())?;
    let entry = sema::entry::select(&mut report, &facts).map_err(|e| e.to_string())?;
    rustgm::lower::instances::build_prepared(&report, &mut facts, entry, prepared)
        .map_err(|e| e.to_string())
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
#[cfg(feature = "object-interop-tests")]
fn successful(
    name: &str,
    output: std::process::Output,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    if !output.status.success() || !output.stderr.is_empty() {
        return Err(format!(
            "{name}: {}\n{}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }
    Ok(output.stdout)
}
#[test]
#[cfg(feature = "object-interop-tests")]
#[ignore = "requires the optional pinned CPPGM object profile and host C compiler"]
fn imports_and_callbacks_link_with_independent_c() -> Result<(), Box<dyn std::error::Error>> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?;
    let folder = directory()?;
    let provider = folder.join("provider.o");
    successful(
        "C provider",
        Command::new("cc")
            .args(["-std=c11", "-Wall", "-Wextra", "-Werror", "-c"])
            .arg(root.join("pa30/tests/spec/imports/provider.c"))
            .arg("-o")
            .arg(&provider)
            .output()?,
    )?;
    for &(name, path, accepted) in CASES.iter().take(5) {
        let source = fixture(path)?;
        assert!(accepted);
        let input = folder.join(format!("{name}.native.rs"));
        std::fs::write(&input, &source)?;
        let independent = folder.join(format!("{name}.rustc"));
        successful(
            name,
            Command::new("rustc")
                .args([
                    "+1.98.0",
                    "--edition=2024",
                    "--crate-name=foreign_native",
                    "--cap-lints=allow",
                    "-C",
                    "overflow-checks=yes",
                    "-C",
                    "panic=abort",
                ])
                .arg("-C")
                .arg(format!("link-arg={}", provider.display()))
                .arg(&input)
                .arg("-o")
                .arg(&independent)
                .output()?,
        )?;
        let expected = successful(name, Command::new(&independent).output()?)?;
        let lowir = successful(
            name,
            Command::new(env!("CARGO_BIN_EXE_rustgm"))
                .arg("--emit-lowir")
                .args(["--crate-name", "foreign_native"])
                .arg(&input)
                .output()?,
        )?;
        let path = folder.join(format!("{name}.lowir"));
        std::fs::write(&path, &lowir)?;
        let object = folder.join(format!("{name}.rustgm.o"));
        successful(
            name,
            Command::new("python3")
                .arg(root.join("scripts/backend.py"))
                .arg("--emit-object")
                .arg(&path)
                .arg("-o")
                .arg(&object)
                .output()?,
        )?;
        let executable = folder.join(format!("{name}.rustgm"));
        successful(
            name,
            Command::new("cc")
                .arg(&object)
                .arg(&provider)
                .arg("-o")
                .arg(&executable)
                .output()?,
        )?;
        let actual = successful(name, Command::new(&executable).output()?)?;
        assert_eq!(actual, expected, "{name}");
        assert!(actual.is_empty());
    }
    Ok(())
}
