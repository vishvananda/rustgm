//! Object input binders and object lifetimes must survive independent type folds.
use rustgm::types::{
    GenericArgs, ItemId, Kind, RegionId, Store, TypeId,
    arguments::Argument,
    auto_traits::AutoTrait,
    dynamic::{Binding, Bounds, ShapeId},
    infer::Context,
    substitute::Bound,
};

fn shape(types: &mut Store, arguments: u32) -> Result<ShapeId, &'static str> {
    types.intern_dynamic_shape(Bounds {
        principal: Some(ItemId(10)),
        principal_arguments: arguments,
        bindings: &[],
        auto_traits: &[],
    })
}

#[test]
fn object_bound_sets_have_canonical_identity() -> Result<(), &'static str> {
    let mut types = Store::default();
    let mut make = |autos: &[AutoTrait]| {
        types.intern_dynamic_shape(Bounds {
            principal: Some(ItemId(10)),
            principal_arguments: 0,
            bindings: &[],
            auto_traits: autos,
        })
    };
    let a = make(&[AutoTrait::Sync, AutoTrait::Send])?;
    let b = make(&[AutoTrait::Send, AutoTrait::Sync, AutoTrait::Send])?;
    assert_eq!(a, b);
    assert_eq!(
        types.dynamic_auto_traits(a),
        Some([AutoTrait::Send, AutoTrait::Sync].as_slice())
    );
    let plain = shape(&mut types, 0)?;
    assert_ne!(a, plain);
    let dynamic = types.intern(Kind::Dynamic {
        shape: a,
        arguments: GenericArgs::EMPTY,
        bound_regions: 0,
        region: RegionId::STATIC,
    })?;
    assert_eq!(types.is_sized(dynamic), Some(false));
    Ok(())
}

#[test]
fn associated_equalities_validate_complete_input_layouts() -> Result<(), &'static str> {
    let mut types = Store::default();
    let shape = types.intern_dynamic_shape(Bounds {
        principal: Some(ItemId(10)),
        principal_arguments: 1,
        bindings: &[Binding {
            item: ItemId(12),
            trait_item: ItemId(11),
            arguments: 1,
        }],
        auto_traits: &[],
    })?;
    let valid = types.intern_arguments(&[
        Argument::Lifetime(RegionId::STATIC),
        Argument::Type(TypeId::UNIT),
        Argument::Type(TypeId::NEVER),
    ])?;
    assert!(types.valid_dynamic_arguments(shape, valid));
    let wrong_result = types.intern_arguments(&[
        Argument::Lifetime(RegionId::STATIC),
        Argument::Type(TypeId::UNIT),
        Argument::Lifetime(RegionId::STATIC),
    ])?;
    assert!(!types.valid_dynamic_arguments(shape, wrong_result));
    assert!(!types.valid_dynamic_arguments(shape, GenericArgs::EMPTY));
    assert!(!types.valid_dynamic_arguments(ShapeId(u32::MAX), valid));
    assert!(
        types
            .intern_dynamic_shape(Bounds {
                principal: None,
                principal_arguments: 0,
                bindings: &[],
                auto_traits: &[],
            })
            .is_err()
    );
    Ok(())
}

#[test]
fn substitution_separates_object_and_trait_binders() -> Result<(), &'static str> {
    let mut types = Store::default();
    let shape = shape(&mut types, 2)?;
    let local = RegionId::bound(0, 0)?;
    let outer = RegionId::bound(1, 0)?;
    let reference = types.intern(Kind::Reference {
        mutable: false,
        region: outer,
        referent: TypeId::UNIT,
    })?;
    let arguments =
        types.intern_arguments(&[Argument::Lifetime(local), Argument::Type(reference)])?;
    let original = types.intern(Kind::Dynamic {
        shape,
        arguments,
        bound_regions: 1,
        region: local,
    })?;
    assert_eq!(types.escaping_binders(original), Some(1));
    let result = Bound::new(vec![RegionId(17)]).apply(&mut types, original)?;
    assert_eq!(types.escaping_binders(result), Some(0));
    let expected_reference = types.intern(Kind::Reference {
        mutable: false,
        region: RegionId(17),
        referent: TypeId::UNIT,
    })?;
    let arguments = types.intern_arguments(&[
        Argument::Lifetime(local),
        Argument::Type(expected_reference),
    ])?;
    let expected = types.intern(Kind::Dynamic {
        shape,
        arguments,
        bound_regions: 1,
        region: RegionId(17),
    })?;
    assert_eq!(result, expected);
    let mut inference = Context::default();
    let term = inference
        .known(&types, original)
        .map_err(|_| "invalid object term")?;
    inference
        .head(&types, term)
        .map_err(|_| "object materialization failed")?;
    let mapped = inference
        .substitute_regions(&mut types, term, &[RegionId(17)])
        .map_err(|_| "object inference binder substitution failed")?;
    let wanted = inference
        .known(&types, expected)
        .map_err(|_| "invalid expected object term")?;
    inference
        .equate(&types, mapped, wanted)
        .map_err(|_| "object inference result mismatch")?;
    Ok(())
}

#[test]
fn object_lifetime_does_not_erase_trait_argument_requirements() -> Result<(), &'static str> {
    use rustgm::types::{aggregate, callable, destruction, outlives};
    let mut types = Store::default();
    let shape = shape(&mut types, 1)?;
    let reference = types.intern(Kind::Reference {
        mutable: false,
        region: RegionId(17),
        referent: TypeId::UNIT,
    })?;
    let arguments = types.intern_arguments(&[Argument::Type(reference)])?;
    let object = types.intern(Kind::Dynamic {
        shape,
        arguments,
        bound_regions: 0,
        region: RegionId(18),
    })?;
    let bounds = outlives::Bounds::default();
    let callables = callable::Templates::default();
    let mut query = outlives::Queries::new(&types, &bounds, &callables);
    let mut regions = query.components(object)?.scalars.regions.to_vec();
    regions.sort_unstable_by_key(|r| r.0);
    assert_eq!(regions, [RegionId(17), RegionId(18)]);
    let definitions = aggregate::Definitions::default();
    let mut drops = destruction::regions::Regions::new([])?;
    assert_eq!(
        drops.get(&types, &definitions, object)?,
        [RegionId(17), RegionId(18)]
    );
    Ok(())
}

#[test]
fn object_lifetimes_shorten_but_trait_arguments_are_invariant()
-> Result<(), Box<dyn std::error::Error>> {
    use rustgm::types::{relations::Constraints, variance::Variances};
    let mut types = Store::default();
    let shape = shape(&mut types, 1)?;
    let a = RegionId::bound(0, 0)?;
    let b = RegionId::bound(0, 1)?;
    let aa = types.intern_arguments(&[Argument::Lifetime(RegionId::bound(1, 0)?)])?;
    let ba = types.intern_arguments(&[Argument::Lifetime(RegionId::bound(1, 1)?)])?;
    let actual = types.intern(Kind::Dynamic {
        shape,
        arguments: aa,
        bound_regions: 0,
        region: a,
    })?;
    for (arguments, accepted) in [(aa, true), (ba, false)] {
        let expected = types.intern(Kind::Dynamic {
            shape,
            arguments,
            bound_regions: 0,
            region: b,
        })?;
        let variances = Variances::default();
        let mut constraints = Constraints::new(&types, &variances, 2, 10_000)?;
        constraints.assume(a, b)?;
        constraints.coerce(actual, expected, false, 0)?;
        assert_eq!(constraints.solve().is_ok(), accepted);
    }
    Ok(())
}

#[test]
fn object_binders_allow_alpha_renaming_and_unused_slots() -> Result<(), &'static str> {
    let mut types = Store::default();
    let shape = shape(&mut types, 2)?;
    let mut object = |count, first, second| {
        let arguments = types.intern_arguments(&[
            Argument::Lifetime(RegionId::bound(0, first)?),
            Argument::Lifetime(RegionId::bound(0, second)?),
        ])?;
        types.intern(Kind::Dynamic {
            shape,
            arguments,
            bound_regions: count,
            region: RegionId::STATIC,
        })
    };
    let a = object(2, 0, 1)?;
    let b = object(3, 2, 0)?;
    let repeated = object(2, 0, 0)?;
    for (other, accepted) in [(b, true), (repeated, false)] {
        let mut inference = Context::default();
        let a = inference
            .known(&types, a)
            .map_err(|_| "invalid first object")?;
        let b = inference
            .known(&types, other)
            .map_err(|_| "invalid second object")?;
        assert_eq!(
            inference.equate_scoped_terms(&types, a, b).is_ok(),
            accepted
        );
        assert!(inference.region_equalities.is_empty());
    }
    Ok(())
}

#[test]
fn object_pointers_use_vtable_metadata_without_becoming_copy_values() -> Result<(), &'static str> {
    use rustgm::{
        abi::{
            layout::{Layouts, Shape},
            metadata::Metadata,
        },
        types::{aggregate, copy, destruction},
    };
    let mut types = Store::default();
    let shape = shape(&mut types, 0)?;
    let object = types.intern(Kind::Dynamic {
        shape,
        arguments: GenericArgs::EMPTY,
        bound_regions: 0,
        region: RegionId::STATIC,
    })?;
    let pointer = types.intern(Kind::Reference {
        mutable: false,
        region: RegionId::STATIC,
        referent: object,
    })?;
    let definitions = aggregate::Definitions::default();
    let mut layouts = Layouts::default();
    assert_eq!(
        layouts.pointer_metadata(&types, &definitions, object)?,
        Metadata::VTable(object)
    );
    layouts
        .demand(&types, &definitions, pointer)
        .map_err(|e| e.message)?;
    let layout = layouts
        .get(pointer)
        .ok_or("missing object pointer layout")?;
    assert!(matches!(layout.shape, Shape::WidePointer));
    assert_eq!((layout.size, layout.align), (16, 8));
    let selected = rustgm::support::hash::HashMap::default();
    let mut copies = copy::Copies::default();
    assert!(!copies.demand(&types, &definitions, &selected, object)?.copy);
    assert!(
        copies
            .demand(&types, &definitions, &selected, pointer)?
            .copy
    );
    let mut drops = destruction::NeedsDrop::new([]);
    assert!(drops.demand(&types, &definitions, object)?);
    assert!(!drops.demand(&types, &definitions, pointer)?);
    Ok(())
}
