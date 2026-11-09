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

/// The checked rustc outcomes of `outcomes.json`. The probes, which need
/// the other header's trait premises, are left out.
const CASES: [(&str, &str, Option<&str>); 59] = [
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
    (PACKET, "nested-premise-side", Some(OVERLAP)),
    (PACKET, "nested-premise-side-swapped", Some(OVERLAP)),
    (PACKET, "nested-client-side", None),
    (PACKET, "nested-client-side-swapped", None),
    (PACKET, "nested-copy-client-side", None),
    (PACKET, "nested-copy-client-side-swapped", None),
    (PACKET, "nested-written-premise", None),
    (PACKET, "nested-written-premise-swapped", None),
    (PACKET, "nested-deep", Some(OVERLAP)),
    (PACKET, "nested-deep-swapped", Some(OVERLAP)),
    (PACKET, "nested-tuple-tail", Some(OVERLAP)),
    (PACKET, "nested-tuple-tail-swapped", Some(OVERLAP)),
    (PACKET, "nested-tuple-head", None),
    (PACKET, "nested-tuple-head-swapped", None),
    (PACKET, "nested-array", None),
    (PACKET, "nested-array-swapped", None),
    (PACKET, "nested-metasized", Some(OVERLAP)),
    (PACKET, "nested-metasized-swapped", Some(OVERLAP)),
    (PACKET, "nested-metasized-held", None),
    (PACKET, "nested-metasized-held-swapped", None),
    (PACKET, "nested-wf-control", None),
    (PACKET, "nested-wf-control-swapped", None),
    (PACKET, "nested-projected-size", None),
    (PACKET, "nested-projected-size-swapped", None),
    (PACKET, "nested-projected-premise-side", Some(OVERLAP)),
    (
        PACKET,
        "nested-projected-premise-side-swapped",
        Some(OVERLAP),
    ),
    (PACKET, "nested-projected-sized-impl", None),
    (PACKET, "nested-projected-sized-impl-swapped", None),
    (PACKET, "nested-projected-composite", None),
    (PACKET, "nested-projected-composite-swapped", None),
    (PACKET, "nested-projected-unsized", Some(OVERLAP)),
    (PACKET, "nested-projected-unsized-swapped", Some(OVERLAP)),
    (PACKET, "nested-projected-written", None),
    (PACKET, "nested-projected-written-swapped", None),
    (PACKET, "nested-normalized-builtin", None),
    (PACKET, "nested-normalized-builtin-swapped", None),
    (PACKET, "nested-projected-metasized", None),
    (PACKET, "nested-projected-metasized-swapped", None),
];

/// A case's diagnostic, or for an accepted case whether its `Holds` pair may
/// overlap with each header first.
type Observation = (Option<String>, Option<[bool; 2]>);
/// An accepted case's facts and its one pair of `Holds` impl headers, or the
/// case's diagnostic.
type Analyzed = Result<(crate::sema::model::Declarations, [(Predicate, DefId); 2]), String>;

fn analyze(directory: &str, name: &str) -> TestResult<Analyzed> {
    let source = crate::test_source::fixture(&format!("{directory}/{name}.rs"))?;
    let report = crate::test_source::source_provider(&source, "negative_premise_sides")?;
    let facts = match crate::pipeline::analyze_source(&report) {
        Ok(analysis) => analysis.facts,
        Err(error) => return Ok(Err(error.message.clone())),
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
    let pair = <[_; 2]>::try_from(headers).map_err(|_| "one pair of Holds impls")?;
    Ok(Ok((facts, pair)))
}

fn observe(directory: &str, name: &str) -> TestResult<Observation> {
    let (mut facts, [left, right]) = match analyze(directory, name)? {
        Ok(analyzed) => analyzed,
        Err(message) => return Ok((Some(message), None)),
    };
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
            assert!(
                !forward && !backward,
                "{name}: disjoint in one header order only: {forward} {backward}"
            );
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
        assert_eq!(outcome, partner, "{name} differs from {base}");
        pairs += 1;
    }
    assert_eq!(pairs, 23, "every swapped twin is compared");
    Ok(())
}

/// A negative coherence query under a non-empty premise set never reaches the
/// crate-wide memo of closed body goals in neutral callers.
#[test]
fn negative_queries_leave_the_body_selection_memo_unchanged() -> TestResult {
    let (mut facts, [left, right]) = analyze(PACKET, "nested-client-side")?
        .map_err(|error| format!("nested-client-side: {error}"))?;
    let entries = facts.generics.selection_memo.entries();
    let [queries, premised] = super::negative_queries();
    let forward = super::super::possible(&mut facts, left, right)?.0;
    let backward = super::super::possible(&mut facts, right, left)?.0;
    assert!(!forward && !backward, "the pair is disjoint in both orders");
    let [queries_after, premised_after] = super::negative_queries();
    assert!(queries_after > queries, "a negative query ran");
    assert!(
        premised_after > premised,
        "a negative query ran under a non-empty premise set"
    );
    assert_eq!(facts.generics.selection_memo.entries(), entries);
    Ok(())
}
