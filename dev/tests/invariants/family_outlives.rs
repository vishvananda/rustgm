//! Declared result lifetime promises address parent and family slots independently.
use crate::types::{
    GenericArgs, ItemId, Primitive, RegionId, Store, TypeId,
    arguments::{Argument, Constant},
    projection::{Application, Schema},
    traits,
};
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
const BYTE: TypeId = TypeId::primitive(Primitive::U8);

fn application(store: &mut Store, own: &[Argument]) -> TestResult<Application> {
    store.declare_projection(
        ItemId(2),
        Schema {
            trait_item: ItemId(1),
            sized: false,
            copies: crate::types::copy::Properties::default(),
        },
    )?;
    let constant = store.intern_constant(Constant { ty: BYTE, bits: 3 })?;
    let arguments = store.intern_arguments(&[
        Argument::Lifetime(RegionId(7)),
        Argument::Lifetime(RegionId(8)),
        Argument::Type(BYTE),
        Argument::Const(constant),
    ])?;
    let trait_application = store.intern_trait_application(traits::Application {
        item: ItemId(1),
        arguments,
    })?;
    Ok(Application {
        item: ItemId(2),
        subject: BYTE,
        trait_application,
        arguments: store.intern_arguments(own)?,
    })
}

#[test]
fn family_lifetimes_start_after_captured_regions_not_parent_type_arguments() -> TestResult {
    let mut store = Store::default();
    let value = application(
        &mut store,
        &[
            Argument::Lifetime(RegionId(9)),
            Argument::Lifetime(RegionId(10)),
            Argument::Type(BYTE),
        ],
    )?;
    let promises = [
        RegionId::bound(0, 1)?,
        RegionId::bound(0, 2)?,
        RegionId::bound(0, 3)?,
        RegionId::STATIC,
    ];
    store.declare_projection_outlives(value.item, 2, &promises)?;
    let actual = store
        .projection_outlives(value, 0)?
        .collect::<Result<Vec<_>, _>>()?;
    assert_eq!(
        actual,
        [
            Some(RegionId(8)),
            Some(RegionId(9)),
            Some(RegionId(10)),
            Some(RegionId::STATIC)
        ]
    );
    store.declare_projection_outlives(value.item, 2, &promises)?;
    assert!(
        store
            .declare_projection_outlives(value.item, 1, &promises)
            .is_err()
    );
    let other = application(
        &mut store,
        &[
            Argument::Lifetime(RegionId(12)),
            Argument::Lifetime(RegionId(13)),
        ],
    )?;
    let actual = store
        .projection_outlives(other, 0)?
        .collect::<Result<Vec<_>, _>>()?;
    assert_eq!(
        actual,
        [
            Some(RegionId(8)),
            Some(RegionId(12)),
            Some(RegionId(13)),
            Some(RegionId::STATIC)
        ]
    );
    Ok(())
}

#[test]
fn nested_local_family_lifetimes_do_not_escape_as_outer_promises() -> TestResult {
    let mut store = Store::default();
    let value = application(
        &mut store,
        &[
            Argument::Lifetime(RegionId::bound(0, 0)?),
            Argument::Lifetime(RegionId::bound(2, 1)?),
            Argument::Lifetime(RegionId::STATIC),
        ],
    )?;
    let promises = (2..5)
        .map(|index| RegionId::bound(0, index))
        .collect::<Result<Vec<_>, _>>()?;
    store.declare_projection_outlives(value.item, 2, &promises)?;
    let actual = store
        .projection_outlives(value, 2)?
        .collect::<Result<Vec<_>, _>>()?;
    assert_eq!(
        actual,
        [None, Some(RegionId::bound(0, 1)?), Some(RegionId::STATIC)]
    );
    Ok(())
}

#[test]
fn malformed_lifetime_templates_or_family_arguments_are_rejected() -> TestResult {
    let mut store = Store::default();
    let value = application(&mut store, &[Argument::Type(BYTE)])?;
    for invalid in [RegionId(30), RegionId::bound(1, 0)?] {
        assert!(
            store
                .declare_projection_outlives(value.item, 2, &[invalid])
                .is_err()
        );
    }
    store.declare_projection_outlives(value.item, 2, &[RegionId::bound(0, 2)?])?;
    assert!(
        store
            .projection_outlives(value, 0)?
            .next()
            .ok_or("missing promise")?
            .is_err()
    );
    assert!(
        store
            .projection_outlives(
                Application {
                    arguments: GenericArgs::EMPTY,
                    ..value
                },
                0
            )?
            .next()
            .ok_or("missing promise")?
            .is_err()
    );
    Ok(())
}
