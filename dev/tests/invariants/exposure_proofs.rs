//! Concrete table exposure does not change a body's ownership-proof mode.
use super::*;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

fn fixture() -> Result<(Program, Vec<Proved>, glue::Recipes)> {
    let report = crate::test_source::resolve("fn f(){}", true)?;
    let mut facts = crate::sema::declarations::analyze(&report)?;
    facts = crate::sema::bodies::analyze(&report, facts)?;
    let mut program = crate::body::build(&report, &facts)?;
    assert!(program.drops.is_none());
    let bodies = std::mem::take(&mut program.bodies)
        .into_iter()
        .map(|body| Proved {
            body,
            plan: None,
            destruction: false,
            static_targets: None,
        })
        .collect();
    let cache = glue::Cache::exposed_empty(&mut facts.types, &mut facts.aggregates)?;
    let mut catalog = cache.attach(&facts.types, &facts.aggregates, &facts.members);
    assert_eq!(catalog.expose(crate::types::TypeId::UNIT)?, None);
    Ok((program, bodies, catalog.into_recipes()))
}

#[test]
fn exposure_only_assembly_has_no_body_plans_or_invented_drop_flags() -> Result {
    let (program, bodies, recipes) = fixture()?;
    assert!(!bodies.is_empty());
    let (program, proof) = assemble(program, bodies, Some(recipes))?.into_lowering();
    assert!(program.drops.is_none());
    let proof = proof.ok_or("exposed recipes")?;
    assert_eq!(proof.plan_capacity(), 0);
    assert_eq!(proof.recipes().exposed(crate::types::TypeId::UNIT)?, None);
    assert!(
        proof
            .recipes()
            .exposed(crate::types::TypeId(u32::MAX))
            .is_err()
    );
    Ok(())
}

#[test]
fn exposure_only_assembly_rejects_empty_catalog_and_invented_plan_authority() -> Result {
    let (program, bodies, _) = fixture()?;
    assert!(assemble(program, bodies, Some(glue::Recipes::default())).is_err());
    let (program, mut bodies, recipes) = fixture()?;
    bodies.first_mut().ok_or("body")?.destruction = true;
    assert!(assemble(program, bodies, Some(recipes)).is_err());
    let (program, mut bodies, recipes) = fixture()?;
    bodies.first_mut().ok_or("body")?.plan = Some(Box::default());
    assert!(assemble(program, bodies, Some(recipes)).is_err());
    Ok(())
}
