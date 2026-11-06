//! Actual generic source forwarding keeps ordinary declarations and body proof.
use crate::resolve as source;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const PACKET: &str = "pa31/tests/spec/from-yeet-source";

#[test]
fn actual_from_yeet_keeps_generic_dispatch_and_ordinary_header_opponents() -> Result {
    let mut failures = Vec::new();
    let mut count = 0;
    for line in source::fixture(&format!("{PACKET}/cases.tsv"))?.lines() {
        let (name, expected) = line.split_once('\t').ok_or("case columns")?;
        let text = source::fixture(&format!("{PACKET}/{name}"))?;
        let result = source::source_provider(&text, "from_yeet_provider").and_then(|report| {
            let rustgm::pipeline::Analysis {
                mut facts,
                prepared,
            } = rustgm::pipeline::analyze_source(&report).map_err(|e| e.to_string())?;
            rustgm::lower::instances::inspect_prepared(&report, &mut facts, prepared, |_, _| ())
                .map_err(|e| e.to_string())
        });
        match (expected, result) {
            ("0", Ok(())) => {}
            ("1", Err(error))
                if ![
                    "not implemented",
                    "budget",
                    "exhausted",
                    "lost its",
                    "lacks its",
                ]
                .iter()
                .any(|fence| error.contains(fence)) => {}
            (_, result) => failures.push(format!("{name}: {result:?}")),
        }
        count += 1;
    }
    assert_eq!(count, 12);
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}

#[test]
#[ignore = "requires the pinned CPPGM object backend and a C compiler"]
fn from_yeet_executes_the_actual_generic_function_and_selected_trait_body() -> Result {
    let text = source::fixture(&format!("{PACKET}/explicit.rs"))?;
    let report = source::source_provider(&text, "from_yeet_native")?;
    let rustgm::pipeline::Analysis {
        mut facts,
        prepared,
    } = rustgm::pipeline::analyze_source(&report)?;
    let program = rustgm::lower::instances::build_roots_prepared(
        &report,
        &mut facts,
        rustgm::lower::Roots {
            entry: None,
            exports: &[],
        },
        prepared,
    )?;
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let folder = root.join("obj/parallel/from-yeet-source/candidate-native");
    std::fs::create_dir_all(&folder)?;
    crate::ffi_program::check(
        &root,
        &folder,
        "from_yeet",
        &program,
        &source::fixture(&format!("{PACKET}/observer.c"))?,
        &source::fixture(&format!("{PACKET}/native.expected"))?,
    )?;
    Ok(())
}
