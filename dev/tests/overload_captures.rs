//! Capture the genuine borrowed receiver prefix; execute Field calls in the body.
use crate::resolve as source;
use rustgm::{
    sema,
    sema::closures::{AccessKind, captures::Mode},
    types::closure::CallKind,
};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const PACKET: &str = "pa31/tests/spec/source-field-captures";

#[test]
fn actual_field_capture_sources_preserve_call_modes_moves_and_loans() -> Result {
    let mut failures = Vec::new();
    let mut count = 0;
    for manifest in ["cases.tsv", "prefix-cases.tsv"] {
        for line in source::fixture(&format!("{PACKET}/{manifest}"))?.lines() {
            let (name, expected) = line.split_once('\t').ok_or("capture case columns")?;
            let text = source::fixture(&format!("{PACKET}/{name}"))?;
            let result = source::source_provider(&text, "field_capture").and_then(|report| {
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
                        "missing",
                        "lost its actual",
                        "changed its actual",
                        "unfinished",
                        "another actual",
                        "unsupported",
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
    }
    assert_eq!(count, 25);
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}

#[test]
fn field_receiver_prefixes_keep_real_capture_types_and_sparse_layout() -> Result {
    for (name, kind, mode, depth) in [
        ("shared-fn", CallKind::Shared, Mode::Shared, 0),
        ("mutable-fnmut", CallKind::Mutable, Mode::Mutable, 0),
        ("move-read-fn", CallKind::Shared, Mode::Value, 0),
        ("reference-prefix", CallKind::Shared, Mode::Shared, 1),
        (
            "mutable-reference-prefix",
            CallKind::Mutable,
            Mode::Mutable,
            1,
        ),
        ("late-mut-method", CallKind::Mutable, Mode::Mutable, 0),
    ] {
        let text = source::fixture(&format!("{PACKET}/{name}.rs"))?;
        let report = source::source_provider(&text, "field_capture_types")?;
        let facts = sema::bodies::analyze(&report, sema::declarations::analyze(&report)?)?;
        let body = facts.closures.all().first().ok_or("actual closure body")?;
        let environment = facts
            .closures
            .environment(body.identity)
            .ok_or("capture environment")?;
        assert_eq!(
            facts.types.closure_kind(body.identity),
            Some(kind),
            "{name}"
        );
        assert_eq!(environment.plan.captures().len(), 1, "{name}");
        let capture = environment.plan.captures()[0];
        assert_eq!(capture.mode, mode, "{name}");
        // The mutable fixture also reads the written physical inner.leaf.value
        // path. Those three genuine projections coexist with the wrapper root.
        let paths = if name == "mutable-fnmut" {
            4
        } else {
            1 + depth as usize
        };
        assert_eq!(environment.plan.path_count(), paths, "{name}");
        let mut prefix = capture.path;
        for _ in 0..depth {
            let (parent, sema::closures::captures::Step::Dereference { .. }) = environment
                .plan
                .parent(prefix)
                .ok_or("physical reference prefix")?
            else {
                return Err(format!("{name}: invented target-field capture").into());
            };
            prefix = parent;
        }
        assert!(
            environment.plan.parent(prefix).is_none(),
            "{name}: wrapper prefix"
        );
        if depth == 0 {
            assert!(environment.adjusted_accesses.is_empty(), "{name}");
        } else {
            let rows: Vec<_> = environment
                .adjusted_accesses
                .iter()
                .filter(|((_, kind), _)| *kind == AccessKind::Field)
                .collect();
            assert!(!rows.is_empty(), "{name}");
            for (_, access) in rows {
                assert_eq!(access.dereferences, depth, "{name}");
                assert!(environment.plan.covering(access.path).is_some(), "{name}");
            }
        }
    }
    assert_eq!(size_of::<AccessKind>(), 1);
    assert_eq!(size_of::<sema::closures::AdjustedAccess>(), 8);
    assert_eq!(size_of::<sema::closures::Environment>(), 328);
    assert_eq!(size_of::<rustgm::body::ir::Body>(), 456);
    assert_eq!(size_of::<sema::model::Operation>(), 16);
    println!("capture-layout access=1 adjusted=8 environment=328 body=456 operation=16");
    Ok(())
}

#[test]
fn repeated_field_receivers_share_physical_capture_paths_with_bounded_work() -> Result {
    for width in [1usize, 8, 32] {
        let text = source::fixture(&format!("{PACKET}/scaling-{width}.rs"))?;
        let report = source::source_provider(&text, "capture_scaling")?;
        let rustgm::pipeline::Analysis {
            mut facts,
            prepared,
        } = rustgm::pipeline::analyze_source(&report)?;
        let body = facts.closures.all().first().ok_or("scaling closure")?;
        let environment = facts
            .closures
            .environment(body.identity)
            .ok_or("scaling capture")?;
        assert_eq!(environment.plan.path_count(), 2);
        assert_eq!(environment.plan.captures().len(), 1);
        assert_eq!(environment.adjusted_accesses.len(), width);
        assert!(
            environment
                .adjusted_accesses
                .keys()
                .all(|(_, kind)| *kind == AccessKind::Field)
        );
        assert!(environment.plan.work < 64 + 16 * width);
        let work = facts.body_work.inference_steps
            + facts.body_work.canonical_steps
            + facts.body_work.steps;
        assert!(work < 50_000 + 5_000 * width, "{width}: {work}");
        println!(
            "capture-width={width} paths=2 fields=1 adjustments={width} graph={} total={work}",
            environment.plan.work
        );
        rustgm::lower::instances::inspect_prepared(&report, &mut facts, prepared, |_, _| ())?;
    }
    assert_eq!(
        size_of::<(
            (rustgm::resolve::unit::NodeKey, AccessKind),
            sema::closures::AdjustedAccess
        )>(),
        20
    );
    Ok(())
}

#[test]
#[ignore = "requires the locked CPPGM object backend and a C compiler"]
fn actual_field_capture_and_call_time_effects_match_nine_native_observers() -> Result {
    let text = source::fixture(&format!("{PACKET}/native.rs"))?;
    let report = source::source_provider(&text, "field_capture_native")?;
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
    let expected = source::fixture(&format!("{PACKET}/native.expected"))?;
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let folder = root.join("obj/parallel/async-iterator-v1/capture-native");
    std::fs::create_dir_all(&folder)?;
    crate::ffi_program::check(
        &root,
        &folder,
        "capture",
        &program,
        &source::fixture(&format!("{PACKET}/observer.c"))?,
        &expected,
    )
}

#[test]
fn actual_foreign_imports_remain_external_in_opaque_aware_body_scheduling() -> Result {
    let text = source::fixture(&format!("{PACKET}/native.rs"))?;
    let report = source::source_provider(&text, "foreign_capture_body")?;
    let rustgm::pipeline::Analysis { facts, .. } = rustgm::pipeline::analyze_source(&report)?;
    let function = facts
        .functions
        .iter()
        .find(|function| {
            report
                .package
                .names
                .get(report.resolution.definitions[function.definition.0 as usize].name)
                == "record"
        })
        .ok_or("real foreign observer declaration")?;
    assert!(function.body.is_none());
    assert_eq!(
        facts
            .foreign
            .as_deref()
            .and_then(|rows| rows.function(function.definition))
            .map(|row| row.linkage),
        Some(rustgm::support::linkage::Linkage::Import)
    );
    assert!(!facts.opaques.owners.is_empty());
    Ok(())
}
