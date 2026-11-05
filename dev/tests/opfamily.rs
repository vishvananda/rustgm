//! Five actual source operator families reuse ordinary call and effect proof.
use crate::resolve as test_source;
use rustgm::{pipeline, resolve::expansion::Report, sema::model::Declarations};
use std::{path::Path, process::Command};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
fn source(name: &str) -> Result<String> {
    Ok(test_source::fixture(&format!(
        "pa31/tests/spec/source-binary-five/{name}.rs"
    ))?)
}
fn analyzed(name: &str) -> Result<(Report, pipeline::Analysis)> {
    let report = test_source::source_provider(&source(name)?, "binary_five")?;
    let facts = pipeline::analyze_source(&report)?;
    Ok((report, facts))
}
#[test]
fn complete_operator_declarations_keep_type_effect_and_unused_header_boundaries() -> Result {
    let cases = test_source::fixture("pa31/tests/spec/source-binary-five/cases.tsv")?;
    let mut count = 0;
    for row in cases.lines() {
        let (name, code) = row.split_once('\t').ok_or("case columns")?;
        let name = name.strip_suffix(".rs").ok_or("source suffix")?;
        let result = analyzed(name);
        match (code, result) {
            ("0", Ok(_)) => {}
            ("1", Err(error)) => {
                assert!(
                    !error
                        .to_string()
                        .contains("semantic contract is not implemented"),
                    "{name}: {error}"
                );
            }
            (_, result) => {
                return Err(format!(
                    "{name}: expected {code}, observed {}",
                    result
                        .err()
                        .map_or_else(|| "accepted".to_owned(), |e| e.to_string())
                )
                .into());
            }
        }
        count += 1;
    }
    assert_eq!(count, 49);
    Ok(())
}
fn steps(facts: &Declarations) -> usize {
    facts.body_work.canonical_steps
}
#[test]
fn actual_operator_query_work_scales_with_demanded_source_operations() -> Result {
    let (_, base) = analyzed("runtime")?;
    let initial = steps(&base.facts);
    let mut growth = Vec::new();
    for count in [8, 32, 128] {
        let (_, measured) = analyzed(&format!("bound-{count}"))?;
        growth.push(
            steps(&measured.facts)
                .checked_sub(initial)
                .ok_or("nonmonotone work")?,
        );
    }
    eprintln!("source operator canonical work: base={initial}, growth={growth:?}");
    assert!(growth[0] > 0 && growth[1] >= growth[0]);
    assert!(growth[2] <= growth[1].saturating_mul(5), "{growth:?}");
    Ok(())
}
fn program(name: &str) -> Result<rustgm::lowir::program::Program> {
    let (
        report,
        pipeline::Analysis {
            mut facts,
            prepared,
        },
    ) = analyzed(name)?;
    Ok(rustgm::lower::instances::build_roots_prepared(
        &report,
        &mut facts,
        rustgm::lower::Roots {
            entry: None,
            exports: &[],
        },
        prepared,
    )?)
}
#[test]
fn actual_custom_and_primitive_operators_lower_through_real_bodies() -> Result {
    for name in ["runtime", "primitive"] {
        let program = program(name)?;
        assert!(program.symbols.iter().any(|s| s.body.is_some()));
    }
    Ok(())
}
fn native(name: &str, observer: &str, expected: &[u8]) -> Result {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("repository")?;
    let folder = root.join(format!(
        "obj/parallel/source-transport/operators/native-{name}-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&folder)?;
    let lowir = folder.join("operators.lowir");
    rustgm::lowir::writer::write(&program(name)?, &mut std::fs::File::create(&lowir)?)?;
    let object = folder.join("operators.o");
    let output = Command::new("python3")
        .arg(root.join("scripts/backend.py"))
        .arg("--emit-object")
        .arg(&lowir)
        .arg("-o")
        .arg(&object)
        .output()?;
    std::fs::write(folder.join("backend.stderr"), &output.stderr)?;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let executable = folder.join("observer");
    let output = Command::new("cc")
        .arg(
            root.join("pa31/tests/spec/source-binary-five")
                .join(observer),
        )
        .arg(object)
        .arg("-o")
        .arg(&executable)
        .output()?;
    std::fs::write(folder.join("link.stderr"), &output.stderr)?;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let output = Command::new(executable).output()?;
    std::fs::write(folder.join("native.stdout"), &output.stdout)?;
    std::fs::write(folder.join("native.stderr"), &output.stderr)?;
    assert!(output.status.success());
    assert_eq!(output.stdout, expected);
    assert!(output.stderr.is_empty());
    Ok(())
}

#[test]
fn prior_source_shift_and_or_contracts_still_lower_actual_methods() -> Result {
    let text = test_source::fixture("pa31/tests/spec/source-shift-or/runtime-overloads.rs")?;
    let report = test_source::source_provider(&text, "binary_legacy")?;
    let pipeline::Analysis {
        mut facts,
        prepared,
    } = pipeline::analyze_source(&report)?;
    let program = rustgm::lower::instances::build_roots_prepared(
        &report,
        &mut facts,
        rustgm::lower::Roots {
            entry: None,
            exports: &[],
        },
        prepared,
    )?;
    for name in ["encoded", "shifted", "ored"] {
        assert!(
            program
                .symbols
                .iter()
                .any(|s| s.name == name && s.body.is_some()),
            "{name}"
        );
    }
    Ok(())
}

#[test]
#[ignore = "requires the configured locked CPPGM object backend and host C compiler"]
fn actual_custom_operators_execute_selected_source_methods_natively() -> Result {
    native("runtime", "observer.c", b"11 13 15 17 19 42 44 46 48 50\n")
}
#[test]
#[ignore = "requires the configured locked CPPGM object backend and host C compiler"]
fn actual_runtime_operand_data_flows_through_selected_source_operator() -> Result {
    native("runtime-input", "input-observer.c", b"23\n")
}
