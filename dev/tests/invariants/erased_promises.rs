//! Principal promises map each complete object application across its own binder.
use crate::types::{
    GenericArgs, ItemId, Kind, RegionId, Store, TypeId, arguments::Argument, dynamic::Bounds,
};

fn object(
    types: &mut Store,
    inputs: &[RegionId],
    count: u32,
    region: RegionId,
) -> Result<TypeId, &'static str> {
    let shape = types.intern_dynamic_shape(Bounds {
        principal: Some(ItemId(9)),
        principal_arguments: u32::try_from(inputs.len()).map_err(|_| "too many inputs")?,
        bindings: &[],
        auto_traits: &[],
    })?;
    let arguments = arguments(types, inputs)?;
    types.intern(Kind::Dynamic {
        shape,
        arguments,
        bound_regions: count,
        region,
    })
}
fn arguments(types: &mut Store, regions: &[RegionId]) -> Result<GenericArgs, &'static str> {
    types.intern_arguments(
        &regions
            .iter()
            .copied()
            .map(Argument::Lifetime)
            .collect::<Vec<_>>(),
    )
}

#[test]
fn guarantees_follow_declared_slots_after_object_substitution() -> Result<(), &'static str> {
    let mut types = Store::default();
    let first = RegionId::bound(0, 0)?;
    let second = RegionId::bound(0, 1)?;
    let template = arguments(&mut types, &[first, second])?;
    types.declare_dynamic_outlives(ItemId(9), template, &[second])?;
    let region = RegionId::bound(0, 5)?;
    let a = RegionId::bound(1, 3)?;
    let b = RegionId::bound(1, 7)?;
    for (inputs, expected) in [
        ([a, b], RegionId::bound(0, 7)?),
        ([b, a], RegionId::bound(0, 3)?),
    ] {
        let ty = object(&mut types, &inputs, 0, region)?;
        let promises = types
            .dynamic_requirements(ty, 0)?
            .collect::<Result<Vec<_>, _>>()?;
        assert_eq!(promises, vec![Some((region, expected))]);
    }
    assert!(
        types
            .declare_dynamic_outlives(ItemId(9), template, &[first])
            .is_err()
    );
    Ok(())
}

#[test]
fn local_quantifier_guarantees_do_not_escape_the_object_binder() -> Result<(), &'static str> {
    let mut types = Store::default();
    let local = RegionId::bound(0, 0)?;
    let template = arguments(&mut types, &[local])?;
    types.declare_dynamic_outlives(ItemId(9), template, &[local])?;
    let explicit = RegionId(17);
    let ty = object(&mut types, &[local], 1, explicit)?;
    assert_eq!(
        types
            .dynamic_requirements(ty, 0)?
            .collect::<Result<Vec<_>, _>>()?,
        vec![None]
    );
    Ok(())
}
