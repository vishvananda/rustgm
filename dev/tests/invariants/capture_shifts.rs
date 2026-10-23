//! Shared capture relocation preserves lexical depth and simultaneous parameter insertion.
use crate::types::{
    ItemId, Kind, RegionId, Store, TypeId,
    arguments::Argument,
    parameters::{Parameter, ParameterKind},
    substitute::Bound,
};
fn reference(store: &mut Store, depth: u16) -> Result<TypeId, &'static str> {
    store.intern(Kind::Reference {
        mutable: false,
        region: RegionId::bound(depth, 0)?,
        referent: TypeId::UNIT,
    })
}
fn callable(store: &mut Store, local: TypeId, capture: TypeId) -> Result<TypeId, &'static str> {
    let parameters = store.intern_list(&[local, capture])?;
    store.intern(Kind::Function {
        parameters,
        result: capture,
        bound_regions: 1,
        unsafe_: false,
        convention: crate::types::callable::Convention::RUST,
    })
}
#[test]
fn shift_amounts_and_nested_callable_depths_are_independent() -> Result<(), &'static str> {
    let mut store = Store::default();
    let local = reference(&mut store, 0)?;
    let capture = reference(&mut store, 1)?;
    let function = callable(&mut store, local, capture)?;
    for amount in [1, 2, 3, 1] {
        let before = store.len();
        let (shifted, _) = store.shift_type(function, amount)?;
        let expected_capture = reference(&mut store, amount + 1)?;
        let expected = callable(&mut store, local, expected_capture)?;
        assert_eq!(shifted, expected);
        let after = store.len();
        assert_eq!(store.shift_type(function, amount)?, (expected, 0));
        assert_eq!(store.len(), after);
        assert!(after - before <= 2);
    }
    assert_eq!(store.shift_type(function, 0)?, (function, 0));
    Ok(())
}

#[test]
fn independent_parameter_maps_do_not_reapply_each_others_replacements() -> Result<(), &'static str>
{
    let mut store = Store::default();
    let mut parameters = Vec::new();
    for index in 0..2 {
        let id = store.intern_parameter(Parameter {
            owner: ItemId(1),
            index,
            kind: ParameterKind::Type { sized: false },
        })?;
        let ty = store.intern(Kind::Parameter(id))?;
        parameters.push((id, ty));
    }
    let replacement = store.intern(Kind::Reference {
        mutable: false,
        region: RegionId::bound(0, 0)?,
        referent: parameters[1].1,
    })?;
    let signature = callable(&mut store, parameters[0].1, parameters[0].1)?;
    for other in [TypeId::UNIT, parameters[0].1] {
        let mut mapping = Bound::parameters(
            &store,
            &[
                (parameters[0].0, Argument::Type(replacement)),
                (parameters[1].0, Argument::Type(other)),
            ],
        )?;
        let actual = mapping.apply(&mut store, signature)?;
        let inserted = store.intern(Kind::Reference {
            mutable: false,
            region: RegionId::bound(1, 0)?,
            referent: parameters[1].1,
        })?;
        let expected = callable(&mut store, inserted, inserted)?;
        assert_eq!(actual, expected);
    }
    Ok(())
}

#[test]
fn failed_capture_shift_preserves_completed_results_and_budget() -> Result<(), &'static str> {
    let mut store = Store::default();
    let input = reference(&mut store, 0)?;
    let (expected, _) = store.shift_type(input, 1)?;
    let before = store.shifts.as_ref().ok_or("missing shifts")?.steps;
    let overflow = reference(&mut store, 0x7fff)?;
    assert!(store.shift_type(overflow, 1).is_err());
    assert!(store.shift_type(TypeId(u32::MAX), 1).is_err());
    assert!(store.shifts.as_ref().ok_or("lost shifts")?.steps > before);
    assert_eq!(store.shift_type(input, 1)?, (expected, 0));
    let independent = reference(&mut store, 2)?;
    let (shifted, _) = store.shift_type(independent, 1)?;
    assert_eq!(shifted, reference(&mut store, 3)?);
    let shifts = store.shifts.as_mut().ok_or("missing shift budget")?;
    shifts.steps = 1_000_001;
    let folders = shifts.folders.len();
    assert!(store.shift_type(input, 2).is_err());
    assert_eq!(
        store
            .shifts
            .as_ref()
            .ok_or("lost shift budget")?
            .folders
            .len(),
        folders
    );
    Ok(())
}

#[test]
fn completed_capture_lookup_is_cold_readonly_and_bound_to_exact_amount_and_type()
-> Result<(), &'static str> {
    let mut store = Store::default();
    let local = reference(&mut store, 0)?;
    let capture = reference(&mut store, 1)?;
    let callback = callable(&mut store, local, capture)?;
    assert_eq!(store.prepared_shift(TypeId::UNIT, 1)?, Some(TypeId::UNIT));
    assert!(store.shifts.is_none());
    assert_eq!(store.prepared_shift(callback, 1)?, None);
    assert!(store.shifts.is_none());
    assert!(store.prepared_shift(TypeId(u32::MAX), 0).is_err());
    let (expected, _) = store.shift_type(callback, 1)?;
    let before = (store.len(), store.shifts.as_ref().ok_or("shifts")?.steps);
    for _ in 0..128 {
        assert_eq!(store.prepared_shift(callback, 1)?, Some(expected));
        assert_eq!(store.prepared_shift(callback, 2)?, None);
    }
    assert_eq!(
        (store.len(), store.shifts.as_ref().ok_or("shifts")?.steps),
        before
    );
    // A completed child at depth one cannot substitute for its root key.
    assert_eq!(store.prepared_shift(capture, 1)?, None);
    let (shifted_capture, _) = store.shift_type(capture, 1)?;
    assert_eq!(shifted_capture, reference(&mut store, 2)?);
    assert_eq!(store.prepared_shift(capture, 1)?, Some(shifted_capture));
    let foreign = reference(&mut store, 3)?;
    assert_eq!(store.prepared_shift(foreign, 1)?, None);
    Ok(())
}

#[test]
fn completed_capture_lookup_preserves_both_folder_and_shared_budget_failure()
-> Result<(), &'static str> {
    let mut store = Store::default();
    let input = reference(&mut store, 0)?;
    let (expected, _) = store.shift_type(input, 1)?;
    assert_eq!(store.prepared_shift(input, 1)?, Some(expected));
    let shifts = store.shifts.as_mut().ok_or("shifts")?;
    shifts.folders.get_mut(&1).ok_or("folder")?.steps = 1_000_001;
    assert!(store.prepared_shift(input, 1).is_err());
    assert!(store.prepared_shift(TypeId::UNIT, 1).is_err());
    let shifts = store.shifts.as_mut().ok_or("shifts")?;
    shifts.folders.get_mut(&1).ok_or("folder")?.steps = 0;
    shifts.steps = 1_000_001;
    assert!(store.prepared_shift(input, 1).is_err());
    assert!(store.prepared_shift(input, 2).is_err());
    assert!(store.prepared_shift(TypeId::UNIT, 0).is_err());
    Ok(())
}
