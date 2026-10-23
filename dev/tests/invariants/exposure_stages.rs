//! A drained table-only batch must retain concrete exposure for a later helper.
use super::*;
use crate::types::{Kind, TypeId};
use std::path::Path;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

#[test]
fn first_vtable_batch_retains_exposure_before_later_dynamic_helper_demand() -> Result {
    let source = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../pa31/tests/spec/drop-glue-dyn/complete-empty-null-entry.rs"),
    )?;
    let report = crate::test_source::source_provider(&source, "exposure_stages")?;
    let crate::pipeline::Analysis { mut facts, .. } = crate::pipeline::analyze_source(&report)?;
    let program = crate::body::build_concrete(&report, &facts)?;
    let (mut builder, _) = Builder::new(program)?;
    assert!(builder.program.drops.is_none());
    assert!(builder.recipes.is_none());
    assert!(builder.object_dependencies(&mut facts, &[])?.is_empty());
    assert!(
        builder.recipes.is_none(),
        "no exposure must allocate no recipe cache"
    );
    let object = (0..u32::try_from(facts.types.len())?)
        .map(TypeId)
        .find(|&ty| matches!(facts.types.kind(ty), Some(Kind::Dynamic { .. })))
        .ok_or("object")?;
    let concrete = (0..u32::try_from(facts.types.len())?)
        .map(TypeId)
        .find(|&ty| matches!(facts.types.kind(ty), Some(Kind::Adt(..))))
        .ok_or("concrete")?;
    sema::vtable_plans::prepare_table(&report, &mut facts, concrete, object)?;
    let tables = sema::vtable_plans::complete_tables(&mut facts)?;
    assert!(!tables.is_empty());
    let roots = tables
        .iter()
        .map(|&id| {
            facts
                .objects
                .table(id)
                .map(|table| table.concrete)
                .ok_or("table")
        })
        .collect::<std::result::Result<Vec<_>, _>>()?;
    assert!(builder.object_dependencies(&mut facts, &roots)?.is_empty());
    assert!(sema::vtable_plans::complete_tables(&mut facts)?.is_empty());
    let before = builder
        .recipes
        .as_ref()
        .ok_or("retained table cache")?
        .recipes()
        .exposed(concrete)?;
    assert_eq!(before, None);
    assert!(
        builder
            .object_dependencies(&mut facts, &[object])?
            .is_empty()
    );
    let recipes = builder
        .recipes
        .as_ref()
        .ok_or("retained helper cache")?
        .recipes();
    assert_eq!(recipes.exposed(concrete)?, before);
    let recipe = recipes
        .get(recipes.exposed(object)?.ok_or("object recipe")?)
        .ok_or("recipe")?;
    assert!(matches!(recipe.kind, glue::Shape::Dynamic));
    assert!(builder.program.drops.is_none());
    Ok(())
}
