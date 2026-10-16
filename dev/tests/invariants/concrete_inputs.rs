//! Closed projection inputs distinguish static lifetimes from external variables.
use crate::types::{
    GenericArgs, ItemId, Kind, Primitive, RegionId, Store, TypeId,
    arguments::{Argument, ConstKind},
    parameters::{
        Parameter, ParameterKind,
        predicates::{Bound, Predicate},
    },
    projection, traits,
};

fn reference(store: &mut Store, region: RegionId) -> Result<TypeId, &'static str> {
    store.intern(Kind::Reference {
        referent: TypeId::UNIT,
        region,
        mutable: false,
    })
}
fn concrete(store: &Store, ty: TypeId) -> Result<bool, &'static str> {
    Predicate {
        ty,
        bound: Bound::Sized,
    }
    .is_concrete(store)
}

#[test]
fn static_inputs_preserve_substitution_flags_and_compact_storage() -> Result<(), &'static str> {
    assert_eq!(std::mem::size_of::<super::Properties>(), 4);
    let mut store = Store::default();
    let static_ = reference(&mut store, RegionId::STATIC)?;
    let free = reference(&mut store, RegionId(7))?;
    assert_eq!(store.has_free_regions(static_, 0), Some(true));
    assert!(concrete(&store, static_)?);
    assert!(!concrete(&store, free)?);
    for elements in [[static_, free], [free, static_]] {
        let elements = store.intern_list(&elements)?;
        let tuple = store.intern(Kind::Tuple(elements))?;
        assert!(!concrete(&store, tuple)?);
    }
    Ok(())
}

#[test]
fn callable_local_binders_do_not_hide_external_lifetime_dependencies() -> Result<(), &'static str> {
    let mut store = Store::default();
    for (region, expected) in [
        (RegionId::STATIC, true),
        (RegionId::bound(0, 0)?, true),
        (RegionId::bound(1, 0)?, false),
        (RegionId(9), false),
    ] {
        let input = reference(&mut store, region)?;
        let parameters = store.intern_list(&[input])?;
        let callable = store.intern(Kind::Function {
            parameters,
            result: TypeId::UNIT,
            bound_regions: 1,
            unsafe_: false,
        })?;
        assert_eq!(concrete(&store, callable)?, expected);
    }
    Ok(())
}

#[test]
fn trait_arguments_and_projected_inputs_remain_part_of_closedness() -> Result<(), &'static str> {
    let mut store = Store::default();
    let ty = store.intern_parameter(Parameter {
        owner: ItemId(1),
        index: 0,
        kind: ParameterKind::Type { sized: true },
    })?;
    let ty = store.intern(Kind::Parameter(ty))?;
    let constant = store.intern_parameter(Parameter {
        owner: ItemId(1),
        index: 1,
        kind: ParameterKind::Const {
            ty: TypeId::primitive(Primitive::Usize),
        },
    })?;
    let constant = store.intern_const(ConstKind::Parameter(constant))?;
    for (argument, expected) in [
        (Argument::Lifetime(RegionId::STATIC), true),
        (Argument::Lifetime(RegionId(3)), false),
        (Argument::Lifetime(RegionId::bound(0, 0)?), false),
        (Argument::Type(ty), false),
        (Argument::Const(constant), false),
    ] {
        let arguments =
            store.intern_arguments(&[argument, Argument::Lifetime(RegionId::STATIC)])?;
        let application = store.intern_trait_application(traits::Application {
            item: ItemId(2),
            arguments,
        })?;
        assert_eq!(
            Predicate {
                ty: TypeId::UNIT,
                bound: Bound::Trait(application),
            }
            .is_concrete(&store)?,
            expected
        );
    }
    store.declare_projection(
        ItemId(3),
        projection::Schema {
            trait_item: ItemId(2),
            sized: true,
            copies: crate::types::copy::Properties::default(),
        },
    )?;
    let trait_application = store.intern_trait_application(traits::Application {
        item: ItemId(2),
        arguments: GenericArgs::EMPTY,
    })?;
    let projected = store.intern(Kind::Projection(projection::Application {
        item: ItemId(3),
        subject: TypeId::UNIT,
        trait_application,
        arguments: GenericArgs::EMPTY,
    }))?;
    assert!(!concrete(&store, projected)?);
    Ok(())
}
