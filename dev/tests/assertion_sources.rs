//! Ordinary source panic headers and bodies do not authorize generated assertions.
use crate::resolve as source;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const PACKET: &str = "pa31/tests/spec/assertion-panics";

#[test]
fn panic_hook_headers_preserve_unused_shapes_and_real_ordinary_body_proof() -> Result {
    let mut failures = Vec::new();
    let mut count = 0;
    for line in source::fixture(&format!("{PACKET}/cases-v2.tsv"))?.lines() {
        let (name, expected) = line.split_once('\t').ok_or("panic case columns")?;
        let text = source::fixture(&format!("{PACKET}/{name}"))?;
        let result = source::source_provider(&text, "panic_headers").and_then(|report| {
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
                    "compiler assertion requires",
                    "budget",
                    "exhausted",
                    "lost its",
                    "changed its",
                    "lacks its completed",
                ]
                .iter()
                .any(|fence| error.contains(fence)) =>
            {
                println!("{name}: {error}");
            }
            (_, result) => failures.push(format!(
                "{name}: {}",
                result.map_or_else(|e| e, |()| "accepted".into())
            )),
        }
        count += 1;
    }
    assert_eq!(count, 22);
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}

#[test]
fn only_actual_emitted_guards_require_the_corresponding_source_panic_runtime() -> Result {
    for line in source::fixture(&format!("{PACKET}/guard-cases-v2.tsv"))?.lines() {
        let (name, phase) = line.split_once('\t').ok_or("guard case columns")?;
        let text = source::fixture(&format!("{PACKET}/{name}"))?;
        let report = source::source_provider(&text, "source_guards")?;
        // Valid source admission/body proof precedes the executable routing boundary.
        let rustgm::pipeline::Analysis {
            mut facts,
            prepared,
        } = rustgm::pipeline::analyze_source(&report)?;
        let result = rustgm::lower::instances::build_roots_prepared(
            &report,
            &mut facts,
            rustgm::lower::Roots {
                entry: None,
                exports: &[],
            },
            prepared,
        );
        if phase == "ready" {
            result.map_err(|error| format!("{name}: {error}"))?;
        } else {
            let error = result
                .err()
                .ok_or_else(|| format!("{name}: source hook silently used course abort"))?;
            assert!(error.to_string().contains("compiler assertion requires checked source panic signature, caller and runtime routing"), "{name}: {error}");
        }
    }
    assert_eq!(size_of::<rustgm::sema::values::Reader<'_>>(), 8);
    assert_eq!(size_of::<rustgm::body::ir::Body>(), 456);
    assert_eq!(size_of::<rustgm::body::ir::Owner>(), 20);
    assert_eq!(size_of::<rustgm::types::Kind>(), 24);
    Ok(())
}

#[test]
#[ignore = "requires the locked CPPGM object backend and a C compiler"]
fn ordinary_marked_panic_functions_execute_all_twenty_eight_actual_bodies() -> Result {
    let text = source::fixture(&format!("{PACKET}/native.rs"))?;
    let report = source::source_provider(&text, "panic_marker_native")?;
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
    let folder = root.join("obj/parallel/panic-hooks-v1/native");
    std::fs::create_dir_all(&folder)?;
    crate::ffi_program::check(
        &root,
        &folder,
        "markers",
        &program,
        &source::fixture(&format!("{PACKET}/observer.c"))?,
        &source::fixture(&format!("{PACKET}/native.expected"))?,
    )
}
