//! Initializer extension retains evaluated backing without reviving local storage.
use crate::resolve as source;
use rustgm::{lower, pipeline, resolve::model::DefId};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

fn report(owner: &str, name: &str) -> Result<rustgm::resolve::expansion::Report> {
    let text = source::fixture(&format!("{owner}/tests/spec/initializer-storage/{name}.rs"))?;
    Ok(if owner == "pa31" {
        source::source_provider(&text, "initializer_storage")?
    } else {
        source::resolve(&text, true)?
    })
}
fn program(owner: &str, name: &str) -> Result<rustgm::lowir::program::Program> {
    let report = report(owner, name)?;
    let pipeline::Analysis {
        mut facts,
        prepared,
    } = pipeline::analyze_source(&report)?;
    let entry = if matches!(owner, "pa25" | "pa27") {
        Some(
            report
                .resolution
                .definitions
                .iter()
                .position(|d| report.package.names.get(d.name) == "main")
                .and_then(|index| u32::try_from(index).ok())
                .map(DefId)
                .ok_or("ordinary fixture lacks its required main entry")?,
        )
    } else {
        None
    };
    let program = lower::instances::build_roots_prepared(
        &report,
        &mut facts,
        lower::Roots {
            entry,
            exports: &[],
        },
        prepared,
    )?;
    assert!(program.symbols.iter().any(|symbol| symbol.body.is_some()));
    if matches!(owner, "pa25" | "pa27") {
        assert!(program.symbols.iter().any(|symbol| symbol.entry));
    }
    Ok(program)
}

#[test]
fn computed_initializer_backing_survives_ordinary_and_source_frames() -> Result {
    for owner in ["pa25", "pa28"] {
        for name in [
            "static-rows",
            "const-rows",
            "named-control",
            "scalar",
            "computed-block",
        ] {
            program(owner, name).map_err(|error| format!("{owner}/{name}: {error}"))?;
        }
    }
    program("pa25", "mutable-local")?;
    for name in ["source-rows", "scalar"] {
        program("pa31", name).map_err(|error| format!("pa31/{name}: {error}"))?;
    }
    Ok(())
}

#[test]
fn extension_keeps_scope_and_nested_dangling_opponents() -> Result {
    for (owner, name) in [
        ("pa25", "mutated-temporary"),
        ("pa25", "unpaid-lifetime"),
        ("pa25", "mutable-constant"),
        ("pa25", "mutable-static"),
        ("pa25", "mutable-projection"),
        ("pa27", "dangling-local"),
        ("pa27", "dangling-callee"),
        ("pa27", "nested-dangling"),
    ] {
        let error = program(owner, name)
            .err()
            .ok_or_else(|| format!("{owner}/{name} accepted"))?
            .to_string();
        assert!(
            !error.contains("not implemented"),
            "{owner}/{name}: {error}"
        );
        assert!(!error.contains("budget"), "{owner}/{name}: {error}");
        assert!(
            error.contains("expired storage")
                || error.contains("lifetime")
                || error.contains("borrowed mutably"),
            "{owner}/{name}: {error}"
        );
        println!("{owner}/{name}: {error}");
    }
    Ok(())
}

#[test]
fn evaluated_backing_has_bounded_preparation_and_execution_work() -> Result {
    let mut previous: Option<[usize; 4]> = None;
    for count in [8, 32, 128] {
        let report = report("pa25", &format!("scaling-{count}"))?;
        let analysis = pipeline::analyze_source(&report)?;
        let work = [
            analysis.facts.body_work.inference_steps,
            analysis.facts.body_work.canonical_steps,
            analysis.facts.constant_work.steps,
            analysis.facts.constant_work.code_bytes,
        ];
        if let Some(old) = previous {
            assert!(work.into_iter().zip(old).all(|(new, old)| new <= old * 5));
        }
        println!("extended backing count={count} work={work:?}");
        previous = Some(work);
        program("pa25", &format!("scaling-{count}"))?;
    }
    Ok(())
}

#[test]
#[ignore = "requires pinned CPPGM and host C linker"]
fn ordinary_and_source_extended_tables_execute() -> Result {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let folder = root
        .join("obj/parallel/extended-temporaries/native")
        .join(std::process::id().to_string());
    std::fs::create_dir_all(&folder)?;
    for (owner, name, observer, expected) in [
        ("pa28", "static-rows", "observe.c", "7 11 13 14 2 2\n"),
        ("pa28", "const-rows", "observe.c", "7 11 13 14 2 2\n"),
        ("pa28", "named-control", "observe.c", "7 11 13 14 2 2\n"),
        ("pa28", "scalar", "observe-scalar.c", "7\n"),
        ("pa28", "computed-block", "observe-scalar.c", "9\n"),
        (
            "pa31",
            "source-rows",
            "observe.c",
            "192 22 32 256 1 1 304 775 2 0\n",
        ),
        ("pa31", "scalar", "observe-scalar.c", "7\n"),
    ] {
        crate::ffi_program::check(
            &root,
            &folder,
            &format!("{owner}-{name}"),
            &program(owner, name)?,
            &source::fixture(&format!(
                "{owner}/tests/spec/initializer-storage/{observer}"
            ))?,
            expected,
        )?;
    }
    Ok(())
}
