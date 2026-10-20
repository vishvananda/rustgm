//! Dynamic tail queries retain one clamp per aggregate and constant-time field lookup.
use super::{Fixture, Repr, Representation, U8, U16, U32};
use rustgm::{
    abi::tails::{Alignment, Offset},
    types::{GenericArgs, ItemId, Kind, RegionId, TypeId, dynamic::Bounds},
};

fn object(f: &mut Fixture) -> Result<TypeId, &'static str> {
    let shape = f.types.intern_dynamic_shape(Bounds {
        principal: Some(ItemId(777)),
        principal_arguments: 0,
        bindings: &[],
        auto_traits: &[],
    })?;
    f.types.intern(Kind::Dynamic {
        shape,
        arguments: GenericArgs::EMPTY,
        bound_regions: 0,
        region: RegionId::STATIC,
    })
}

#[test]
fn nested_objects_retain_prefix_offsets_and_alignment_clamps() -> Result<(), &'static str> {
    let mut f = Fixture::default();
    let object = object(&mut f)?;
    let c = Repr {
        representation: Representation::C,
        ..Repr::default()
    };
    let inner = f.record(c, &[U16, object])?;
    let outer = f.record(c, &[U8, U32, inner])?;
    let id = f
        .tails
        .demand(&f.types, &f.definitions, &mut f.layouts, outer)?;
    let layout = *f.tails.at(id).ok_or("missing outer layout")?;
    assert_eq!(layout.align, 4);
    assert_eq!(
        layout.dynamic_alignment,
        Some(Alignment {
            minimum: 4,
            maximum: 1 << 29
        })
    );
    let fields = layout.fields.ok_or("missing fields")?;
    assert_eq!(
        f.tails.offsets(fields).collect::<Vec<_>>(),
        vec![
            Offset::Fixed(0),
            Offset::Fixed(4),
            Offset::Aligned {
                prefix: 8,
                alignment: Alignment {
                    minimum: 2,
                    maximum: 1 << 29
                }
            },
        ]
    );
    assert_eq!(f.tails.field_offset(fields, 3), None);
    let steps = f.tails.steps;
    for _ in 0..128 {
        assert_eq!(
            f.tails
                .demand(&f.types, &f.definitions, &mut f.layouts, outer)?,
            id
        );
        assert_eq!(f.tails.field_offset(fields, 1), Some(Offset::Fixed(4)));
    }
    assert_eq!(f.tails.steps, steps);
    assert!(f.layouts.get(outer).is_none());
    Ok(())
}

#[test]
fn dynamic_alignment_is_shared_through_deep_transparent_wrappers() -> Result<(), &'static str> {
    let mut f = Fixture::default();
    let mut ty = object(&mut f)?;
    for _ in 0..1024 {
        ty = f.record(
            Repr {
                representation: Representation::Transparent,
                ..Repr::default()
            },
            &[TypeId::UNIT, ty],
        )?;
    }
    let id = f
        .tails
        .demand(&f.types, &f.definitions, &mut f.layouts, ty)?;
    let layout = f.tails.at(id).ok_or("missing transparent layout")?;
    assert_eq!(layout.dynamic_alignment, Some(Alignment::OBJECT));
    assert_eq!(
        f.tails
            .offsets(layout.fields.ok_or("missing fields")?)
            .collect::<Vec<_>>(),
        vec![Offset::Fixed(0), Offset::Fixed(0)]
    );
    assert!(f.tails.steps <= 3 * 1024 + 1);
    Ok(())
}
