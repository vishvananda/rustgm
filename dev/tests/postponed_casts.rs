//! Later closure calls provide the real cast inputs before full preparation.
use crate::{native_references as native, resolve as source, semantic};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const PACKET: &str = "pa24/tests/spec/projection-casts";

#[test]
fn later_closure_inputs_preserve_declared_cast_and_trait_outcomes() -> Result {
    let mut failures = Vec::new();
    let mut count = 0;
    for row in source::fixture(&format!("{PACKET}/cases.tsv"))?.lines() {
        let (path, expected) = row.split_once('\t').ok_or("cast case columns")?;
        let text = source::fixture(path)?;
        let result = semantic::analyze(&text).and_then(|(mut report, mut facts)| {
            let entry =
                rustgm::sema::entry::select(&mut report, &facts).map_err(|e| e.to_string())?;
            rustgm::lower::instances::build(&report, &mut facts, entry).map_err(|e| e.to_string())
        });
        match (expected, result) {
            ("0", Ok(_)) => {}
            ("1", Err(error))
                if path.ends_with("/uncalled.rs")
                    && error.contains("associated type projection requires normalization") => {}
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
    assert_eq!(count, 16);
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}

#[test]
fn actual_source_index_companions_retain_full_typed_lowering() -> Result {
    for row in source::fixture("pa31/tests/spec/projection-casts/cases.tsv")?.lines() {
        let (path, expected) = row.split_once('\t').ok_or("source cast case columns")?;
        assert_eq!(expected, "0");
        let text = source::fixture(path)?;
        let report = source::source_provider(&text, "projection_casts")?;
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
fn deferred_cast_work_tracks_sites_and_ignores_unrelated_types() -> Result {
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
                        "inference width{width}: {work:?}"
                    );
                    assert!(
                        work.1 <= 4 * canonical + 256,
                        "canonical width{width}: {work:?}"
                    );
                }
                previous = Some(work);
                control = Some(work);
            } else {
                let (inference, canonical) = control.ok_or("missing width control")?;
                assert!(
                    work.0 <= inference,
                    "width{width} added inference work: {inference}->{work:?}"
                );
                assert_eq!(work.1, canonical, "width{width} added canonical work");
            }
        }
    }
    Ok(())
}

#[test]
#[ignore = "requires the qualified CPPGM backend"]
fn later_closure_casts_execute_real_scalar_and_function_item_conversions() -> Result {
    let mut inputs = Vec::new();
    for name in [
        "called",
        "typed-control",
        "generic-parent",
        "nested",
        "tuple-input",
        "early-return",
        "function-item",
    ] {
        inputs.push((name, source::fixture(&format!("{PACKET}/{name}.rs"))?));
    }
    let cases: Vec<_> = inputs
        .iter()
        .map(|(name, text)| (*name, text.as_str(), false))
        .collect();
    native::check_literal_cases("pa24-projection-casts/native", &cases)
}
