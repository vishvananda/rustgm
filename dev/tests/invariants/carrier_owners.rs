//! Alias metadata authenticates one actual body before a scoped Copy query exists.
use super::*;

#[test]
fn actual_body_alias_carriers_reject_foreign_missing_and_unprepared_inputs() -> Result {
    let source = crate::test_source::fixture(
        "pa31/tests/spec/trait-default-effects/projections/public-context/public-provider-moved.rs",
    )?;
    let (mut facts, templates, contracts) = fixture_source(&source)?;
    let template = templates
        .bodies
        .iter()
        .find(|template| {
            template.environment.body_aliases().is_some()
                && contracts.iter().any(|contract| {
                    contract.key == template.body.key()
                        && contract.requires_defining_cleanup_preparation()
                })
        })
        .ok_or("actual alias-bearing defining body")?;
    let contract = contracts
        .iter()
        .find(|contract| contract.key == template.body.key())
        .ok_or("actual original contract")?;
    let environment = Arc::clone(&template.environment);
    carrier::check(&mut facts, &template.body, contract, Some(&environment))?;
    let before = facts.body_work.canonical_steps;
    assert!(carrier::check(&mut facts, &template.body, contract, None).is_err());
    assert!(facts.body_work.canonical_steps > before);
    let foreign = Arc::new(environment.as_ref().clone());
    assert!(carrier::check(&mut facts, &template.body, contract, Some(&foreign)).is_err());
    let mut wrong = template.body.clone();
    let body::ir::Owner::Function { definition, .. } = &mut wrong.owner else {
        return Err("actual function owner".into());
    };
    *definition = crate::resolve::model::DefId(u32::MAX);
    assert!(carrier::check(&mut facts, &wrong, contract, Some(&environment)).is_err());
    let source = template.body.source.source().ok_or("actual source owner")?;
    let original = facts
        .generics
        .builtin_requirements
        .get_mut(&source)
        .ok_or("actual producer row")?
        .environment
        .take();
    let before = facts.body_work.canonical_steps;
    assert!(carrier::check(&mut facts, &template.body, contract, Some(&environment)).is_err());
    assert!(facts.body_work.canonical_steps > before);
    facts
        .generics
        .builtin_requirements
        .get_mut(&source)
        .ok_or("actual producer row")?
        .environment = original;
    let alias = *environment
        .body_aliases()
        .ok_or("actual aliases")?
        .ordered()
        .first()
        .ok_or("actual alias")?;
    let region = facts
        .body_regions
        .get_mut(source, alias)
        .ok_or("actual alias origin")?;
    let origin = region.origin;
    region.origin = crate::sema::model::BodyRegionOrigin::Inference;
    assert!(carrier::check(&mut facts, &template.body, contract, Some(&environment)).is_err());
    facts
        .body_regions
        .get_mut(source, alias)
        .ok_or("actual alias origin")?
        .origin = origin;
    carrier::check(&mut facts, &template.body, contract, Some(&environment))?;
    Ok(())
}
