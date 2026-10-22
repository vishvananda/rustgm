//! Selected range transport preserves word coordinates without weakening typed demands.
use super::*;
use crate::const_eval::static_reads::ReadPurpose;
use crate::sema::values::{ImageFragment, WordTarget};
fn image(machine: &mut Machine<'_, '_>, ty: TypeId) -> Result<Constant, Failure> {
    let mut allocations = Vec::new();
    let pointer = machine.allocate(
        &mut allocations,
        Some(scalar(37)),
        TypeId::primitive(Primitive::U32),
    )?;
    let fragments = [
        ImageFragment {
            offset: 0,
            source_offset: 0,
            length: 8,
            target: WordTarget::Value(pointer),
        },
        ImageFragment {
            offset: 8,
            source_offset: 0,
            length: 8,
            target: WordTarget::Value(pointer.restrict_shared()),
        },
    ];
    machine
        .evaluator
        .byte_array_snapshot(ty, &[7; 16], &[65535], &fragments, 10000)
        .map(|(value, _)| value)
}
fn ready(value: &crate::const_eval::memory::Read<Constant>) -> Result<Constant, Failure> {
    match value {
        crate::const_eval::memory::Read::Value(value) => Ok(*value),
        crate::const_eval::memory::Read::Pending => Err("unexpected pending range".into()),
    }
}
#[test]
fn fragment_ranges_clip_both_ends_and_revalidate_reference_reads() -> Result<(), String> {
    with_array(16, 1 << 20, |machine, ty| {
        let value = image(machine, ty)?;
        let mut allocations = Vec::new();
        let root = machine
            .allocate(&mut allocations, Some(value), ty)?
            .location()
            .ok_or("root")?;
        let range = Range {
            from: 1,
            length: 14,
        };
        let copy = ready(&machine.read_range(root, &[], range, ReadPurpose::Value)?)?;
        let view = Reader::new(machine.evaluator.facts())
            .array_image(copy)?
            .ok_or("range image")?;
        assert_eq!(view.bytes.len(), 14);
        assert_eq!(view.fragments.len(), 2);
        assert_eq!(
            (
                view.fragments[0].offset,
                view.fragments[0].source_offset,
                view.fragments[0].length
            ),
            (0, 1, 7)
        );
        assert_eq!(
            (
                view.fragments[1].offset,
                view.fragments[1].source_offset,
                view.fragments[1].length
            ),
            (7, 0, 7)
        );
        let selected = view.ty;
        assert!(
            machine
                .read_range(root, &[], range, ReadPurpose::ReferenceValidation)
                .is_err()
        );
        let nested = ready(&machine.read_image_path(
            copy,
            selected,
            &[],
            Some(Range {
                from: 2,
                length: 10,
            }),
            ReadPurpose::Value,
        )?)?;
        let view = Reader::new(machine.evaluator.facts())
            .array_image(nested)?
            .ok_or("nested image")?;
        assert_eq!(
            (view.fragments[0].source_offset, view.fragments[0].length),
            (3, 5)
        );
        assert_eq!((view.fragments[1].offset, view.fragments[1].length), (5, 5));
        assert!(
            Reader::new(machine.evaluator.facts())
                .require_publishable_array(nested)
                .is_err()
        );
        assert_eq!(machine.slots, 0);
        Ok(())
    })
}
#[test]
fn selected_fragment_range_work_tracks_lookup_and_extent_not_root_width() -> Result<(), String> {
    for width in [8usize, 32, 128] {
        with_array((width * 8) as u64, 1 << 20, |machine, ty| {
            let mut allocations = Vec::new();
            let pointer = machine.allocate(
                &mut allocations,
                Some(scalar(37)),
                TypeId::primitive(Primitive::U32),
            )?;
            let bytes = vec![7; width * 8];
            let mask = vec![u64::MAX; bytes.len().div_ceil(64)];
            let fragments: Vec<_> = (0..width)
                .map(|index| ImageFragment {
                    offset: (index * 8) as u64,
                    source_offset: 0,
                    length: 8,
                    target: WordTarget::Value(pointer),
                })
                .collect();
            let (value, _) =
                machine
                    .evaluator
                    .byte_array_snapshot(ty, &bytes, &mask, &fragments, 1 << 20)?;
            let range = Some(Range {
                from: ((width - 1) * 8) as u64,
                length: 8,
            });
            ready(&machine.read_image_path(value, ty, &[], range, ReadPurpose::Value)?)?;
            let types = machine.evaluator.facts().types.len();
            for repeats in [8usize, 32, 128] {
                let before = machine.work.steps;
                for _ in 0..repeats {
                    ready(&machine.read_image_path(value, ty, &[], range, ReadPurpose::Value)?)?;
                }
                let work = machine.work.steps - before;
                assert!(
                    work <= repeats * (512 + 128 * width.ilog2() as usize),
                    "width={width} repeats={repeats} work={work}"
                );
                println!("fragment-range width={width} repeats={repeats} work={work}");
            }
            assert_eq!(machine.evaluator.facts().types.len(), types);
            assert_eq!(machine.slots, 0);
            Ok(())
        })?;
    }
    Ok(())
}
#[test]
fn fragment_range_bounds_and_metadata_only_image_checks_are_exact() -> Result<(), String> {
    with_array(16, 1 << 20, |machine, ty| {
        let value = image(machine, ty)?;
        for (from, length) in [(17, 0), (16, 1), (u64::MAX, 2)] {
            assert!(
                machine
                    .evaluator
                    .byte_array_range_type(ty, from, length)
                    .is_err()
            );
        }
        assert!(
            machine
                .evaluator
                .byte_array_range_type(TypeId::UNIT, 0, 0)
                .is_err()
        );
        let empty = machine.evaluator.byte_array_range_type(ty, 16, 0)?;
        assert!(matches!(
            machine.evaluator.facts().types.kind(empty),
            Some(Kind::Array { length: 0, .. })
        ));
        assert!(machine.check_frozen_length(value, ty, &[15]).is_ok());
        assert!(machine.check_frozen_length(value, ty, &[16]).is_err());
        assert!(
            machine
                .check_frozen_length(value, TypeId::UNIT, &[0])
                .is_err()
        );
        assert!(
            machine
                .decode_source_range(
                    super::super::continuations::Source::Image(value),
                    15,
                    ty,
                    Range { from: 0, length: 8 },
                    ReadPurpose::Value
                )
                .is_err()
        );
        Ok(())
    })
}
#[test]
fn fragment_range_assignment_updates_original_backing_once() -> Result<(), String> {
    with_array(16, 1 << 20, |machine, ty| {
        let value = image(machine, ty)?;
        let child = machine.evaluator.byte_array_range_type(ty, 0, 8)?;
        let copy = ready(&machine.read_image_path(
            value,
            ty,
            &[],
            Some(Range { from: 0, length: 8 }),
            ReadPurpose::Value,
        )?)?;
        let zero = machine.evaluator.repeat(scalar(0), 16)?;
        let mut allocations = Vec::new();
        let destination = machine.allocate(&mut allocations, Some(zero), ty)?;
        let address = machine.evaluator.subsequence(destination, 4, 8, child)?;
        machine.write(address, copy)?;
        let root = destination.location().ok_or("destination")?;
        assert!(machine.has_byte_root(root));
        let copy = ready(&machine.read_range(
            root,
            &[],
            Range { from: 4, length: 8 },
            ReadPurpose::Value,
        )?)?;
        let view = Reader::new(machine.evaluator.facts())
            .array_image(copy)?
            .ok_or("stored range")?;
        assert_eq!(view.fragments.len(), 1);
        assert_eq!(view.fragments[0].source_offset, 0);
        assert!(ready_cell(machine, root, &[0])? == scalar(0));
        assert!(ready_cell(machine, root, &[12])? == scalar(0));
        assert!(
            machine
                .write_range(
                    root,
                    &[],
                    Range {
                        from: 15,
                        length: 8
                    },
                    copy
                )
                .is_err()
        );
        assert_eq!(machine.slots, 0);
        Ok(())
    })
}
