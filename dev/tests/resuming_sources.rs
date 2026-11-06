//! Resume tags retain ordinary fields, bodies and unsafe obligations.
use crate::resolve as source;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const PACKET: &str = "pa31/tests/spec/resume-context";

#[test]
fn actual_resume_roles_keep_header_body_and_unsafe_rules() -> Result {
    let mut failures = Vec::new();
    let mut count = 0;
    for row in source::fixture(&format!("{PACKET}/cases.tsv"))?.lines() {
        let (path, expected) = row.split_once('\t').ok_or("case columns")?;
        let text = source::fixture(path)?;
        let result = source::source_provider(&text, "resume_roles").and_then(|report| {
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
                    "exhausted",
                    "budget",
                    "lost its",
                    "lacks its",
                ]
                .iter()
                .any(|fence| error.contains(fence)) => {}
            (_, result) => failures.push(format!("{path}: {result:?}")),
        }
        count += 1;
    }
    assert_eq!(count, 13);
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}

#[test]
fn pointer_body_companion_preserves_the_exact_original_get_context() -> Result {
    let original = source::fixture(&format!("{PACKET}/original-future.rs"))?;
    let (_, helper) = original
        .split_once("#[lang = \"get_context\"]")
        .ok_or("actual original helper")?;
    let companion = source::fixture(&format!("{PACKET}/body-native.rs"))?;
    assert!(companion.contains(&format!("#[lang = \"get_context\"]{helper}")));
    Ok(())
}

#[test]
#[ignore = "requires the pinned CPPGM object backend and a C compiler"]
fn original_pointer_body_executes_with_the_authored_context_companion() -> Result {
    let text = source::fixture(&format!("{PACKET}/body-native.rs"))?;
    let report = source::source_provider(&text, "resume_body")?;
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
    let folder = root.join("obj/parallel/resume-context-contracts/candidate-native");
    std::fs::create_dir_all(&folder)?;
    crate::ffi_program::check(
        &root,
        &folder,
        "resume_body",
        &program,
        &source::fixture(&format!("{PACKET}/observer.c"))?,
        "23 47\n",
    )?;
    Ok(())
}
