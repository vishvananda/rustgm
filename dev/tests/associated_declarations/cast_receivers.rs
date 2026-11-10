//! Declared associated-result bounds participate in ordinary receiver selection.
use super::{native, prepared, semantic};
use crate::resolve as source;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const PACKET: &str = "pa20/tests/spec/projected-receivers";

fn inputs() -> Result<Vec<(String, String, bool)>> {
    let mut inputs = Vec::new();
    for line in source::fixture(&format!("{PACKET}/cases.tsv"))?.lines() {
        let (path, expected) = line.split_once('\t').ok_or("receiver outcome columns")?;
        let name = path
            .rsplit('/')
            .next()
            .and_then(|name| name.strip_suffix(".rs"))
            .ok_or("receiver case name")?;
        let expected = match expected {
            "0" => true,
            "1" => false,
            _ => return Err("invalid receiver outcome".into()),
        };
        inputs.push((name.to_owned(), source::fixture(path)?, expected));
    }
    assert_eq!(inputs.len(), 26);
    Ok(inputs)
}

#[test]
fn projected_receiver_contracts_reach_complete_lowir_with_their_original_debts() -> Result {
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
fn receiver_queries_scale_with_uses_without_scanning_unrelated_types() -> Result {
    let mut previous = None;
    for width in [8, 32, 128] {
        let mut ordinary = None;
        for noise in [0, 128] {
            let text = source::fixture(&format!("{PACKET}/width-{width}-noise-{noise}.rs"))?;
            let (_, facts) = semantic::analyze(&text)?;
            let work = [
                facts.body_work.inference_steps,
                facts.body_work.canonical_steps,
                facts.methods.selection_steps,
                facts.methods.selection_probes,
            ];
            println!("projected receiver width={width} noise={noise} work={work:?}");
            if noise == 0 {
                if let Some(before) = previous {
                    for (old, actual) in std::iter::zip(before, work) {
                        assert!(
                            actual <= 4 * old + 512,
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
fn projected_receivers_execute_selected_values_mutations_and_borrows() -> Result {
    let inputs: Vec<_> = inputs()?.into_iter().filter(|(_, _, pass)| *pass).collect();
    assert_eq!(inputs.len(), 18);
    let cases: Vec<_> = inputs
        .iter()
        .map(|(name, text, _)| (name.as_str(), text.as_str(), false))
        .collect();
    native::check_literal_cases("pa20-projected-receivers/native", &cases)
}
