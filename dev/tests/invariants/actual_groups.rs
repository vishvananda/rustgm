//! Actual full-proof rows, including an ancestor, cannot be replaced by matching headers.
use super::*;
use crate::support::hash::HashMap;
use crate::{body, borrow, pipeline, test_source};
use std::sync::Arc;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
fn actual(damage: bool) -> Result<(Vec<PairedProved>, bool)> {
    let report = test_source::resolve("fn a()->u8{7}fn b()->u8{let x:u8=9;x}", true)?;
    let mut facts = pipeline::analyze_source(&report)?.facts;
    let mut program = body::build_concrete(&report, &facts)?;
    body::promote::analyze(&report, &mut facts, &mut program)?;
    let inputs: HashMap<_, _> = body::regions::collect(&facts)?
        .bodies
        .into_iter()
        .map(|input| (input.key, Arc::new(input)))
        .collect();
    for input in inputs.values() {
        input.prepare_outlives(&mut facts.types, &mut facts.lifetimes.nominal_requirements)?;
    }
    if damage {
        let last = program.bodies.last_mut().ok_or("last body")?;
        let scalar = last
            .values
            .iter_mut()
            .find(|v| matches!(v.kind, body::ir::Rvalue::Scalar(_)))
            .ok_or("ancestor scalar")?;
        scalar.kind = body::ir::Rvalue::Read(body::ir::Place::Local(body::ir::LocalId(0)));
    }
    let contracts = body::regions::Contracts {
        bodies: Vec::new(),
        nominal_requirements: &facts.lifetimes.nominal_requirements,
        callables: &facts.lifetimes.callables,
    };
    let paired = borrow::regions::check_retained_group(
        &facts.types,
        &facts.lifetimes.variances,
        &contracts,
        program
            .bodies
            .iter()
            .map(|body| Ok((Arc::clone(inputs.get(&body.key()).ok_or("input")?), body)))
            .collect::<Result<Vec<_>>>()?,
    )?;
    let originals = body::templates::Templates { bodies: Vec::new() };
    let mut copies = crate::types::copy::Copies::default();
    let mut environment = Environment {
        types: &facts.types,
        definitions: &facts.aggregates,
        derives: &facts.derives,
        copies: &mut copies,
    };
    let checked = templates::verify(
        &mut environment,
        &facts.lifetimes.variances,
        &contracts,
        &originals,
        &[],
        |_, _| Err("unexpected field"),
    )?;
    let mut verifier = Verifier::new(None)?;
    let mut rows = Vec::new();
    let mut failed = false;
    for (proof, body) in paired.into_iter().zip(program.bodies) {
        if let Ok(row) = verifier.prove_paired(&mut environment, &checked, proof, body, None) {
            rows.push(row);
        } else {
            failed = true;
            break;
        }
    }
    Ok((rows, failed))
}
#[test]
fn groups_reject_missing_duplicate_foreign_stale_and_reordered_proofs() -> Result {
    let (mut rows, failed) = actual(false)?;
    assert!(!failed);
    let (foreign, _) = actual(false)?;
    assert_eq!(rows.len(), 2);
    let mut work = 0;
    assert!(CompleteGroup::new(rows.iter(), 64 * 1024 * 1024, &mut work).is_ok());
    assert_eq!(work, 2);
    assert!(CompleteGroup::new(rows[..1].iter(), 64 * 1024 * 1024, &mut work).is_err());
    assert!(
        CompleteGroup::new(
            [&rows[0], &rows[0]].into_iter(),
            64 * 1024 * 1024,
            &mut work
        )
        .is_err()
    );
    assert!(
        CompleteGroup::new(
            [&rows[0], &foreign[1]].into_iter(),
            64 * 1024 * 1024,
            &mut work
        )
        .is_err()
    );
    assert!(CompleteGroup::new(rows.iter().rev(), 64 * 1024 * 1024, &mut work).is_err());
    assert!(CompleteGroup::new(rows.iter(), 0, &mut work).is_err());
    rows[0].proved.body.source = rows[1].body().source;
    assert!(CompleteGroup::new(rows.iter(), 64 * 1024 * 1024, &mut work).is_err());
    Ok(())
}
#[test]
fn failed_actual_ancestor_cannot_complete_or_activate_a_group() -> Result {
    let (rows, failed) = actual(true)?;
    assert!(
        failed,
        "the actual uninitialized ancestor must fail the full verifier"
    );
    assert_eq!(rows.len(), 1);
    assert!(CompleteGroup::new(rows.iter(), 64 * 1024 * 1024, &mut 0).is_err());
    // No CompleteGroup means no ParentMember or scope activation can be obtained.
    Ok(())
}
