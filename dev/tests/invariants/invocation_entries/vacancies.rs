//! Owned generic receivers leave the original appended source slot unused.
use super::*;
#[test]
fn owned_adapter_keeps_later_argument_and_nested_binder_slots() -> Result {
    let mut types = Store::default();
    let receiver = types.intern(Type::Reference {
        referent: TypeId::UNIT,
        mutable: false,
        region: RegionId::bound(0, 1)?,
    })?;
    let later = reference(&mut types, RegionId::bound(0, 2)?)?;
    let nested_local = reference(&mut types, RegionId::bound(0, 0)?)?;
    let nested_outer = reference(&mut types, RegionId::bound(1, 2)?)?;
    let nested = function(&mut types, &[nested_local, nested_outer], 1)?;
    let signature = function(&mut types, &[receiver, later, nested], 3)?;
    let canonical = Execution {
        self_ty: TypeId::UNIT,
        signature,
        universals: 3,
        receiver: CallKind::Shared,
    };
    let execution = shape_execution(&mut types, canonical, 1, CallTrait::FnOnce, true)?;
    assert_eq!(execution.universals, 3);
    let Some(Type::Function {
        parameters,
        bound_regions: 3,
        ..
    }) = types.kind(execution.signature)
    else {
        return Err("owned execution lost its retained binder");
    };
    assert_eq!(
        types.list(parameters),
        Some([TypeId::UNIT, later, nested].as_slice())
    );
    let actual_later = reference(&mut types, RegionId(9))?;
    let actual_outer = reference(&mut types, RegionId::bound(1, 7)?)?;
    let actual_nested = function(&mut types, &[nested_local, actual_outer], 1)?;
    let actual = function(&mut types, &[TypeId::UNIT, actual_later, actual_nested], 0)?;
    let bindings = matching::execution_bindings(&types, execution, actual)?;
    assert!(!bindings.slots.contains_key(&1));
    let nested_actual = RegionId::bound(0, 7)?;
    assert!(
        matches!(bindings.slots.get(&2), Some(region) if *region == RegionId(9) || *region == nested_actual)
    );
    assert_eq!(bindings.equalities.len(), 1);
    let pair = bindings.equalities[0];
    assert!(pair == (RegionId(9), nested_actual) || pair == (nested_actual, RegionId(9)));
    Ok(())
}
