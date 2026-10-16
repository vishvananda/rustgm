//! Independent coordinate and shared-graph checks for immutable assumption lookup.
use super::Lookup;
use crate::types::{
    GenericArgs, ItemId, Kind, Primitive, RegionId, Store, TypeId,
    arguments::{Argument, Constant},
    parameters::{
        environment::Environment,
        predicates::{Bound, Predicate},
        shape::Shapes,
    },
    projection, traits,
};
use std::sync::Arc;
type Result = std::result::Result<(), &'static str>;
fn reference(store: &mut Store, depth: u16) -> std::result::Result<TypeId, &'static str> {
    store.intern(Kind::Reference {
        mutable: false,
        region: RegionId::bound(depth, 0)?,
        referent: TypeId::UNIT,
    })
}
fn function(
    store: &mut Store,
    parameters: &[TypeId],
    bound_regions: u32,
) -> std::result::Result<TypeId, &'static str> {
    let parameters = store.intern_list(parameters)?;
    store.intern(Kind::Function {
        parameters,
        result: TypeId::UNIT,
        bound_regions,
        unsafe_: false,
    })
}
fn projected(store: &mut Store, depth: u16) -> std::result::Result<TypeId, &'static str> {
    store.declare_projection(
        ItemId(2),
        projection::Schema {
            trait_item: ItemId(1),
            sized: false,
            copies: crate::types::copy::Properties::default(),
        },
    )?;
    let arguments = store.intern_arguments(&[Argument::Lifetime(RegionId::bound(depth, 0)?)])?;
    let trait_application = store.intern_trait_application(traits::Application {
        item: ItemId(1),
        arguments,
    })?;
    store.intern(Kind::Projection(projection::Application {
        item: ItemId(2),
        subject: TypeId::UNIT,
        trait_application,
        arguments: GenericArgs::EMPTY,
    }))
}
#[test]
fn lookup_preserves_nested_locals_and_complete_mixed_arguments() -> Result {
    let mut store = Store::default();
    let local = reference(&mut store, 0)?;
    let outer = reference(&mut store, 1)?;
    let shifted = reference(&mut store, 3)?;
    let original = function(&mut store, &[local, outer], 1)?;
    let shifted = function(&mut store, &[local, shifted], 1)?;
    let three = store.intern_constant(Constant {
        ty: TypeId::primitive(Primitive::Usize),
        bits: 3,
    })?;
    let four = store.intern_constant(Constant {
        ty: TypeId::primitive(Primitive::Usize),
        bits: 4,
    })?;
    let original_args = store.intern_arguments(&[
        Argument::Type(original),
        Argument::Lifetime(RegionId::bound(0, 1)?),
        Argument::Const(three),
    ])?;
    let shifted_args = store.intern_arguments(&[
        Argument::Type(shifted),
        Argument::Lifetime(RegionId::bound(2, 1)?),
        Argument::Const(three),
    ])?;
    let wrong_args = store.intern_arguments(&[
        Argument::Type(shifted),
        Argument::Lifetime(RegionId::bound(2, 1)?),
        Argument::Const(four),
    ])?;
    let original_application = store.intern_trait_application(traits::Application {
        item: ItemId(7),
        arguments: original_args,
    })?;
    let shifted_application = store.intern_trait_application(traits::Application {
        item: ItemId(7),
        arguments: shifted_args,
    })?;
    let wrong_application = store.intern_trait_application(traits::Application {
        item: ItemId(7),
        arguments: wrong_args,
    })?;
    let count = store.len();
    let mut lookup = Lookup::new(2);
    assert_eq!(lookup.ty(&store, shifted)?, Some(original));
    assert_eq!(
        lookup.application(&store, shifted_application)?,
        Some(original_application)
    );
    assert_eq!(lookup.ty(&store, local)?, None);
    assert_eq!(lookup.application(&store, wrong_application)?, None);
    assert_eq!(store.len(), count);
    let work = lookup.steps();
    for _ in 0..100 {
        assert_eq!(lookup.ty(&store, shifted)?, Some(original));
        assert_eq!(
            lookup.application(&store, shifted_application)?,
            Some(original_application)
        );
        assert_eq!(lookup.ty(&store, local)?, None);
    }
    assert_eq!(lookup.steps(), work);
    Ok(())
}
#[test]
fn shape_cache_separates_root_captures_and_local_binders() -> Result {
    let mut store = Store::default();
    let root = projected(&mut store, 0)?;
    let captured = projected(&mut store, 1)?;
    let double = projected(&mut store, 2)?;
    let Kind::Projection(application) = store.kind(root).ok_or("missing projection")? else {
        return Err("unexpected projection kind");
    };
    let premise = Predicate {
        ty: TypeId::UNIT,
        bound: Bound::Trait(application.trait_application),
    };
    let environment = Environment::new(&store, &[])?.with_predicates(
        &store,
        &[
            premise,
            Predicate {
                ty: root,
                bound: Bound::Sized,
            },
        ],
    )?;
    let array = store.intern(Kind::Array {
        element: captured,
        length: 1,
    })?;
    let valid = function(&mut store, &[array], 1)?;
    let empty = function(&mut store, &[array], 0)?;
    let local = function(&mut store, &[root], 1)?;
    let inner = function(&mut store, &[double], 0)?;
    let double = function(&mut store, &[inner], 0)?;
    let count = store.len();
    let mut shapes = Shapes::new(Arc::new(environment));
    assert!(shapes.demand(&store, valid)?.well_formed);
    assert!(shapes.demand(&store, root)?.sized);
    assert!(shapes.demand(&store, empty)?.well_formed);
    assert!(shapes.demand(&store, double)?.well_formed);
    assert!(shapes.demand(&store, captured).is_err());
    assert!(shapes.demand(&store, local).is_err());
    assert!(shapes.demand(&store, valid)?.well_formed);
    let environment = Environment::new(&store, &[])?.with_predicates(&store, &[premise])?;
    assert!(
        !Shapes::new(Arc::new(environment))
            .demand(&store, valid)?
            .well_formed
    );
    assert_eq!(store.len(), count);
    Ok(())
}
#[test]
fn absent_canonical_targets_cannot_add_permissions_to_a_frozen_environment() -> Result {
    let mut store = Store::default();
    let shifted = reference(&mut store, 2)?;
    let mut lookup = Lookup::new(2);
    assert_eq!(lookup.ty(&store, shifted)?, None);
    let root = reference(&mut store, 0)?;
    // New assumptions require a new query. Unrelated later interning cannot
    // extend the immutable environment associated with the original lookup.
    assert_eq!(lookup.ty(&store, shifted)?, None);
    assert_eq!(Lookup::new(2).ty(&store, shifted)?, Some(root));
    Ok(())
}
#[test]
fn shared_captured_graphs_are_walked_once_without_recursive_ownership() -> Result {
    let mut store = Store::default();
    let mut original = reference(&mut store, 0)?;
    let mut shifted = reference(&mut store, 2)?;
    for _ in 0..10_000 {
        let list = store.intern_list(&[original, original])?;
        original = store.intern(Kind::Tuple(list))?;
        let list = store.intern_list(&[shifted, shifted])?;
        shifted = store.intern(Kind::Tuple(list))?;
    }
    let mut lookup = Lookup::new(2);
    assert_eq!(lookup.ty(&store, shifted)?, Some(original));
    assert!(lookup.steps() < 100_000);
    let work = lookup.steps();
    for _ in 0..100 {
        assert_eq!(lookup.ty(&store, shifted)?, Some(original));
    }
    assert_eq!(lookup.steps(), work);
    Ok(())
}
