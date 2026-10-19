//! Whole call clauses reach body inference and final region validation.
#[path = "../../harness/witness_cases.rs"]
mod cases;
use super::{Argument, BYTE, Declarations, Kind, RegionId, TestResult, tests};
use crate::{
    body::regions,
    resolve::expansion::Report,
    sema,
    types::parameters::clauses::{Clause, Quantified},
};
use std::sync::Arc;

fn contract(report: &Report, facts: &mut Declarations) -> TestResult {
    let owner = tests::definition(report, "need")?;
    let formal = facts
        .generics
        .parameters
        .get(&owner)
        .into_iter()
        .flatten()
        .find_map(|parameter| match parameter.argument {
            Argument::Type(ty) => Some(ty),
            _ => None,
        })
        .ok_or("missing call type parameter")?;
    let arguments = facts.types.intern_arguments(&[
        Argument::Lifetime(RegionId::bound(0, 0)?),
        Argument::Type(formal),
    ])?;
    let predicate = tests::predicate(facts, tests::definition(report, "A")?, BYTE, arguments)?;
    let clause = Clause::ForAll(facts.types.intern_quantified(Quantified {
        bound_regions: 1,
        predicate,
    })?);
    let source = facts
        .functions
        .iter()
        .find(|function| function.definition == owner)
        .and_then(|function| function.body)
        .ok_or("missing need body")?;
    facts.generics.predicates.entry(owner).or_default().insert(
        0,
        sema::generics::Predicate {
            source,
            value: clause,
        },
    );
    let clauses: Vec<_> = facts.generics.predicates[&owner]
        .iter()
        .map(|p| p.value)
        .collect();
    let environment = facts
        .generics
        .environments
        .get(&owner)
        .ok_or("missing need environment")?
        .as_ref()
        .clone()
        .with_clauses(&facts.types, &clauses)?;
    facts
        .generics
        .environments
        .insert(owner, Arc::new(environment));
    facts
        .generics
        .traits
        .as_mut()
        .ok_or("missing completed trait contracts")?
        .formation(&mut facts.types, clause)?;
    Ok(())
}

fn check(source: &str, expected: bool) -> TestResult<Declarations> {
    // Source admission remains gated. Install the qualified whole contract at
    // the declaration/body boundary; exercise actual call inference thereafter.
    let (report, mut facts) = tests::declarations(source)?;
    contract(&report, &mut facts)?;
    let facts = sema::bodies::analyze(&report, facts)?;
    assert!(!facts.quantified_proofs.is_empty());
    let contracts = regions::collect(&facts)?;
    let checked =
        crate::borrow::regions::check(&facts.types, &facts.lifetimes.variances, &contracts);
    assert_eq!(
        checked.is_ok(),
        expected,
        "{source}: {:?}",
        checked.as_ref().err()
    );
    if let Err(error) = checked {
        assert!(error.message.contains("does not outlive"), "{error}");
        assert!(error.source.is_some());
    }
    Ok(facts)
}

#[test]
fn inferred_call_inputs_pay_their_universal_lifetime_debts() {
    for case in cases::CASES {
        let result = check(&case.body_source(), case.accepted);
        assert!(result.is_ok(), "{}: {:?}", case.name, result.as_ref().err());
    }
}

#[test]
fn partial_call_inference_keeps_one_witness_until_other_arguments_settle() -> TestResult {
    for (bound, expected) in [("", true), ("where 'a:'b", false)] {
        let facts = check(
            &format!(
                "trait A<'a,T>{{}}trait C<T>{{}}impl C<(&'static u8,u16)>for u8{{}}impl<'a,'b,U>A<'a,(&'b u8,U)>for u8 {bound}{{}}fn need<T>() where u8:C<T>{{}}fn main(){{need();}}"
            ),
            expected,
        )?;
        assert_eq!(facts.quantified_proofs.len(), 1);
        assert_eq!(facts.quantified_proofs[0].evidence.proof.scope_count(), 1);
        assert_eq!(facts.body_work.trait_selections, 3);
    }
    Ok(())
}

#[test]
fn unused_private_witnesses_may_depend_on_universals() -> TestResult {
    let facts = check(
        "trait A<'a,T>{}impl<'a:'b,'b>A<'a,u16>for u8{}fn need<T>(){}fn main(){need();}",
        true,
    )?;
    let proof = &facts.quantified_proofs[0].evidence.proof;
    // One universal alias and two impl witnesses; no exported lifetime is needed.
    assert_eq!(proof.regions.len(), 3);
    Ok(())
}

#[test]
fn exported_callable_inputs_preserve_local_binders_and_captured_witnesses() -> TestResult {
    for (bound, expected) in [("", true), ("where 'a:'b", false)] {
        let facts = check(
            &format!(
                "trait A<'a,T>{{}}impl<'a,'b>A<'a,for<'c>fn(&'c u8,&'b u8)>for u8 {bound}{{}}fn need<T>(){{}}fn main(){{need();}}"
            ),
            expected,
        )?;
        let proof = &facts.quantified_proofs[0].evidence.proof;
        assert_eq!(proof.regions.len(), 4);
        let mut found = false;
        for node in facts.nodes.iter().flatten() {
            let Some(Kind::FunctionItem { arguments, .. }) =
                node.ty.and_then(|ty| facts.types.kind(ty))
            else {
                continue;
            };
            for argument in facts
                .types
                .arguments(arguments)
                .ok_or("missing callable item arguments")?
            {
                let Argument::Type(ty) = argument else {
                    continue;
                };
                let Some(Kind::Function {
                    parameters,
                    bound_regions: 1,
                    ..
                }) = facts.types.kind(ty)
                else {
                    continue;
                };
                let parameters = facts
                    .types
                    .list(parameters)
                    .ok_or("missing inferred callable parameters")?;
                assert_eq!(parameters.len(), 2);
                assert!(
                    matches!(facts.types.kind(parameters[0]), Some(Kind::Reference { region, .. }) if region == RegionId::bound(0, 0)?)
                );
                assert!(
                    matches!(facts.types.kind(parameters[1]), Some(Kind::Reference { region, .. }) if region == RegionId(proof.regions.end - 1))
                );
                found = true;
            }
        }
        assert!(found, "missing inferred callable item argument");
    }
    Ok(())
}
