//! Opaque array copies preserve fragments; integer and reference demands do not.
use super::*;
use crate::const_eval::static_reads::ReadPurpose;
use crate::sema::values::{ImageFragment, WordTarget};

#[test]
fn array_images_preserve_fragment_restrictions_and_numeric_selection() -> Result<(), String> {
    with_array(16, 1 << 20, |machine, ty| {
        let mut allocations = Vec::new();
        let leaf = machine.allocate(
            &mut allocations,
            Some(scalar(37)),
            TypeId::primitive(Primitive::U32),
        )?;
        let fragments = [ImageFragment {
            offset: 0,
            source_offset: 0,
            length: 8,
            target: WordTarget::Value(leaf.restrict_shared()),
        }];
        assert!(
            machine
                .evaluator
                .byte_array_snapshot(ty, &[7; 16], &[65535], &fragments, 0)
                .is_err()
        );
        let (value, _) =
            machine
                .evaluator
                .byte_array_snapshot(ty, &[7; 16], &[65535], &fragments, 10000)?;
        let reader = Reader::new(machine.evaluator.facts());
        assert!(reader.require_publishable_array(value).is_err());
        assert!(reader.array_image_field(value, 0).is_err());
        assert!(reader.array_image_field(value, 7).is_err());
        assert!(reader.array_image_field(value, 16).is_err());
        assert!(reader.array_image_field(value, 8)? == scalar(7));
        assert!(machine.evaluator.field(value, 8)? == scalar(7));
        let root = machine.allocate(&mut allocations, Some(value), ty)?;
        let root = root.location().ok_or("array root")?;
        assert!(ready_cell(machine, root, &[])?.location().is_none());
        // A frozen/cache hit must not reuse the permissive whole-value transfer path.
        assert!(
            machine
                .read_cell(root, &[], ReadPurpose::ReferenceValidation)
                .is_err()
        );
        assert!(
            matches!(machine.read_cell(root, &[8], ReadPurpose::ReferenceValidation)?, crate::const_eval::memory::Read::Value(v) if v == scalar(7))
        );
        let id = machine.convert_byte_root(root)?;
        let value = match machine.decode_bytes(id, 0, ty, ReadPurpose::Value)? {
            super::super::super::Read::Value(value) => value,
            super::super::super::Read::Pending => {
                return Err("unexpected pending byte array".into());
            }
        };
        let image = Reader::new(machine.evaluator.facts())
            .array_image(value)?
            .ok_or("array image")?;
        assert_eq!(image.fragments.len(), 1);
        let WordTarget::Value(pointer) = image.fragments[0].target else {
            return Err("pointer word".into());
        };
        assert_eq!(pointer.location(), leaf.location());
        assert_eq!(
            pointer.write_permission(),
            leaf.restrict_shared().write_permission()
        );
        assert!(
            machine
                .decode_bytes(id, 0, ty, ReadPurpose::ReferenceValidation)
                .is_err()
        );
        assert_eq!(machine.slots, 0);
        Ok(())
    })
}

#[test]
fn array_snapshot_extent_initialization_and_union_limits_stay_distinct() -> Result<(), String> {
    with_array(16, 1 << 20, |machine, ty| {
        assert!(
            machine
                .evaluator
                .byte_array_snapshot(ty, &[7; 16], &[1], &[], 10000)
                .is_err()
        );
        assert!(
            machine
                .evaluator
                .byte_array_snapshot(ty, &[7; 15], &[65535], &[], 10000)
                .is_err()
        );
        assert!(
            machine
                .evaluator
                .byte_array_snapshot(TypeId::UNIT, &[7; 16], &[65535], &[], 10000)
                .is_err()
        );
        let (value, _) =
            machine
                .evaluator
                .byte_array_snapshot(ty, &[7; 16], &[65535], &[], 10000)?;
        Reader::new(machine.evaluator.facts()).require_publishable_array(value)?;
        Ok(())
    })?;
    // The pre-existing union publication limit must not cap ordinary byte arrays.
    with_array(65544, 1 << 20, |machine, ty| {
        let bytes = vec![7; 65544];
        let initialized = vec![u64::MAX; bytes.len().div_ceil(64)];
        let (value, work) =
            machine
                .evaluator
                .byte_array_snapshot(ty, &bytes, &initialized, &[], 1 << 20)?;
        assert!(work <= 3 * bytes.len());
        assert_eq!(
            Reader::new(machine.evaluator.facts())
                .array_image(value)?
                .ok_or("large array image")?
                .bytes
                .len(),
            bytes.len()
        );
        Ok(())
    })
}

#[test]
fn array_snapshot_work_is_linear_and_selected_reads_are_logarithmic() -> Result<(), String> {
    for width in [8usize, 32, 128] {
        with_array((width * 8 + 8) as u64, 1 << 20, |machine, ty| {
            let mut allocations = Vec::new();
            let pointer = machine.allocate(
                &mut allocations,
                Some(scalar(37)),
                TypeId::primitive(Primitive::U32),
            )?;
            let bytes = vec![7; width * 8 + 8];
            let initialized = vec![u64::MAX; bytes.len().div_ceil(64)];
            let fragments: Vec<_> = (0..width)
                .map(|index| ImageFragment {
                    offset: (index * 8) as u64,
                    source_offset: 0,
                    length: 8,
                    target: WordTarget::Value(pointer),
                })
                .collect();
            let (value, work) = machine.evaluator.byte_array_snapshot(
                ty,
                &bytes,
                &initialized,
                &fragments,
                1 << 20,
            )?;
            assert!(work <= 3 * bytes.len() + 16 * fragments.len() + 64);
            let bound = width.ilog2() as usize + 2;
            for repeats in [8usize, 32, 128] {
                let before = machine.work.steps;
                for _ in 0..repeats {
                    assert!(machine.stored_field(value, (width * 8) as u64)? == scalar(7));
                }
                assert_eq!(machine.work.steps - before, repeats * bound);
            }
            println!("array width={width} snapshot_work={work} selected_work={bound}");
            Ok(())
        })?;
    }
    Ok(())
}
