//! Arbitrary method receivers expose owners without manufacturing dereference.
use crate::resolve as test_source;
use rustgm::{
    lower, pipeline,
    resolve::model::{DefId, DefKind},
};
use std::{path::Path, process::Command};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

fn program(name: &str) -> Result<(rustgm::lowir::program::Program, usize)> {
    let source = test_source::fixture(&format!("pa31/tests/spec/receiver-source/{name}"))?;
    let report = test_source::source_provider(&source, "receiver_source")?;
    let pipeline::Analysis {
        mut facts,
        prepared,
    } = pipeline::analyze_source(&report)?;
    let work = facts.methods.selection_steps;
    let mut roots = Vec::new();
    for (index, definition) in report.resolution.definitions.iter().enumerate() {
        let id = DefId(u32::try_from(index)?);
        if definition.kind == DefKind::Function
            && facts.associated.owner(id).is_none()
            && matches!(
                report.package.names.get(definition.name),
                "probe"
                    | "direct"
                    | "dot"
                    | "shared"
                    | "nested"
                    | "once"
                    | "generic_read"
                    | "raw_read"
            )
        {
            roots.push(id);
        }
    }
    let program = lower::instances::build_roots_prepared(
        &report,
        &mut facts,
        lower::Roots {
            entry: None,
            exports: &roots,
        },
        prepared,
    )?;
    Ok((program, work))
}

#[test]
fn actual_receiver_targets_preserve_ordinary_headers_and_type_only_reachability() -> Result {
    let cases = test_source::fixture("pa31/tests/spec/receiver-source/cases.tsv")?;
    let mut failures = Vec::new();
    let mut count = 0;
    let mut work = Vec::new();
    for line in cases.lines() {
        let (name, expected) = line.split_once('\t').ok_or("case columns")?;
        match (expected, program(name)) {
            ("0", Ok((_, steps))) => {
                if name == "dot.rs" || name.starts_with("bound-") {
                    work.push(steps);
                }
            }
            ("1", Err(e))
                if ![
                    "semantic contract is not implemented",
                    "unknown feature",
                    "expected a token",
                ]
                .iter()
                .any(|s| e.to_string().contains(s)) => {}
            (_, result) => failures.push(format!(
                "{name}: {}",
                result
                    .err()
                    .map_or_else(|| "accepted".to_owned(), |e| e.to_string())
            )),
        }
        count += 1;
    }
    assert_eq!(count, 44);
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    assert_eq!(work.len(), 4);
    let growth = work[1..]
        .iter()
        .map(|n| n.checked_sub(work[0]).ok_or("work decreased"))
        .collect::<std::result::Result<Vec<_>, _>>()?;
    eprintln!(
        "actual Receiver selection work: baseline={}, growth={growth:?}",
        work[0]
    );
    assert!(
        growth[0] > 0
            && growth[1] <= growth[0].saturating_mul(5)
            && growth[2] <= growth[1].saturating_mul(5),
        "{growth:?}"
    );
    Ok(())
}

fn checked(label: &str, command: &mut Command, folder: &Path) -> Result<Vec<u8>> {
    let output = command.output()?;
    std::fs::write(folder.join(format!("{label}.stderr")), &output.stderr)?;
    if !output.status.success() {
        return Err(format!("{label}: {}", String::from_utf8_lossy(&output.stderr)).into());
    }
    Ok(output.stdout)
}

#[test]
#[ignore = "requires current locked CPPGM object backend and host C compiler"]
fn real_receiver_methods_execute_once_with_generic_and_raw_pointer_contracts() -> Result {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?;
    for (name, observer, expected) in [
        ("runtime.rs", "observer.c", "23 17 11 29 32\n"),
        ("generic-native.rs", "generic-native-observer.c", "23\n"),
        ("raw-native.rs", "raw-native-observer.c", "23\n"),
    ] {
        let folder = root.join(format!(
            "obj/parallel/receiver-source-v1/native-{name}-{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&folder)?;
        let ir = folder.join("source.lowir");
        rustgm::lowir::writer::write(&program(name)?.0, &mut std::fs::File::create(&ir)?)?;
        let object = folder.join("source.o");
        checked(
            "backend",
            Command::new("python3")
                .current_dir(root)
                .arg("scripts/backend.py")
                .arg("--emit-object")
                .arg(&ir)
                .arg("-o")
                .arg(&object),
            &folder,
        )?;
        let binary = folder.join("native");
        checked(
            "link",
            Command::new("cc")
                .arg(root.join("pa31/tests/spec/receiver-source").join(observer))
                .arg(&object)
                .arg("-o")
                .arg(&binary),
            &folder,
        )?;
        assert_eq!(
            checked("execute", &mut Command::new(binary), &folder)?,
            expected.as_bytes()
        );
    }
    Ok(())
}
