//! Ready call equations precede projected common-type joins.
use super::{native, prepared, semantic};
use crate::resolve as source;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const PACKET: &str = "pa20/tests/spec/pending-joins";

fn inputs() -> Result<Vec<(String, String, bool)>> {
    let mut inputs = Vec::new();
    for line in source::fixture(&format!("{PACKET}/cases.tsv"))?.lines() {
        let (path, expected) = line.split_once('\t').ok_or("join outcome columns")?;
        let name = path
            .rsplit('/')
            .next()
            .and_then(|name| name.strip_suffix(".rs"))
            .ok_or("join case name")?;
        let expected = match expected {
            "0" => true,
            "1" => false,
            _ => return Err("invalid join outcome".into()),
        };
        inputs.push((name.to_owned(), source::fixture(path)?, expected));
    }
    assert_eq!(inputs.len(), 29);
    Ok(inputs)
}

#[test]
fn pending_joins_keep_complete_call_and_conversion_obligations() -> Result {
    let mut failures = Vec::new();
    for (name, text, accepted) in inputs()? {
        let result = prepared::checked(&text);
        match (accepted, result) {
            (true, Ok(_)) => {}
            (false, Err(error))
                if ![
                    "normalization",
                    "not implemented",
                    "invalid",
                    "budget",
                    "exhausted",
                ]
                .iter()
                .any(|part| error.contains(part)) => {}
            (_, result) => failures.push(format!("{name}: {:?}", result.err())),
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}

#[test]
fn join_readiness_scales_with_calls_and_ignores_unrelated_types() -> Result {
    let mut previous = None;
    for width in [8, 32, 128] {
        let mut ordinary = None;
        for noise in [0, 128] {
            let text = source::fixture(&format!("{PACKET}/width-{width}-noise-{noise}.rs"))?;
            let (_, facts) = semantic::analyze(&text)?;
            let work = [
                facts.body_work.inference_steps,
                facts.body_work.canonical_steps,
                facts.body_work.trait_selections,
                facts.methods.selection_steps,
                facts.methods.selection_probes,
            ];
            println!("pending join width={width} noise={noise} work={work:?}");
            if noise == 0 {
                if let Some(before) = previous {
                    for (old, actual) in std::iter::zip(before, work) {
                        assert!(
                            actual <= 4 * old + 1024,
                            "width={width}: {before:?}->{work:?}"
                        );
                    }
                }
                previous = Some(work);
                ordinary = Some(work);
            } else {
                assert_eq!(ordinary, Some(work), "width={width} unrelated type work");
            }
        }
    }
    Ok(())
}

#[test]
#[ignore = "requires the qualified CPPGM backend"]
fn pending_joins_execute_both_arms_and_all_conversion_values() -> Result {
    let inputs: Vec<_> = inputs()?.into_iter().filter(|(_, _, pass)| *pass).collect();
    assert_eq!(inputs.len(), 23);
    let cases: Vec<_> = inputs
        .iter()
        .map(|(name, text, _)| (name.as_str(), text.as_str(), false))
        .collect();
    native::check_literal_cases("pa20-pending-joins/native", &cases)
}
