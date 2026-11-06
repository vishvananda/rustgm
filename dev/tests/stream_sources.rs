//! Source `AsyncIterator` retains ordinary implementation and supertrait evidence.
use crate::resolve as source;
use rustgm::pipeline;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

#[test]
fn async_iterator_source_roles_and_consumers_match_actual_outcomes() -> Result {
    let manifest = source::fixture("pa31/tests/spec/async-iterator/cases.tsv")?;
    let mut failures = Vec::new();
    let mut count = 0;
    for line in manifest.lines() {
        let (name, status) = line.split_once('\t').ok_or("two case columns")?;
        let text = source::fixture(&format!("pa31/tests/spec/async-iterator/{name}"))?;
        let report = if name.starts_with("receiver-plain") {
            source::resolve(&text, true)
        } else {
            source::source_provider(&text, "stream_source")
        };
        let result = report.and_then(|report| {
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
                    "invalid lifetime constraint identity",
                    "missing associated",
                    "const parameter cannot be used as a type",
                    "changed its actual schema",
                    "budget exceeded",
                    "attribute retention lacks its actual producer budget",
                    "caller inputs exceed source work/storage budget",
                    "source contract work or retention exceeds profile limit",
                    "source header contract query work exceeds profile limit",
                    "invalid complete structural",
                    "invalid structural fingerprint",
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
    assert_eq!(count, 35);
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}

#[test]
#[ignore = "requires the locked CPPGM object backend and a C compiler"]
fn async_iterator_calls_execute_real_poll_forwarding_and_helper_bodies() -> Result {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let folder = root.join("obj/parallel/async-iterator-v1/native");
    std::fs::create_dir_all(&folder)?;
    for (name, observer, expected) in [
        ("native", "observe.c", "23 0 0\n"),
        ("forward", "forward.c", "23 23\n"),
        ("helpers", "helpers.c", "23 1 0\n"),
        ("into", "into.c", "23\n"),
    ] {
        let text = source::fixture(&format!("pa31/tests/spec/async-iterator/{name}.rs"))?;
        let report = source::source_provider(&text, "stream_native")?;
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
        crate::ffi_program::check(
            &root,
            &folder,
            name,
            &program,
            &source::fixture(&format!("pa31/tests/spec/async-iterator/{observer}"))?,
            expected,
        )?;
    }
    Ok(())
}

#[test]
fn original_async_iterator_forwarders_retain_actual_operand_borrows() -> Result {
    let original = source::fixture("pa31/tests/spec/async-iterator/original.rs")?;
    for name in ["whole.rs", "helpers.rs"] {
        let text = source::fixture(&format!("pa31/tests/spec/async-iterator/{name}"))?;
        assert!(text.contains(&original));
        let report = source::source_provider(&text, "stream_forwarder")?;
        let pipeline::Analysis {
            mut facts,
            prepared,
        } = pipeline::analyze_source(&report)?;
        rustgm::lower::instances::inspect_prepared(&report, &mut facts, prepared, |_, _| ())?;
    }
    Ok(())
}
