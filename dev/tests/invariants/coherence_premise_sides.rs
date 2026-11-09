//! rustc 1.98 negates each impl's predicates under the other impl's
//! `param_env`, in both orders, after plugging the unified header variables
//! with placeholders. So a negated clause's own header pays none of its debts:
//! a rigid parameter is Sized or `MetaSized` only by the other header's
//! premises, and a lifetime obligation holds only by that header's written
//! type-outlives premises, with no WF types and no region-region bound. Its
//! verdict cannot depend on which header comes first.
use crate::{resolve::model::DefId, types::parameters::predicates::Predicate};
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

const PACKET: &str = "pa31/tests/spec/negative-premise-sides";
const SIDES: &str = "pa31/tests/spec/negative-metasized";
const OVERLAP: &str = "overlapping trait implementation headers require coherence proof";

/// The checked rustc outcomes of `outcomes.json`, except the recorded
/// residual, `nested-premise-side`, which rustc rejects with E0119.
const CASES: [(&str, &str, Option<&str>); 23] = [
    (SIDES, "sized-premise-side", Some(OVERLAP)),
    (PACKET, "sized-premise-side-swapped", Some(OVERLAP)),
    (PACKET, "sized-client-side", None),
    (PACKET, "sized-client-side-swapped", None),
    (PACKET, "copy-premise-side", Some(OVERLAP)),
    (PACKET, "copy-client-side", None),
    (SIDES, "premise-side", Some(OVERLAP)),
    (PACKET, "premise-side-swapped", Some(OVERLAP)),
    (PACKET, "metasized-sized-representative", None),
    (PACKET, "metasized-sized-representative-swapped", None),
    (PACKET, "region-client-side", Some(OVERLAP)),
    (PACKET, "region-premise-side", Some(OVERLAP)),
    (PACKET, "region-absent", Some(OVERLAP)),
    (PACKET, "region-head-unified", Some(OVERLAP)),
    (PACKET, "region-implied", Some(OVERLAP)),
    (PACKET, "region-strict-control", Some(OVERLAP)),
    (PACKET, "strict-unconditional", None),
    (PACKET, "strict-static-unheld", Some(OVERLAP)),
    (PACKET, "type-outlives-held", None),
    (PACKET, "type-outlives", Some(OVERLAP)),
    (PACKET, "type-outlives-premise-side", Some(OVERLAP)),
    (PACKET, "nested-premise-side", None),
    (PACKET, "nested-premise-side-swapped", Some(OVERLAP)),
];

/// The recorded residual: `Wrap<V>: Sized` is still decided by the shape
/// walk, which reads the declaration of the representative that `freeze`
/// chose. Its two orders must still differ, so a fix must update the record.
const RESIDUAL: &str = "nested-premise-side";

/// A case's diagnostic, or for an accepted case whether its `Holds` pair may
/// overlap with each header first.
type Observation = (Option<String>, Option<[bool; 2]>);

fn observe(directory: &str, name: &str) -> TestResult<Observation> {
    let source = crate::test_source::fixture(&format!("{directory}/{name}.rs"))?;
    let report = crate::test_source::source_provider(&source, "negative_premise_sides")?;
    let mut facts = match crate::pipeline::analyze_source(&report) {
        Ok(analysis) => analysis.facts,
        Err(error) => return Ok((Some(error.message.clone()), None)),
    };
    let holds = report
        .resolution
        .definitions
        .iter()
        .position(|definition| report.package.names.get(definition.name) == "Holds")
        .map(u32::try_from)
        .transpose()?
        .ok_or("Holds")?;
    let mut headers: Vec<(Predicate, DefId)> = Vec::new();
    for header in facts
        .generics
        .implementations
        .as_ref()
        .ok_or("registry")?
        .headers()
    {
        let Some((application, _)) = header.predicate.bound.trait_parts() else {
            continue;
        };
        let item = facts
            .types
            .trait_application(application)
            .ok_or("trait application")?
            .item;
        if item.0 == holds {
            headers.push((header.predicate, header.owner));
        }
    }
    let [left, right] = <[_; 2]>::try_from(headers).map_err(|_| "one pair of Holds impls")?;
    let forward = super::super::possible(&mut facts, left, right)?.0;
    let backward = super::super::possible(&mut facts, right, left)?.0;
    Ok((None, Some([forward, backward])))
}

#[test]
fn negative_conditions_assume_only_the_other_headers_premises_in_either_order() -> TestResult {
    let mut outcomes = Vec::with_capacity(CASES.len());
    for (directory, name, expected) in CASES {
        let (outcome, orders) = observe(directory, name)?;
        match (expected, outcome.as_deref()) {
            (None, None) => {}
            (Some(message), Some(error)) => assert_eq!(error, message, "{name}"),
            (None, Some(error)) => return Err(format!("{name}: rejected: {error}").into()),
            (Some(message), None) => return Err(format!("{name}: accepted, not {message}").into()),
        }
        if let Some([forward, backward]) = orders {
            if name == RESIDUAL {
                assert!(
                    !forward && backward,
                    "{name}: the recorded residual changed: {forward} {backward}"
                );
            } else {
                assert!(
                    !forward && !backward,
                    "{name}: disjoint in one header order only: {forward} {backward}"
                );
            }
        }
        outcomes.push((name, outcome));
    }
    let mut pairs = 0_usize;
    for (name, outcome) in &outcomes {
        let Some(base) = name.strip_suffix("-swapped") else {
            continue;
        };
        let (_, partner) = outcomes
            .iter()
            .find(|(other, _)| *other == base)
            .ok_or_else(|| format!("{name} has no unswapped twin"))?;
        if base == RESIDUAL {
            assert_ne!(outcome, partner, "{name}: the recorded residual changed");
        } else {
            assert_eq!(outcome, partner, "{name} differs from {base}");
        }
        pairs += 1;
    }
    assert_eq!(pairs, 5, "every swapped twin is compared");
    Ok(())
}
