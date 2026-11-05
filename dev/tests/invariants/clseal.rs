//! Real prepared source carriers cannot cross arenas, bodies, verifiers or Copy scopes.
use super::*;
use crate::{body, borrow, sema::model::Declarations, types};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
fn fixture() -> Result<(Declarations, body::templates::Template, regions::Body)> {
    let text = crate::test_source::fixture("pa31/tests/spec/source-const-drop/initc.rs")?;
    let report = crate::test_source::source_provider(&text, "cleanup_seal")?;
    let crate::pipeline::Analysis {
        mut facts,
        mut prepared,
    } = crate::pipeline::analyze_source(&report)?;
    let function = facts
        .functions
        .iter()
        .find(|function| {
            report
                .package
                .names
                .get(report.resolution.definitions[function.definition.0 as usize].name)
                == "probe"
        })
        .copied()
        .ok_or("probe")?;
    let template = prepared.templates.function(&report, &facts, &function)?;
    let mut input = body::regions::Collector::default()
        .function(&facts, &function)?
        .clone();
    body::copy_requirements::prepare(&mut facts, &template.body, &mut input)?;
    input.prepare_outlives(&mut facts.types, &mut facts.lifetimes.nominal_requirements)?;
    let mut queries = types::outlives::Queries::new(
        &facts.types,
        &facts.lifetimes.nominal_requirements,
        &facts.lifetimes.callables,
    );
    let proof = regions::check_one(
        &facts.types,
        &facts.lifetimes.variances,
        &mut queries,
        &input,
        &template.body,
    )?;
    facts.types.arena_identity();
    facts.aggregates.arena_identity();
    Ok((facts, template, proof))
}
#[test]
fn authentic_source_seal_rejects_cloned_body_verifier_and_copy_environment() -> Result {
    let (facts, template, mut proof) = fixture()?;
    let mut copies = types::copy::Copies::default();
    let environment = borrow::Environment {
        types: &facts.types,
        definitions: &facts.aggregates,
        derives: &facts.derives,
        source_copy: true,
        copies: &mut copies,
    };
    let owner = Arc::new(());
    let seal = Seal::new(
        &environment,
        &template.body,
        &proof,
        Some(Arc::clone(&owner)),
    );
    seal.validate(&environment, &template.body, &proof, Some(&owner))?;
    assert!(
        seal.validate(&environment, &template.body.clone(), &proof, Some(&owner))
            .is_err()
    );
    assert!(
        seal.validate(&environment, &template.body, &proof, Some(&Arc::new(())))
            .is_err()
    );
    let builtin = Arc::make_mut(proof.builtins.as_mut().ok_or("source carrier")?);
    builtin.environment = Some(Arc::new(
        builtin
            .environment
            .as_ref()
            .ok_or("environment")?
            .as_ref()
            .clone(),
    ));
    assert!(
        seal.validate(&environment, &template.body, &proof, Some(&owner))
            .is_err()
    );
    Ok(())
}
#[test]
fn canonical_number_equality_does_not_attach_plan_to_foreign_or_replaced_arenas() -> Result {
    let (mut facts, template, proof) = fixture()?;
    let (foreign, _, _) = fixture()?;
    let mut copies = types::copy::Copies::default();
    let seal = Seal::new(
        &borrow::Environment {
            types: &facts.types,
            definitions: &facts.aggregates,
            derives: &facts.derives,
            source_copy: true,
            copies: &mut copies,
        },
        &template.body,
        &proof,
        None,
    );
    assert!(
        seal.validate(
            &borrow::Environment {
                types: &foreign.types,
                definitions: &foreign.aggregates,
                derives: &foreign.derives,
                source_copy: true,
                copies: &mut copies
            },
            &template.body,
            &proof,
            None
        )
        .is_err()
    );
    let replaced = std::mem::take(&mut facts.aggregates);
    facts.aggregates.arena_identity();
    assert!(
        seal.validate(
            &borrow::Environment {
                types: &facts.types,
                definitions: &facts.aggregates,
                derives: &facts.derives,
                source_copy: true,
                copies: &mut copies
            },
            &template.body,
            &proof,
            None
        )
        .is_err()
    );
    facts.aggregates = replaced;
    seal.validate(
        &borrow::Environment {
            types: &facts.types,
            definitions: &facts.aggregates,
            derives: &facts.derives,
            source_copy: true,
            copies: &mut copies,
        },
        &template.body,
        &proof,
        None,
    )?;
    Ok(())
}
