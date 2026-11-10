//! Parameter patterns inspect projections registered in their own child body.
use crate::{native_references as native, resolve as source, semantic};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const PACKET: &str = "pa24/tests/spec/parameter-projections";

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
