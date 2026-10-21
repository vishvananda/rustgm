//! A source condition is a caller debt, never permission from matching a type.
use super::*;
use crate::types::{RegionId, Store};
type TestResult = std::result::Result<(), Box<dyn std::error::Error>>;
#[test]
fn every_selected_caller_must_discharge_output_conditions() -> TestResult {
    let report = crate::test_source::resolve(
        "fn take<'a,T>(x:&'a T)->&'a T{let f=move||x;f()}fn separate<'a,'b>(x:&'a u8,y:&'b u8){}fn main(){let n=7u8;let _=take(&n);}",
        true,
    )?;
    let prepared = crate::pipeline::analyze_source(&report)?;
    let mut facts = prepared.facts;
    // The complete template and instance gate publishes the exact source summary.
    let mut report = report;
    let entry = crate::sema::entry::select(&mut report, &facts)?;
    crate::lower::instances::build_prepared(&report, &mut facts, entry, prepared.prepared)?;
    let identity = facts.closures.all().first().ok_or("closure")?.identity;
    let conditions = facts
        .closures
        .conditions(identity)
        .ok_or("source conditions")?;
    assert!(!conditions.is_empty());
    let mut contract = crate::body::regions::collect(&facts)?
        .bodies
        .into_iter()
        .find(|c| c.universals == 2 && c.output.is_none())
        .ok_or("caller")?;
    let a = RegionId::bound(0, 0)?;
    let b = RegionId::bound(0, 1)?;
    let mut store = Store::default();
    let left = super::reference(&mut store, a)?;
    let right = super::reference(&mut store, b)?;
    let formal = super::function(&mut store, &[left, right], 2)?;
    let actual = super::function(&mut store, &[left, right], 0)?;
    let mut execution = super::formal(formal);
    execution.universals = 2;
    let bindings = matching::execution_bindings(&store, execution, actual)?;
    require_conditions(&facts, identity, &bindings, contract.source, &mut contract)?;
    let check = |contract: Contract| {
        crate::borrow::regions::check(
            &facts.types,
            &facts.lifetimes.variances,
            &crate::body::regions::Contracts {
                callables: &facts.lifetimes.callables,
                nominal_requirements: &facts.lifetimes.nominal_requirements,
                bodies: vec![contract],
            },
        )
    };
    assert!(check(contract.clone()).is_err());
    contract.assumptions.extend([(a, b), (b, a)]);
    assert!(check(contract).is_ok());
    Ok(())
}
