//! Ordinary semantic consumers retain real source Field calls before CFG/loan completion.
use crate::resolve as source;
use rustgm::{sema, syntax::ast::Kind};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

#[test]
fn field_semantic_producers_keep_actual_calls_and_complete_target_applications() -> Result {
    let mut failures = Vec::new();
    for name in [
        "shared",
        "mutable",
        "late-mut",
        "nested",
        "shadow",
        "raw-reference",
        "lifetime",
    ] {
        let text = source::fixture(&format!("pa31/tests/spec/source-field-deref/{name}.rs"))?;
        let report = source::source_provider(&text, "field_semantics")?;
        let result = sema::declarations::analyze(&report)
            .and_then(|facts| sema::bodies::analyze(&report, facts));
        let facts = match result {
            Ok(facts) => facts,
            Err(error) => {
                failures.push(format!("{name}: {error}"));
                continue;
            }
        };
        let plans = facts
            .nodes
            .iter()
            .flatten()
            .filter(|node| {
                matches!(
                    node.operation,
                    Some(sema::model::Operation::Field { plan: Some(_), .. })
                )
            })
            .count();
        if name == "shadow" {
            assert_eq!(plans, 0);
        } else {
            assert!(plans > 0, "{name}: no actual field plan");
        }
        for site in facts.methods.calls.keys() {
            assert!(
                !matches!(
                    report
                        .package
                        .unit(site.file)
                        .configured
                        .ast
                        .node(site.node)
                        .kind,
                    Kind::Field { .. }
                ),
                "{name}: a Field step overwrote a single-call fact"
            );
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}

#[test]
fn field_calls_preserve_mutability_lifetime_move_and_const_requirements() -> Result {
    let mut failures = Vec::new();
    let mut count = 0;
    for line in source::fixture("pa31/tests/spec/source-field-deref/cases.tsv")?.lines() {
        let (name, expected) = line.split_once('\t').ok_or("field case columns")?;
        let text = source::fixture(&format!("pa31/tests/spec/source-field-deref/{name}"))?;
        let result = source::source_provider(&text, "field_consumer").and_then(|report| {
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
                    "invalid or exhausted",
                    "budget exceeded",
                    "lost its actual",
                    "changed its actual",
                    "field construction",
                    "field call lost",
                    "field output view",
                    "missing associated",
                    "unknown feature",
                    "source retention lacks",
                    "attribute retention lacks",
                    "caller inputs exceed source work/storage budget",
                    "source contract work or retention exceeds profile limit",
                    "source header contract query work exceeds profile limit",
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
    assert_eq!(count, 19);
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}

#[test]
#[ignore = "requires the locked CPPGM object backend and a C compiler"]
fn field_call_order_temporary_once_shadow_and_const_mutability_reach_native() -> Result {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let folder = root.join("obj/parallel/async-iterator-v1/field-native");
    std::fs::create_dir_all(&folder)?;
    for (name, observer, expected) in [
        (
            "native",
            "observer.c",
            "call=1\nshared=13\ncall=2\nlate=23\ncall=3\ncall=1\ntemporary=13\ncall=4\ncall=1\nnested=13\nshadow=41\n",
        ),
        ("const-mut", "const-observer.c", "const=23\n"),
    ] {
        let text = source::fixture(&format!("pa31/tests/spec/source-field-deref/{name}.rs"))?;
        let report = source::source_provider(&text, "field_native")?;
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
        crate::ffi_program::check(
            &root,
            &folder,
            name,
            &program,
            &source::fixture(&format!("pa31/tests/spec/source-field-deref/{observer}"))?,
            expected,
        )?;
    }
    Ok(())
}

#[test]
fn source_field_receiver_borrow_reads_union_storage_in_its_lexical_context() -> Result {
    for (name, accepted) in [("union-borrow-safe", false), ("union-borrow-unsafe", true)] {
        let text = source::fixture(&format!("pa31/tests/spec/source-field-deref/{name}.rs"))?;
        let report = source::source_provider(&text, "union_field_receiver")?;
        let result = rustgm::pipeline::analyze_source(&report);
        if accepted {
            let rustgm::pipeline::Analysis {
                mut facts,
                prepared,
            } = result?;
            rustgm::lower::instances::inspect_prepared(&report, &mut facts, prepared, |_, _| ())?;
        } else {
            let error = result
                .err()
                .ok_or("unsafe union receiver borrow accepted")?;
            assert!(
                error
                    .to_string()
                    .contains("union field access requires an unsafe context"),
                "{error}"
            );
        }
    }
    Ok(())
}
