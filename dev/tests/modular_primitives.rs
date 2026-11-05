//! Modular primitive calls preserve intrinsic identities and ordinary lookalikes.
use crate::{ffi_program, resolve as test_source};
use rustgm::{lower, pipeline, resolve::model::DefId};
use std::path::Path;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const FIXTURE: &str = "pa28/tests/spec/source-wrapping";

fn program(name: &str) -> Result<rustgm::lowir::program::Program> {
    let source = test_source::fixture(&format!("{FIXTURE}/{name}"))?;
    let report = test_source::source_provider(&source, "source_wrapping")?;
    let pipeline::Analysis {
        mut facts,
        prepared,
    } = pipeline::analyze_source(&report)?;
    let mut exports = Vec::new();
    for (index, definition) in report.resolution.definitions.iter().enumerate() {
        let name = report.package.names.get(definition.name);
        if name.starts_with("wrap_")
            || name.starts_with("fold_")
            || matches!(name, "ordinary" | "forward_add" | "float")
        {
            exports.push(DefId(u32::try_from(index)?));
        }
    }
    Ok(lower::instances::build_roots_prepared(
        &report,
        &mut facts,
        lower::Roots {
            entry: None,
            exports: &exports,
        },
        prepared,
    )?)
}

#[test]
fn all_integer_wrapping_widths_keep_declared_signatures_and_checked_call_boundaries() -> Result {
    let cases = test_source::fixture(&format!("{FIXTURE}/cases.tsv"))?;
    for line in cases.lines() {
        let (name, expected) = line.split_once('\t').ok_or("columns")?;
        match (expected, program(name)) {
            ("0", Ok(_)) => {}
            ("1", Err(e)) if !e.to_string().contains("lowering is not implemented") => {}
            (_, result) => {
                return Err(format!(
                    "{name}: {}",
                    result
                        .err()
                        .map_or_else(|| "accepted".to_owned(), |e| e.to_string())
                )
                .into());
            }
        }
    }
    Ok(())
}

#[test]
#[ignore = "requires the locked object backend and independent C observers"]
fn actual_modular_results_and_ordinary_same_name_body_reach_native() -> Result {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?;
    let folder = root.join(format!(
        "obj/parallel/source-wrapping-v1/native-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&folder)?;
    for (file, observer, expected) in [
        ("runtime.rs", "observer.c", "72 wrapping cases\n"),
        (
            "generic-forward.rs",
            "generic-observer.c",
            "73 wrapping cases\n",
        ),
        ("ordinary.rs", "ordinary-observer.c", "23\n"),
        ("float.rs", "float-observer.c", "3.0\n"),
    ] {
        ffi_program::check(
            root,
            &folder,
            file,
            &program(file)?,
            &test_source::fixture(&format!("{FIXTURE}/{observer}"))?,
            expected,
        )?;
    }
    Ok(())
}
