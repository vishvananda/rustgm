//! Actual async call traits retain ordinary implementations and borrowed futures.
use crate::resolve as source;
use rustgm::pipeline;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

#[test]
fn actual_async_call_headers_and_bodies_match_frozen_language_outcomes() -> Result {
    check_cases("cases.tsv", 25)
}

#[test]
fn automatic_async_calls_require_real_future_capture_abi_and_lifetime_proofs() -> Result {
    check_cases("implicit-cases.tsv", 21)
}

fn check_cases(manifest: &str, expected_count: usize) -> Result {
    let manifest = source::fixture(&format!("pa31/tests/spec/async-calls/{manifest}"))?;
    let mut failures = Vec::new();
    let mut count = 0;
    for line in manifest.lines() {
        let (name, status) = line.split_once('\t').ok_or("case columns")?;
        let text = source::fixture(&format!("pa31/tests/spec/async-calls/{name}"))?;
        let result = source::source_provider(&text, "async_calls").and_then(|report| {
            let pipeline::Analysis {
                mut facts,
                prepared,
            } = pipeline::analyze_source(&report).map_err(|e| e.to_string())?;
            rustgm::lower::instances::inspect_prepared(&report, &mut facts, prepared, |_, _| ())
                .map_err(|e| e.to_string())
        });
        match (status, result) {
            ("0", Ok(())) => {}
            ("1", Err(error))
                if ![
                    "semantic contract is not implemented",
                    "invalid or exhausted",
                    "unknown feature",
                    "not implemented",
                    "associated type requires an unambiguous implementation",
                ]
                .iter()
                .any(|fence| error.contains(fence)) => {}
            (_, result) => failures.push(format!(
                "{name}: {}",
                result.map_or_else(|e| e, |()| "accepted".into())
            )),
        }
        count += 1;
    }
    assert_eq!(count, expected_count);
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}

#[test]
#[ignore = "requires the locked CPPGM object backend and a C compiler"]
fn actual_async_methods_invoke_their_selected_source_functions_and_closures() -> Result {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let folder = root.join("obj/parallel/async-call-source-v1/native-methods");
    std::fs::create_dir_all(&folder)?;
    for (name, observer, expected) in [
        ("manual", "manual.c", "23 17 11 29 31\n"),
        ("generic", "generic.c", "23 17 11\n"),
        ("ordinary", "ordinary.c", "23\n"),
        ("loan", "loan.c", "23\n"),
        ("implicit-method-function", "implicit.c", "23\n"),
        ("implicit-method-pointer", "implicit.c", "23\n"),
        ("implicit-method-closure", "implicit.c", "23\n"),
        ("implicit-method-shared", "implicit.c", "23\n"),
        ("implicit-method-mutable", "implicit.c", "23\n"),
    ] {
        let text = source::fixture(&format!("pa31/tests/spec/async-calls/{name}.rs"))?;
        let client = source::fixture(&format!("pa31/tests/spec/async-calls/{observer}"))?;
        let report = source::source_provider(&text, "lending_native")?;
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
        crate::ffi_program::check(&root, &folder, name, &program, &client, expected)?;
    }
    Ok(())
}
