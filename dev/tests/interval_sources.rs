//! Actual source range declarations and construction retain ordinary type and ownership proof.
use crate::resolve as source;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const PACKET: &str = "pa31/tests/spec/source-ranges";

#[test]
fn source_range_fields_and_constructors_keep_real_generic_and_body_rules() -> Result {
    let mut failures = Vec::new();
    let mut count = 0;
    for row in source::fixture(&format!("{PACKET}/cases.tsv"))?.lines() {
        let (path, expected) = row.split_once('\t').ok_or("case columns")?;
        if path.ends_with("/old-inclusive-form.rs") {
            assert_eq!(expected, "0");
            continue;
        }
        let text = source::fixture(path)?;
        let result = source::source_provider(&text, "source_ranges").and_then(|report| {
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
    assert_eq!(count, 55);
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}

#[test]
fn original_constructor_and_fields_remain_exact_source_fragments() -> Result {
    let original = source::fixture(&format!("{PACKET}/original-ops-range.rs"))?;
    let constructor = source::fixture(&format!("{PACKET}/constructor.fragment.rs"))?;
    assert!(original.contains(constructor.trim_end()));
    for (name, original) in [
        ("RangeFrom", &original),
        ("RangeFull", &original),
        ("RangeInclusive", &original),
        ("Range", &original),
        ("RangeToInclusive", &original),
        ("RangeTo", &original),
    ] {
        let fragment = source::fixture(&format!("{PACKET}/{name}.fragment.rs"))?;
        let (_, body) = fragment.split_once('\n').ok_or("actual struct fragment")?;
        assert!(original.contains(body.trim_end()), "{name}");
    }
    Ok(())
}

#[test]
fn old_inclusive_syntax_retains_its_required_actual_constructor_call_gap() -> Result {
    let text = source::fixture(&format!("{PACKET}/old-inclusive-form.rs"))?;
    let report = source::source_provider(&text, "inclusive_call")?;
    let error = rustgm::pipeline::analyze_source(&report)
        .err()
        .ok_or("inclusive source calls still require their actual plan")?;
    assert!(
        error
            .to_string()
            .contains("source inclusive range constructor calls require their actual call plan"),
        "{error}"
    );
    Ok(())
}

#[test]
fn inferred_range_work_tracks_nested_inputs_without_catalog_scans() -> Result {
    use std::io::Write;
    let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../obj/parallel/range-source-contracts/scaling-candidate");
    std::fs::create_dir_all(&folder)?;
    let mut observations = std::fs::File::create(folder.join("work.tsv"))?;
    let mut previous = None;
    let mut previous_canonical = None;
    for depth in [8, 16, 32, 64] {
        let mut ordinary = None;
        for unrelated in [0, 128] {
            let text = source::fixture(&format!(
                "{PACKET}/scaling/flat-depth-{depth}-extra-{unrelated}.rs"
            ))?;
            let report = source::source_provider(&text, "range_scaling")?;
            let analysis = rustgm::pipeline::analyze_source(&report)?;
            let work = analysis.facts.body_work.inference_steps;
            let canonical = analysis.facts.body_work.canonical_steps;
            writeln!(observations, "{depth}\t{unrelated}\t{work}\t{canonical}")?;
            if unrelated == 0 {
                if let Some(prior) = previous {
                    assert!(work <= 2 * prior + 256, "depth{depth}: {prior}->{work}");
                }
                if let Some(prior) = previous_canonical {
                    assert!(
                        canonical <= 2 * prior + 256,
                        "canonical depth{depth}: {prior}->{canonical}"
                    );
                }
                previous = Some(work);
                previous_canonical = Some(canonical);
                ordinary = Some((work, canonical));
            } else {
                assert_eq!(
                    ordinary,
                    Some((work, canonical)),
                    "depth{depth} scanned unrelated declarations"
                );
            }
        }
    }
    Ok(())
}

#[test]
#[ignore = "requires the pinned CPPGM object backend and a C compiler"]
fn range_construction_executes_actual_fields_const_body_order_and_drop() -> Result {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let folder = root.join("obj/parallel/range-source-contracts/candidate-native");
    std::fs::create_dir_all(&folder)?;
    for (name, observer, expected) in [
        ("constructor-native", "constructor-observer.c", "23 47\n"),
        ("reordered-native", "reordered-observer.c", "23 47\n"),
        ("drop-native", "drop-observer.c", "23 47 47 23 \n"),
        ("modern-native", "modern-observer.c", "23 47 23 47 47 1\n"),
    ] {
        let text = source::fixture(&format!("{PACKET}/{name}.rs"))?;
        let report = source::source_provider(&text, "range_native")?;
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
            &source::fixture(&format!("{PACKET}/{observer}"))?,
            expected,
        )?;
    }
    Ok(())
}
