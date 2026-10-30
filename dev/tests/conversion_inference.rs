//! Independent `From` inputs exercise actual source roles through the CLI.
use std::{
    path::{Path, PathBuf},
    process::{Command, Output},
};
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
const CASES: &[(&str, Option<u8>)] = &[
    ("actual-role", Some(32)),
    ("ordinary-lookalike", Some(64)),
    ("explicit-f64", Some(64)),
    ("explicit-f32", Some(32)),
    ("other-subject", Some(64)),
    ("renamed-role", Some(32)),
    ("subject-alias", Some(32)),
    ("generic-wrapper", Some(32)),
    ("shared-root", Some(32)),
    ("explicit-root", Some(64)),
    ("unused-role", Some(64)),
    ("empty-role", Some(32)),
    ("wrong-arity-zero", None),
    ("wrong-arity-two", None),
    ("wrong-owner", None),
    ("duplicate-role", None),
    ("original-trait", Some(32)),
    ("late-subject", Some(32)),
    ("shared-watch-forward", Some(32)),
    ("shared-watch-reverse", Some(32)),
    ("completed-blanket", Some(64)),
    ("diagnostic-only", Some(64)),
    ("different-diagnostic-owner", Some(64)),
];
fn root() -> TestResult<PathBuf> {
    Ok(Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("missing repository")?
        .into())
}
fn compile(name: &str) -> TestResult<Output> {
    Ok(Command::new(env!("CARGO_BIN_EXE_rustgm"))
        .args([
            "--emit-lowir",
            "--edition=2024",
            "--cap-lints=allow",
            "--crate-type=lib",
            "--crate-name",
            "minicore",
            "--crate-disambiguator",
            "from_fallback",
            "--source-core",
            "minicore@from_fallback",
        ])
        .arg(
            root()?
                .join("pa31/tests/spec/source-from")
                .join(format!("{name}.rs")),
        )
        .output()?)
}
#[test]
fn real_language_and_diagnostic_identities_keep_declared_outcomes() -> TestResult {
    for &(name, native) in CASES {
        let output = compile(name)?;
        assert_eq!(
            output.status.code(),
            Some(i32::from(native.is_none())),
            "{name}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        if native.is_some() {
            assert!(
                !output.stdout.is_empty() && output.stderr.is_empty(),
                "{name}"
            );
        } else {
            assert!(
                output.stdout.is_empty() && output.stderr.starts_with(b"ERROR: "),
                "{name}"
            );
        }
    }
    Ok(())
}

fn run(folder: &Path, name: &str, command: &mut Command) -> TestResult<Output> {
    let output = command.output()?;
    std::fs::write(folder.join(format!("{name}.stdout")), &output.stdout)?;
    std::fs::write(folder.join(format!("{name}.stderr")), &output.stderr)?;
    assert!(
        output.status.success() && output.stderr.is_empty(),
        "{name}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(output)
}

#[test]
#[ignore = "requires the pinned CPPGM object backend and host C linker"]
fn exact_source_candidates_match_independent_native_values() -> TestResult {
    let root = root()?;
    let folder = root
        .join("obj/parallel/source-from-native")
        .join(std::process::id().to_string());
    std::fs::create_dir_all(&folder)?;
    for &(name, expected) in CASES {
        let Some(expected) = expected else { continue };
        let output = compile(name)?;
        assert!(
            output.status.success() && output.stderr.is_empty(),
            "{name}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        let lowir = folder.join(format!("{name}.lowir"));
        let object = folder.join(format!("{name}.o"));
        let executable = folder.join(format!("{name}.native"));
        std::fs::write(&lowir, output.stdout)?;
        run(
            &folder,
            &format!("{name}-backend"),
            Command::new("python3")
                .arg(root.join("scripts/backend.py"))
                .arg("--emit-object")
                .arg(&lowir)
                .arg("-o")
                .arg(&object),
        )?;
        run(
            &folder,
            &format!("{name}-link"),
            Command::new("cc")
                .arg(root.join("pa31/tests/spec/source-from/observer.c"))
                .arg(&object)
                .arg("-lm")
                .arg("-o")
                .arg(&executable),
        )?;
        let actual = run(
            &folder,
            &format!("{name}-execute"),
            &mut Command::new(&executable),
        )?;
        assert_eq!(actual.stdout, format!("{expected}\n").as_bytes(), "{name}");
        std::fs::remove_file(&object)?;
        std::fs::remove_file(&executable)?;
    }
    Ok(())
}
