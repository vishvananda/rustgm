//! Scalar carriers preserve logical refinements while changing only the nullable encoding.
use super::*;
use crate::types::{Constraint, Primitive, ScalarRange};
fn patterned(
    types: &mut Store,
    primitive: Primitive,
    start: u128,
    end: u128,
) -> Result<TypeId, &'static str> {
    let base = TypeId::primitive(primitive);
    let id = types.intern_scalar_range(ScalarRange { base, start, end }, &|_, _| Ok(()))?;
    types.intern(Kind::Pattern {
        base,
        constraint: Constraint::ScalarRange(id),
    })
}
#[test]
fn unsigned_zero_carriers_preserve_all_widths_without_pointer_permissions() -> Result<(), String> {
    for primitive in [
        Primitive::U8,
        Primitive::U16,
        Primitive::U32,
        Primitive::U64,
        Primitive::U128,
        Primitive::Usize,
    ] {
        let bits = primitive.integer().ok_or("integer")?.1;
        let mut types = Store::default();
        let mut definitions = Definitions::default();
        let leaf = patterned(&mut types, primitive, 1, u128::MAX >> (128 - bits))?;
        let child = wrapper(&mut types, &mut definitions, ItemId(100), leaf)?;
        let ty = nullable(&mut types, &mut definitions, ItemId(101), child, true)?;
        let mut layouts = Layouts::default();
        layouts
            .demand(&types, &definitions, ty)
            .map_err(|e| e.to_string())?;
        let layout = layouts.get(ty).ok_or("layout")?;
        assert_eq!(
            (layout.size, u64::from(layout.align)),
            (u64::from(bits / 8), u64::from(bits / 8))
        );
        let niche = *layouts.niche(layout)?.ok_or("niche")?;
        assert_eq!(
            niche.carrier,
            Carrier::ZeroExcludedUnsigned { leaf, primitive }
        );
        assert_eq!(layouts.pointer_carrier(&types, &definitions, child)?, None);
        assert_eq!(layouts.retained_pointer_carrier(child), None);
        let bytes = layouts.niches.as_ref().ok_or("cache")?.retained();
        let start = layouts.steps;
        for _ in 0..128 {
            assert_eq!(
                layouts.scalar_carrier(&types, &definitions, child)?,
                Some(niche.carrier)
            );
        }
        assert_eq!(layouts.steps - start, 128);
        assert_eq!(layouts.niches.as_ref().ok_or("cache")?.retained(), bytes);
        println!(
            "{primitive:?} carrier={} answer={} niche={} table={} retained={bytes} warm_work=128",
            size_of::<Carrier>(),
            size_of::<super::super::Answer>(),
            size_of::<Niche>(),
            size_of::<super::super::Niches>()
        );
    }
    Ok(())
}
#[test]
fn partial_ranges_and_failed_layouts_never_publish_a_nullable_encoding() -> Result<(), String> {
    let mut foreign = Store::default();
    let base = TypeId::primitive(Primitive::U8);
    let id = foreign.intern_scalar_range(
        ScalarRange {
            base,
            start: 1,
            end: 255,
        },
        &|_, _| Ok(()),
    )?;
    let error = Carrier::unsigned(&foreign, base, TypeId::primitive(Primitive::Char), id, 4)
        .err()
        .ok_or("foreign range base accepted")?;
    assert!(error.contains("different base"));
    for (primitive, start, end) in [
        (Primitive::U8, 3, 9),
        (Primitive::U8, 0, 254),
        (Primitive::I8, 1, 127),
        (Primitive::Char, 1, 127),
    ] {
        let mut types = Store::default();
        let mut definitions = Definitions::default();
        let leaf = patterned(&mut types, primitive, start, end)?;
        let ty = nullable(&mut types, &mut definitions, ItemId(100), leaf, true)?;
        let mut layouts = Layouts::default();
        let error = layouts
            .demand(&types, &definitions, ty)
            .err()
            .ok_or("unsupported niche admitted")?
            .to_string();
        assert!(error.contains("complete ABI profile"), "{error}");
        assert!(layouts.get(ty).is_none());
        assert_eq!(layouts.niche_mark(), 0);
    }
    for limit in 0..10 {
        let mut types = Store::default();
        let mut definitions = Definitions::default();
        let leaf = patterned(&mut types, Primitive::U8, 1, 255)?;
        let ty = nullable(&mut types, &mut definitions, ItemId(100), leaf, true)?;
        let mut layouts = Layouts::default();
        assert!(
            layouts
                .demand_bounded(&types, &definitions, ty, limit)
                .is_err()
        );
        assert!(layouts.get(ty).is_none());
        assert_eq!(layouts.niche_mark(), 0);
        layouts
            .demand(&types, &definitions, ty)
            .map_err(|e| e.to_string())?;
        assert_eq!(layouts.niche_mark(), 1);
    }
    assert_eq!(
        (
            size_of::<Layout>(),
            size_of::<Shape>(),
            size_of::<Layouts>()
        ),
        (48, 32, 152)
    );
    Ok(())
}
