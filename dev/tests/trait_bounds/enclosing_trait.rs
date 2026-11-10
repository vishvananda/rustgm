//! The implemented trait supplies lexical eligibility without changing proof priority.
use super::{native, semantic};
use crate::{foreign_frontend, resolve as source};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const PACKET: &str = "pa19/tests/spec/impl-scope";

fn inputs() -> Result<Vec<(String, String, bool)>> {
    let mut rows = Vec::new();
    for line in source::fixture(&format!("{PACKET}/cases.tsv"))?.lines() {
        let (path, status) = line.split_once('\t').ok_or("impl scope columns")?;
        let name = path
            .rsplit('/')
            .next()
            .and_then(|name| name.strip_suffix(".rs"))
            .ok_or("impl scope case name")?;
        let accepted = match status {
            "0" => true,
            "1" => false,
            _ => return Err("invalid impl scope outcome".into()),
        };
        rows.push((name.to_owned(), source::fixture(path)?, accepted));
    }
    assert_eq!(rows.len(), 34);
    Ok(rows)
}

#[test]
fn implementation_traits_are_lexical_and_keep_complete_call_proofs() -> Result {
    let mut failures = Vec::new();
    for (name, text, accepted) in inputs()? {
        let result = foreign_frontend::checked(&text);
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
fn implementation_visibility_shares_bounded_scope_work() -> Result {
    let mut previous = None;
    let mut noise_visibility = None;
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
                facts.methods.trait_visibility_steps,
            ];
            println!("impl scope width={width} noise={noise} work={work:?}");
            if noise == 0 {
                if let Some(before) = previous {
                    for (old, actual) in std::iter::zip(before, work) {
                        assert!(actual <= 4 * old + 1024, "{before:?}->{work:?}");
                    }
                }
                previous = Some(work);
                ordinary = Some(work);
            } else {
                let before = ordinary.ok_or("missing ordinary scope work")?;
                assert_eq!(&before[..5], &work[..5], "unrelated type call work");
                let added = work[5]
                    .checked_sub(before[5])
                    .ok_or("lost visibility work")?;
                // Each unit struct introduces one scope and type/value bindings.
                assert!(added <= 3 * noise, "unrelated visibility work");
                if let Some(previous) = noise_visibility {
                    assert_eq!(added, previous, "metadata work depends on call width");
                }
                noise_visibility = Some(added);
            }
        }
    }
    Ok(())
}

#[test]
#[ignore = "requires the qualified CPPGM backend"]
fn implemented_trait_calls_execute_the_selected_body() -> Result {
    let rows: Vec<_> = inputs()?.into_iter().filter(|(_, _, pass)| *pass).collect();
    assert_eq!(rows.len(), 25);
    let cases: Vec<_> = rows
        .iter()
        .map(|(name, text, _)| (name.as_str(), text.as_str(), false))
        .collect();
    native::check_literal_cases("pa19-impl-scope/native", &cases)
}
