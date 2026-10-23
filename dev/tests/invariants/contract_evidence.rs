//! Cached quantified result rules still require a selected parent and own each use's IDs.
use super::{Decision, Frame, Search, Source, quantified_tests::valid, tests};
use crate::{
    sema::traits::{assumptions::Assumptions, implementations::Availability, results},
    types::{GenericArgs, RegionId, arguments::Argument},
};
use std::sync::Arc;

#[test]
fn quantified_result_evidence_keeps_its_parent_and_does_not_escape_to_another_caller()
-> Result<(), Box<dyn std::error::Error>> {
    let (report, mut facts) = tests::declarations(
        "trait A<'a>{}trait Family{type Item:for<'a>A<'a>;}fn caller<T:Family>(){}",
    )?;
    let owner = tests::definition(&report, "caller")?;
    let environment = Arc::new(facts.generics.parameter_environment(&facts.types, owner)?);
    let Argument::Type(subject) = facts.generics.parameters[&owner][0].argument else {
        return Err("missing caller parameter".into());
    };
    let parent = tests::predicate(
        &mut facts,
        tests::definition(&report, "Family")?,
        subject,
        GenericArgs::EMPTY,
    )?;
    let template = facts
        .associated
        .type_contract(tests::definition(&report, "Item")?)
        .ok_or("missing result contract")?
        .projection;
    let projected = results::substitution(&facts, parent)?.apply(&mut facts.types, template)?;
    let mut assumptions = Assumptions::default();
    assumptions.insert(&facts.types, parent)?;
    let arguments = facts
        .types
        .intern_arguments(&[Argument::Lifetime(RegionId::STATIC)])?;
    let predicate = tests::predicate(
        &mut facts,
        tests::definition(&report, "A")?,
        projected,
        arguments,
    )?;
    for first in [100, 200] {
        let (mut context, goal) = tests::instantiate(&mut facts, predicate)?;
        let mut caller = tests::caller(&assumptions, Some(&environment));
        caller.first_region = first;
        let mut search = Search::new(caller, Availability::Validated);
        let frame = Frame::new(&facts, &mut context, &goal, first)?;
        let Decision::Proven(root) = search.run(&mut facts, frame)? else {
            return Err("missing quantified result evidence".into());
        };
        let mut work = vec![root];
        let mut found_parent = false;
        while let Some(index) = work.pop() {
            let node = &search.query.nodes[index];
            found_parent |=
                matches!(node.source, Some(Source::Assumption(value)) if value == parent);
            work.extend(&node.children);
        }
        assert!(
            found_parent,
            "result proof discarded its parent application"
        );
        let evidence = search.evidence(&facts, root)?;
        assert!(
            matches!(evidence.source, Source::QuantifiedResultBound { subject, .. } if subject == projected)
        );
        assert_eq!(evidence.proof.regions, first..first + 1);
        assert!(!evidence.proof.equalities.is_empty());
        assert!(valid(&facts, &evidence.proof, 0)?);
    }
    // The previous queries populated the mapped rule cache. It is not permission.
    let foreign = Assumptions::default();
    let (mut context, goal) = tests::instantiate(&mut facts, predicate)?;
    let mut caller = tests::caller(&foreign, None);
    caller.first_region = 2;
    let mut search = Search::new(caller, Availability::Validated);
    let frame = Frame::new(&facts, &mut context, &goal, 2)?;
    assert!(matches!(search.run(&mut facts, frame)?, Decision::Absent));
    Ok(())
}
