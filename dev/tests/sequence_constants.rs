//! Array-backed constant slice views retain bounds, aliasing and compressed values.
#[path = "harness/checked_sources.rs"]
mod checked_sources;
use std::{path::PathBuf, process::Command};

const ACCEPT: &[&str] = &[
    "slice-index",
    "slice-static",
    "slice-mutate",
    "slice-suffix",
    "slice-empty",
    "slice-repeat",
];

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..")
}
fn source(name: &str) -> PathBuf {
    root().join(format!("pa25/tests/spec/sequences/{name}.rs"))
}
fn evidence(lane: &str) -> Result<PathBuf, std::io::Error> {
    let path = root().join(format!(
        "obj/parallel/pa25/sequence-constants/{lane}/{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&path)?;
    Ok(path)
}

#[test]
fn array_backed_slices_match_independent_language_outcomes()
-> Result<(), Box<dyn std::error::Error>> {
    let evidence = evidence("source")?;
    for &name in ACCEPT {
        checked_sources::check(&source(name), &evidence, name, (0, 0, ""))?;
    }
    for (name, reason) in [
        ("slice-oob", "constant index is out of bounds"),
        ("slice-empty-oob", "constant index is out of bounds"),
        (
            "slice-local-escape",
            "required lifetime does not outlive its target",
        ),
    ] {
        checked_sources::check(&source(name), &evidence, name, (1, 1, reason))?;
    }
    Ok(())
}

#[test]
fn pending_sequence_views_and_const_text_eligibility_stay_explicit()
-> Result<(), Box<dyn std::error::Error>> {
    let evidence = evidence("boundaries")?;
    for (name, rust, reason) in [
        (
            "slice-rest",
            0,
            "constant slice projection is not implemented",
        ),
        (
            "array-rest",
            0,
            "constant slice projection is not implemented",
        ),
        (
            "slice-bytes",
            0,
            "constant projection base is not a pointer",
        ),
        (
            "text-match",
            1,
            "constant text pattern execution is not implemented",
        ),
        (
            "text-empty",
            1,
            "constant text pattern execution is not implemented",
        ),
    ] {
        checked_sources::check(&source(name), &evidence, name, (rust, 1, reason))?;
    }
    Ok(())
}

#[test]
#[ignore = "requires the pinned CPPGM backend; run make test-maintainer-native"]
fn exact_sequence_fixture_bytes_execute_with_both_compilers()
-> Result<(), Box<dyn std::error::Error>> {
    let evidence = evidence("native")?;
    for &name in ACCEPT {
        let input = source(name);
        let lowir = evidence.join(format!("{name}.lowir"));
        let independent = evidence.join(format!("{name}.rustc"));
        let native = evidence.join(format!("{name}.native"));
        let mut rust = Command::new("rustc");
        rust.args([
            "+1.98.0",
            "--edition=2024",
            "--crate-name=sequence",
            "--cap-lints=allow",
            "-C",
            "panic=abort",
        ])
        .arg(&input)
        .arg("-o")
        .arg(&independent);
        let mut frontend = Command::new(env!("CARGO_BIN_EXE_rustgm"));
        frontend
            .arg("--emit-lowir")
            .arg(&input)
            .arg("-o")
            .arg(&lowir);
        let mut backend = Command::new("python3");
        backend
            .arg(root().join("scripts/backend.py"))
            .arg(&lowir)
            .arg("-o")
            .arg(&native);
        for (phase, mut command) in [
            ("rust-build", rust),
            ("rust-run", Command::new(&independent)),
            ("frontend", frontend),
            ("backend", backend),
            ("native-run", Command::new(&native)),
        ] {
            let result = command.output()?;
            std::fs::write(
                evidence.join(format!("{name}.{phase}.stderr")),
                &result.stderr,
            )?;
            assert!(
                result.status.success(),
                "{name}/{phase}: {}",
                String::from_utf8_lossy(&result.stderr)
            );
            assert!(
                result.stdout.is_empty() && result.stderr.is_empty(),
                "{name}/{phase}"
            );
        }
    }
    Ok(())
}
