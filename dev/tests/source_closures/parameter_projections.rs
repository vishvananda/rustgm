//! Parameter patterns inspect projections registered in their own child body.
use crate::{native_references as native, resolve as source, semantic};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const PACKET: &str = "pa24/tests/spec/parameter-projections";

#[test]
fn try_blocks_in_capturing_closures_keep_actual_move_and_target_rules() -> Result {
    let mut failures = Vec::new();
    for (name, cause) in [
        ("literal", None),
        ("argument", None),
        ("copied-capture", None),
        ("owned-capture", None),
        ("mutable-capture", None),
        ("question", None),
        ("filter-helper", None),
        ("nested", None),
        ("wildcard-owned", None),
        ("wrong-output", Some("incompatible types")),
        ("missing-gate", Some("try_blocks")),
        (
            "closure-boundary",
            Some("trait obligation requires an unambiguous matching assumption or implementation"),
        ),
        ("repeat-once", Some("uninitialized or moved")),
        ("repeat-wildcard", Some("uninitialized or moved")),
    ] {
        let text = source::fixture(&format!("pa31/tests/spec/try-captures/{name}.rs"))?;
        let checked = (|| -> Result {
            let report = source::source_provider(&text, "try_capture_uses")?;
            let rustgm::pipeline::Analysis {
                mut facts,
                prepared,
            } = rustgm::pipeline::analyze_source(&report)?;
            let kind = match name {
                "copied-capture" => Some(rustgm::types::closure::CallKind::Shared),
                "mutable-capture" => Some(rustgm::types::closure::CallKind::Mutable),
                "owned-capture" | "wildcard-owned" => Some(rustgm::types::closure::CallKind::Once),
                _ => None,
            };
            if let Some(kind) = kind {
                let bodies = facts.closures.all();
                assert_eq!(bodies.len(), 1);
                let body = &bodies[0];
                assert_eq!(body.execution.ok_or("closure receiver")?.receiver, kind);
                assert_eq!(
                    facts
                        .closures
                        .environment(body.identity)
                        .ok_or("capture environment")?
                        .field_types
                        .len(),
                    1
                );
            }
            rustgm::lower::instances::build_roots_prepared(
                &report,
                &mut facts,
                rustgm::lower::Roots {
                    entry: None,
                    exports: &[],
                },
                prepared,
            )?;
            Ok(())
        })();
        match (cause, checked) {
            (None, Ok(())) => {}
            (Some(cause), Err(error)) if error.to_string().contains(cause) => {}
            (cause, outcome) => failures.push(format!(
                "{name}: expected {cause:?}, got {:?}",
                outcome.err().map(|error| error.to_string())
            )),
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}

#[test]
fn try_capture_work_scales_with_block_sites_and_reuses_one_field() -> Result {
    let mut previous = None;
    for width in [8, 32, 128] {
        let mut baseline = None;
        for noise in [0, 128] {
            let text = source::fixture(&format!(
                "pa31/tests/spec/try-captures/width-{width}-noise-{noise}.rs"
            ))?;
            let report = source::source_provider(&text, "try_capture_scaling")?;
            let rustgm::pipeline::Analysis {
                mut facts,
                prepared,
            } = rustgm::pipeline::analyze_source(&report)?;
            assert_eq!(facts.closures.all().len(), 1);
            let body = &facts.closures.all()[0];
            assert_eq!(
                facts
                    .closures
                    .environment(body.identity)
                    .ok_or("capture environment")?
                    .field_types
                    .len(),
                1
            );
            let work = [
                facts.body_work.inference_steps,
                facts.body_work.trait_selections,
                facts.body_work.canonical_steps,
            ];
            rustgm::lower::instances::build_roots_prepared(
                &report,
                &mut facts,
                rustgm::lower::Roots {
                    entry: None,
                    exports: &[],
                },
                prepared,
            )?;
            println!("try captures width={width} noise={noise} work={work:?} fields=1");
            if noise == 0 {
                if let Some(old) = previous {
                    for (current, before) in work.into_iter().zip(old) {
                        assert!(current <= 4 * before + 256);
                    }
                }
                previous = Some(work);
                baseline = Some(work);
            } else {
                assert_eq!(baseline, Some(work));
            }
        }
    }
    Ok(())
}

#[test]
fn imported_inputs_normalize_before_structural_patterns_and_annotations() -> Result {
    let mut failures = Vec::new();
    let mut count = 0;
    for row in source::fixture(&format!("{PACKET}/cases.tsv"))?.lines() {
        let (path, expected) = row.split_once('\t').ok_or("parameter case columns")?;
        let text = source::fixture(path)?;
        let result = semantic::analyze(&text).and_then(|(mut report, mut facts)| {
            let entry =
                rustgm::sema::entry::select(&mut report, &facts).map_err(|e| e.to_string())?;
            rustgm::lower::instances::build(&report, &mut facts, entry).map_err(|e| e.to_string())
        });
        match (expected, result) {
            ("0", Ok(_)) => {}
            ("1", Err(error))
                if ![
                    "not implemented",
                    "normalization",
                    "exhausted",
                    "budget",
                    "lost its",
                ]
                .iter()
                .any(|message| error.contains(message)) => {}
            (_, result) => failures.push(format!(
                "{path}: {}",
                result.err().unwrap_or_else(|| "accepted".into())
            )),
        }
        count += 1;
    }
    assert_eq!(count, 18);
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}

#[test]
fn genuine_source_iterator_callbacks_retain_complete_body_proof() -> Result {
    for row in source::fixture("pa31/tests/spec/parameter-projections/cases.tsv")?.lines() {
        let (path, expected) = row.split_once('\t').ok_or("source parameter columns")?;
        assert_eq!(expected, "0");
        let text = source::fixture(path)?;
        let report = source::source_provider(&text, "parameter_projections")?;
        let rustgm::pipeline::Analysis {
            mut facts,
            prepared,
        } = rustgm::pipeline::analyze_source(&report).map_err(|e| format!("{path}: {e}"))?;
        rustgm::lower::instances::inspect_prepared(&report, &mut facts, prepared, |_, _| ())
            .map_err(|e| format!("{path}: {e}"))?;
    }
    Ok(())
}

#[test]
fn parameter_projection_work_tracks_sites_without_unrelated_declaration_work() -> Result {
    let mut previous = None;
    for width in [8, 32, 128] {
        let mut control = None;
        for noise in [0, 128] {
            let text = source::fixture(&format!("{PACKET}/width-{width}-noise-{noise}.rs"))?;
            let (_, facts) = semantic::analyze(&text)?;
            let work = (
                facts.body_work.inference_steps,
                facts.body_work.canonical_steps,
            );
            if noise == 0 {
                if let Some((inference, canonical)) = previous {
                    assert!(
                        work.0 <= 4 * inference + 256,
                        "width{width} inference: {work:?}"
                    );
                    assert!(
                        work.1 <= 4 * canonical + 256,
                        "width{width} canonical: {work:?}"
                    );
                }
                previous = Some(work);
                control = Some(work);
            } else {
                let (inference, canonical) = control.ok_or("missing width control")?;
                assert!(
                    work.0 <= inference,
                    "width{width} added inference: {inference}->{work:?}"
                );
                assert_eq!(work.1, canonical, "width{width} added canonical work");
            }
        }
    }
    Ok(())
}

#[test]
#[ignore = "requires the qualified CPPGM backend"]
fn projected_parameter_patterns_execute_real_callback_results() -> Result {
    let mut inputs = Vec::new();
    for row in source::fixture(&format!("{PACKET}/cases.tsv"))?.lines() {
        let (path, expected) = row.split_once('\t').ok_or("native parameter columns")?;
        if expected == "0" {
            let name = path
                .rsplit('/')
                .next()
                .and_then(|name| name.strip_suffix(".rs"))
                .ok_or("native parameter name")?
                .to_owned();
            inputs.push((name, source::fixture(path)?));
        }
    }
    assert_eq!(inputs.len(), 15);
    let cases: Vec<_> = inputs
        .iter()
        .map(|(name, text)| (name.as_str(), text.as_str(), false))
        .collect();
    native::check_literal_cases("pa24-parameter-projections/native", &cases)
}
