//! Explicit pattern modifiers retain the edition of their parsed introducer.
use rustgm::support::edition::Edition;
use std::{path::Path, process::Command};

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const CASES: &[(&str, bool, bool)] = &[
    ("plain", true, true),
    ("mut-reset", true, false),
    ("ref-shared", true, false),
    ("ref-mut", true, false),
    ("ref-mut-shared", false, false),
    ("reference-reset", true, false),
    ("explicit-outer", true, true),
    ("at-binding", true, false),
    ("macro-generated", true, false),
    ("opaque-forwarded", true, false),
    ("shorthand", true, false),
    ("reference-tuple-reset", true, false),
    ("noncopy-move", false, false),
];

fn source(name: &str) -> Result<String> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("missing root")?;
    Ok(std::fs::read_to_string(
        root.join("pa34/tests/spec/pattern-editions")
            .join(format!("{name}.rs")),
    )?)
}

#[test]
fn paired_binding_modes_reach_the_complete_typed_pipeline() -> Result {
    for &(name, older, newer) in CASES {
        let source = source(name)?;
        for (edition, accepted) in [(Edition::Rust2021, older), (Edition::Rust2024, newer)] {
            let result = crate::edition_packages::lowir(&source, edition);
            match result {
                Ok(bytes) => assert!(accepted && !bytes.is_empty(), "{name}/{edition:?}"),
                Err(error) => {
                    assert!(!accepted, "{name}/{edition:?}: {error}");
                    let error = error.to_string();
                    if edition == Edition::Rust2024 {
                        assert!(
                            error.contains("default binding mode in edition 2024"),
                            "{error}"
                        );
                    } else {
                        assert!(
                            error.contains("mutab")
                                || error.contains("shared")
                                || error.contains("borrow")
                                || error.contains("move"),
                            "{error}"
                        );
                    }
                }
            }
        }
    }
    Ok(())
}

#[test]
#[ignore = "requires pinned rustc and CPPGM native backend"]
fn binding_values_and_mutations_match_each_selected_edition() -> Result {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("missing root")?;
    let folder = root.join("obj/parallel/pa34-pattern-native").join(format!(
        "{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos()
    ));
    std::fs::create_dir_all(&folder)?;
    for &(name, older, newer) in CASES {
        for (edition, accepted) in [(Edition::Rust2021, older), (Edition::Rust2024, newer)] {
            if accepted {
                native(&folder, name, edition, root)?;
            }
        }
    }
    Ok(())
}

fn native(folder: &Path, name: &str, edition: Edition, root: &Path) -> Result {
    use crate::edition_packages::run;
    let flag = match edition {
        Edition::Rust2021 => "--edition=2021",
        Edition::Rust2024 => "--edition=2024",
    };
    let tag = format!("{name}-{edition:?}");
    let source = source(name)?;
    let input = folder.join(format!("{tag}.rs"));
    let ir = folder.join(format!("{tag}.lowir"));
    let reference = folder.join(format!("{tag}.reference"));
    let candidate = folder.join(format!("{tag}.candidate"));
    std::fs::write(&input, &source)?;
    run(
        folder,
        &format!("{tag}-reference"),
        Command::new("rustc")
            .args([
                "+1.98.0",
                flag,
                "--crate-name=probe",
                "--cap-lints=allow",
                "-C",
                "panic=abort",
                "-C",
                "overflow-checks=yes",
            ])
            .arg(input)
            .arg("-o")
            .arg(&reference),
    )?;
    std::fs::write(&ir, crate::edition_packages::lowir(&source, edition)?)?;
    run(
        folder,
        &format!("{tag}-backend"),
        Command::new("python3")
            .arg(root.join("scripts/backend.py"))
            .arg(ir)
            .arg("-o")
            .arg(&candidate),
    )?;
    for (kind, binary) in [("reference", reference), ("candidate", candidate)] {
        let executed = format!("{tag}-{kind}-run");
        run(folder, &executed, &mut Command::new(binary))?;
        for stream in ["stdout", "stderr"] {
            assert!(std::fs::read(folder.join(format!("{executed}.{stream}")))?.is_empty());
        }
    }
    Ok(())
}
