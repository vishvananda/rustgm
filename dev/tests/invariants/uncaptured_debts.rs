//! Specialization reproves the child without promoting its parent's conclusions.
use super::{Inputs, specialize};
use crate::{
    body::{self, ir::BodyKey, regions::Obligation},
    borrow,
    types::{RegionId, outlives::Queries},
};

#[test]
fn uncaptured_specialization_retains_every_source_lifetime_debt()
-> Result<(), Box<dyn std::error::Error>> {
    let report = crate::test_source::resolve(
        "fn main(){let f:for<'a> fn(&'a u8)->&'a u8=|x|x;let n=7;let _=f(&n);}",
        true,
    )?;
    let mut facts = crate::pipeline::analyze_source(&report)?.facts;
    let program = body::build_concrete(&report, &facts)?;
    let mut original = body::regions::collect(&facts)?
        .bodies
        .into_iter()
        .find(|input| matches!(input.key, BodyKey::Closure(_)))
        .ok_or("missing source closure")?;
    let cfg = program
        .bodies
        .iter()
        .find(|cfg| cfg.key() == original.key)
        .ok_or("missing closure CFG")?;
    assert!(original.universals > 0);
    // This explicitly typed body has no inferred output port. Add an adversarial
    // parent bridge: a parent's static image must not prove a child's universal
    // outlives static when specialization removes the conditional interface.
    let parent = program
        .bodies
        .iter()
        .find(|body| matches!(body.key(), BodyKey::Declaration(_)))
        .ok_or("missing enclosing body")?;
    original.output = Some(std::sync::Arc::new(crate::sema::nested_outputs::Link {
        source: original.source,
        parent: parent.source,
        revision: 1,
        universals: original.universals,
        bridges: Box::new([crate::sema::nested_outputs::Bridge {
            slot: 0,
            parent: RegionId::STATIC,
            output: true,
        }]),
        formation: None,
    }));
    for unpaid in [false, true] {
        let mut source = original.clone();
        if unpaid {
            source.obligations.push(Obligation {
                source: source.source,
                kind: body::regions::Kind::Outlives((RegionId::bound(0, 0)?, RegionId::STATIC)),
            });
        }
        let mut inputs = Inputs {
            caller: crate::sema::values::execution::CallerInstance::Original,
            instance: None,
            bindings: Vec::new(),
            captures: Vec::new(),
            comparison: None,
            connections: None,
        };
        let (_, contract) = specialize(
            &mut facts,
            &source,
            &mut inputs,
            super::super::Premises::Required,
        )?;
        assert!(contract.output.is_none());
        assert!(contract.require_result_formation);
        assert_eq!(contract.assumptions, source.assumptions);
        let mut queries = Queries::new(
            &facts.types,
            &facts.lifetimes.nominal_requirements,
            &facts.lifetimes.callables,
        );
        let proof = borrow::regions::check_one(
            &facts.types,
            &facts.lifetimes.variances,
            &mut queries,
            &contract,
            cfg,
        );
        if unpaid {
            assert!(
                proof.is_err(),
                "specialization discarded a source lifetime debt"
            );
        } else {
            proof?;
        }
    }
    Ok(())
}
