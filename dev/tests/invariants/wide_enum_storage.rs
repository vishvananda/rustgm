//! Tag width changes actual byte work, never selected ownership or raw transport rules.
use super::*;
fn source(tag: &str, bits: u32, variants: usize) -> Result<String, String> {
    let mut text = format!("#[repr({tag})]enum E{{");
    for n in 0..variants {
        let value = (1_u128 << (bits - 1)) + (n as u128) * 2 + 1;
        write!(&mut text, "V{n}={value},").map_err(|e| e.to_string())?;
    }
    text.push('}');
    Ok(text)
}
#[test]
fn wide_enum_work_tracks_width_and_log_lookup_not_unrelated_types() -> Result<(), String> {
    for (tag, bits) in [("u16", 16), ("u64", 64), ("u128", 128)] {
        for variants in [8usize, 32, 128] {
            let text = source(tag, bits, variants)?;
            let mut observations = Vec::new();
            for noise in [0, 1024] {
                with_enum_source(&text, noise, |machine, enums| {
                    let (ty, ids) = &enums[0];
                    let before = machine.work.steps;
                    let (_, id) = root(machine, *ty, ids[variants - 1])?;
                    let cold = machine.work.steps - before;
                    let retained = machine.memory.retained_cells();
                    let width = bits as usize / 8;
                    let bits = (1_u128 << (bits - 1)) + ((variants - 1) as u128) * 2 + 1;
                    assert_eq!(
                        machine.byte_backing(id)?.bytes,
                        &bits.to_le_bytes()[..width]
                    );
                    let mut rows = Vec::new();
                    for repeats in [8usize, 32, 128] {
                        let before = machine.work.steps;
                        for _ in 0..repeats {
                            machine.decode_byte_enum(Source::Backing(id), 0, *ty)?;
                        }
                        let work = machine.work.steps - before;
                        assert_eq!(work, repeats * (variants.ilog2() as usize + width + 3));
                        assert_eq!(machine.memory.retained_cells(), retained);
                        rows.push(work);
                        println!(
                            "wide-enum tag={tag} variants={variants} noise={noise} repeats={repeats} cold={cold} warm={work} retained={retained}"
                        );
                    }
                    observations.push((cold, retained, rows));
                    Ok(())
                })?;
            }
            assert_eq!(observations[0], observations[1]);
        }
    }
    assert_eq!(size_of::<(u128, VariantId)>(), 32);
    Ok(())
}
#[test]
fn wide_enum_partial_tag_writes_and_padding_keep_independent_initialization() -> Result<(), String>
{
    with_enum_source(
        "#[repr(u16,align(8))]enum E{A=1,B=4660}",
        0,
        |machine, enums| {
            let (ty, ids) = &enums[0];
            let (root, id) = root(machine, *ty, ids[1])?;
            let backing = machine.byte_backing(id)?;
            assert_eq!(backing.bytes, [52, 18, 0, 0, 0, 0, 0, 0]);
            assert!((0..2).all(|index| backing.initialized(index)));
            assert!((2..8).all(|index| !backing.initialized(index)));
            machine.memory.bytes.roots[id.0 as usize].initialized[0] = 1;
            assert!(
                machine
                    .decode_byte_enum(Source::Backing(id), 0, *ty)
                    .is_err()
            );
            machine.fill_byte_root(root, 1, 1, 18)?;
            assert!(
                machine
                    .decode_byte_enum(Source::Backing(id), 0, *ty)
                    .is_ok()
            );
            machine.fill_byte_root(root, 1, 1, 0)?;
            assert!(
                machine
                    .decode_byte_enum(Source::Backing(id), 0, *ty)
                    .is_err()
            );
            assert!(
                (2..8).all(|index| !machine.memory.bytes.roots[id.0 as usize].initialized(index))
            );
            Ok(())
        },
    )
}
#[test]
fn wide_enum_cache_never_interprets_symbolic_pointer_words_as_tag_numbers() -> Result<(), String> {
    with_enum_source(
        "#[repr(u64)]enum E{A=0,B=7}fn pointer_type(_: *const u8){}",
        0,
        |machine, enums| {
            let (ty, ids) = &enums[0];
            let pointer_ty = (0..machine.evaluator.facts().types.len())
                .filter_map(|n| u32::try_from(n).ok().map(TypeId))
                .find(|&ty| {
                    matches!(
                        machine.evaluator.facts().types.kind(ty),
                        Some(Kind::Pointer { .. })
                    )
                })
                .ok_or("pointer type")?;
            machine.footprint(pointer_ty)?;
            let (root, id) = root(machine, *ty, ids[1])?;
            let pointer = machine.allocate(
                &mut Vec::new(),
                Some(Constant::scalar(crate::sema::model::Scalar::Integer(7))),
                TypeId::primitive(crate::types::Primitive::U8),
            )?;
            let mut buffer = machine.empty_backing(*ty)?;
            machine.encode_byte_word(&mut buffer, 0, pointer_ty, pointer)?;
            let snapshot = buffer.snapshot();
            machine.commit_byte_snapshot(root, 0, &snapshot)?;
            assert!(
                machine
                    .decode_byte_enum(Source::Backing(id), 0, *ty)
                    .is_err()
            );
            machine.fill_byte_root(root, 0, 7, 0)?;
            assert!(
                machine
                    .decode_byte_enum(Source::Backing(id), 0, *ty)
                    .is_err()
            );
            machine.fill_byte_root(root, 0, 8, 0)?;
            machine.fill_byte_root(root, 0, 1, 7)?;
            assert!(
                machine
                    .decode_byte_enum(Source::Backing(id), 0, *ty)
                    .is_ok()
            );
            Ok(())
        },
    )
}
