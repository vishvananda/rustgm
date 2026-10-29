//! Early source reads require actual semantic implementation completion.
use super::{TestResult, lower, lower_report};
use std::{path::Path, process::Command};
const POSITIVES: &[&str] = &[
    "reference-array",
    "reversed-implementations",
    "runtime-read",
    "unused-clone-loop",
    "indexed-requirements",
    "generic-runtime-read",
    "generic-clone-prerequisite",
    "associated-type-profile",
    "associated-constant-profile",
    "repeated-8-noise-0",
    "repeated-32-noise-0",
    "repeated-128-noise-0",
    "repeated-8-noise-128",
];
fn fixture(name: &str) -> TestResult<String> {
    Ok(std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../pa31/tests/spec/early-copy-readiness")
            .join(format!("{name}.rs")),
    )?)
}
fn original() -> TestResult<String> {
    Ok(std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../pa31/tests/spec/const-eval-select-ctfe/reference-anonymous-selector.rs"),
    )?)
}
fn ordinary(source: &str) -> TestResult<rustgm::lowir::program::Program> {
    let report = rustgm::resolve::analyze_expanded(rustgm::resolve::expansion::analyze(
        rustgm::resolve::load::InputFile {
            identity: "ordinary.rs".into(),
            path: "ordinary.rs".into(),
            source: rustgm::support::source::SourceFile::new(source.as_bytes().to_vec())?,
        },
        &mut super::NoFiles,
        &rustgm::expand::config::Config::default(),
    )?)?;
    lower_report(&report)
}
#[test]
fn actual_early_copy_controls_reach_complete_original_lowering() -> TestResult {
    for &name in POSITIVES {
        lower(&fixture(name)?).map_err(|e| format!("{name}: {e}"))?;
    }
    ordinary(&fixture("ordinary-no-source-policy")?)?;
    Ok(())
}
#[test]
fn unchanged_selected_reference_caller_reaches_complete_original_lowering() -> TestResult {
    lower(&original()?)?;
    Ok(())
}

#[test]
fn malformed_unused_duplicate_and_cycle_implementations_do_not_publish_permission() -> TestResult {
    for name in [
        "missing-clone",
        "wrong-clone-signature",
        "unused-invalid-clone-body",
        "duplicate-copy",
        "value-signature-cycle",
    ] {
        let error = lower(&fixture(name)?)
            .err()
            .ok_or_else(|| format!("{name}: invalid implementation accepted"))?;
        println!("early_copy_negative {name}: {error}");
    }
    Ok(())
}
#[test]
fn repeated_reads_and_unrelated_source_keep_cumulative_work_observable() -> TestResult {
    for name in [
        "repeated-8-noise-0",
        "repeated-32-noise-0",
        "repeated-128-noise-0",
        "repeated-8-noise-128",
    ] {
        let report = super::report(&fixture(name)?)?;
        let rustgm::pipeline::Analysis {
            mut facts,
            prepared,
        } = rustgm::pipeline::analyze_source(&report)?;
        let before = facts.body_work.canonical_steps;
        rustgm::lower::instances::build_roots_prepared(
            &report,
            &mut facts,
            rustgm::lower::Roots {
                entry: None,
                exports: &[],
            },
            prepared,
        )?;
        println!(
            "early_copy_work {name} preparation={before} total={}",
            facts.body_work.canonical_steps
        );
        assert!(facts.body_work.canonical_steps >= before);
    }
    Ok(())
}
#[test]
#[ignore = "requires the pinned CPPGM backend and host C linker"]
fn real_early_read_and_selector_native_outputs_match_independent_contracts() -> TestResult {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?;
    let folder = root
        .join("obj/parallel/pa31/early-copy-native")
        .join(std::process::id().to_string());
    std::fs::create_dir_all(&folder)?;
    let mut cases = POSITIVES
        .iter()
        .map(|&name| Ok((name.to_owned(), lower(&fixture(name)?)?, "7 7\n")))
        .collect::<TestResult<Vec<_>>>()?;
    cases.push((
        "ordinary-no-source-policy".to_owned(),
        ordinary(&fixture("ordinary-no-source-policy")?)?,
        "7 7\n",
    ));
    let observer = std::fs::read(root.join("pa31/tests/spec/early-copy-readiness/observe.c"))?;
    for (name, program, expected) in cases {
        let ir = folder.join(format!("{name}.lowir"));
        let object = folder.join(format!("{name}.o"));
        let binary = folder.join(format!("{name}.native"));
        let client = folder.join(format!("{name}.c"));
        let mut text = Vec::new();
        rustgm::lowir::writer::write(&program, &mut text)?;
        std::fs::write(&ir, text)?;
        std::fs::write(&client, &observer)?;
        super::native::run(
            &folder,
            &format!("{name}-backend"),
            Command::new("python3")
                .arg(root.join("scripts/backend.py"))
                .arg("--emit-object")
                .arg(&ir)
                .arg("-o")
                .arg(&object),
        )?;
        super::native::run(
            &folder,
            &format!("{name}-link"),
            Command::new("cc")
                .arg(&client)
                .arg(&object)
                .arg("-o")
                .arg(&binary),
        )?;
        super::native::run(
            &folder,
            &format!("{name}-execute"),
            &mut Command::new(&binary),
        )?;
        assert_eq!(
            std::fs::read(folder.join(format!("{name}-execute.stdout")))?,
            expected.as_bytes()
        );
        super::native::run(
            &folder,
            &format!("{name}-hashes"),
            Command::new("sha256sum").arg(&object).arg(&binary),
        )?;
        std::fs::remove_file(object)?;
        std::fs::remove_file(binary)?;
    }
    Ok(())
}
