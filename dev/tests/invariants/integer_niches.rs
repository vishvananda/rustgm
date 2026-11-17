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
/// The canonical raw bits of a signed `..=end | start..`: the two alternatives
/// connect at overflow, so the stored interval runs from `start` up to `end`.
fn wrapped(
    types: &mut Store,
    primitive: Primitive,
    start: u128,
    end: u128,
) -> Result<TypeId, &'static str> {
    let base = TypeId::primitive(primitive);
    let low = 1_u128 << (primitive.integer().ok_or("integer")?.1 - 1);
    let id = types.intern_scalar_or(
        ScalarRange {
            base,
            start: low,
            end,
        },
        ScalarRange {
            base,
            start,
            end: low - 1,
        },
        &|_, _| Ok(()),
    )?;
    types.intern(Kind::Pattern {
        base,
        constraint: Constraint::ScalarRange(id),
    })
}
/// A nullable enum over a transparent wrapper of `leaf` is exactly its integer.
fn zero_carrier(mut types: Store, leaf: TypeId, primitive: Primitive) -> Result<(), String> {
    let bits = primitive.integer().ok_or("integer")?.1;
    let mut definitions = Definitions::default();
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
        Carrier::ZeroExcludedInteger { leaf, primitive }
    );
    assert_eq!(
        (niche.carrier.bytes(), niche.carrier.empty_bits(&types)?),
        (u64::from(bits / 8), 0)
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
    Ok(())
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
        let leaf = patterned(&mut types, primitive, 1, u128::MAX >> (128 - bits))?;
        zero_carrier(types, leaf, primitive)?;
    }
    Ok(())
}
/// `iN is ..=-1 | 1..` stores the same raw interval as `uN is 1..`.
#[test]
fn signed_zero_carriers_exclude_the_same_single_bit_pattern() -> Result<(), String> {
    for primitive in [
        Primitive::I8,
        Primitive::I16,
        Primitive::I32,
        Primitive::I64,
        Primitive::I128,
        Primitive::Isize,
    ] {
        let bits = primitive.integer().ok_or("integer")?.1;
        let mut types = Store::default();
        let leaf = wrapped(&mut types, primitive, 1, u128::MAX >> (128 - bits))?;
        zero_carrier(types, leaf, primitive)?;
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
    let error = Carrier::integer(&foreign, base, TypeId::primitive(Primitive::Char), id, 4)
        .err()
        .ok_or("foreign range base accepted")?;
    assert!(error.contains("different base"));
    for (primitive, start, end, wraps) in [
        (Primitive::U8, 3, 9, false),
        (Primitive::I8, 1, 127, false),
        (Primitive::Char, 1, 127, false),
        // `i32::MIN..=-2` is an ordinary partial range.
        (Primitive::I32, 0x8000_0000, 0xffff_fffe, false),
        // `..=-1 | 2..` excludes zero and one.
        (Primitive::I64, 2, u128::from(u64::MAX), true),
    ] {
        let mut types = Store::default();
        let mut definitions = Definitions::default();
        let leaf = if wraps {
            wrapped(&mut types, primitive, start, end)?
        } else {
            patterned(&mut types, primitive, start, end)?
        };
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

/// A suffix niche retains its logical range and consumes one physical value.
#[test]
fn zero_based_integer_ranges_share_their_unused_suffix() -> Result<(), String> {
    for primitive in [
        Primitive::U8,
        Primitive::U16,
        Primitive::U32,
        Primitive::U64,
        Primitive::U128,
        Primitive::Usize,
        Primitive::I8,
        Primitive::I16,
        Primitive::I32,
        Primitive::I64,
        Primitive::I128,
        Primitive::Isize,
    ] {
        let mut types = Store::default();
        let mut definitions = Definitions::default();
        let leaf = patterned(&mut types, primitive, 0, 41)?;
        let ty = nullable(&mut types, &mut definitions, ItemId(100), leaf, true)?;
        let mut layouts = Layouts::default();
        layouts
            .demand(&types, &definitions, ty)
            .map_err(|e| e.to_string())?;
        let layout = layouts.get(ty).ok_or("bounded layout")?;
        let niche = layouts.niche(layout)?.ok_or("bounded niche")?;
        let mask = u128::MAX >> (128 - primitive.integer().ok_or("integer")?.1);
        assert_eq!(niche.carrier.empty_bits(&types)?, mask);
        assert_eq!(
            layout.size,
            u64::from(primitive.integer().ok_or("integer")?.1 / 8)
        );
        assert!(!niche.carrier.is_pointer());
        let next = niche.carrier.next(&types)?.ok_or("second suffix value")?;
        assert_eq!(next.empty_bits(&types)?, mask - 1);
    }
    for (primitive, wraps) in [(Primitive::U8, false), (Primitive::I8, true)] {
        let mut types = Store::default();
        let leaf = if wraps {
            wrapped(&mut types, primitive, 0, 254)?
        } else {
            patterned(&mut types, primitive, 0, 254)?
        };
        let carrier = Carrier::integer(
            &types,
            leaf,
            TypeId::primitive(primitive),
            match types.kind(leaf).ok_or("pattern")? {
                Kind::Pattern {
                    constraint: Constraint::ScalarRange(id),
                    ..
                } => id,
                _ => return Err("scalar pattern expected".into()),
            },
            1,
        )?
        .ok_or("last suffix value")?;
        assert_eq!(carrier.empty_bits(&types)?, 255);
        assert_eq!(carrier.next(&types)?, None);
    }
    Ok(())
}

/// An independent ordering of every u8 opponent catches alternating boundary
/// choices and exhaustion without repeating the production rank calculation.
#[test]
fn suffix_niches_follow_signed_distance_through_both_boundaries() -> Result<(), String> {
    for end in 0..255_u16 {
        let mut types = Store::default();
        let leaf = patterned(&mut types, Primitive::U8, 0, u128::from(end))?;
        let mut carrier = Some(Carrier::UpperBoundedInteger {
            leaf,
            primitive: Primitive::U8,
            consumed: 0,
        });
        let mut expected: Vec<_> = (end + 1..=255).collect();
        expected.sort_by_key(|&value| {
            let signed = if value < 128 {
                i32::from(value)
            } else {
                i32::from(value) - 256
            };
            (signed.unsigned_abs(), signed > 0)
        });
        if end == 1 {
            let at = expected
                .iter()
                .position(|&value| value == 2)
                .ok_or("bool spare")?;
            expected.remove(at);
            expected.insert(0, 2);
        }
        for bits in expected {
            let current = carrier.ok_or("premature exhaustion")?;
            assert_eq!(current.empty_bits(&types)?, u128::from(bits), "end={end}");
            carrier = current.next(&types)?;
        }
        assert!(carrier.is_none(), "end={end}");
    }
    Ok(())
}
