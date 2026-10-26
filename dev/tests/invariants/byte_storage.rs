//! One backing preserves aliases, masks and per-pointer authority after conversion.
#[path = "array_fragments.rs"]
mod array_fragments;
#[path = "cached_places.rs"]
mod cached_places;
#[path = "captured_memory.rs"]
mod captures;
#[path = "fragment_ranges.rs"]
mod fragment_ranges;
#[path = "hidden_memory.rs"]
mod opaques;
#[path = "string_memory.rs"]
mod strings;
use super::{Cell, Constant, Failure, Machine, Reader, Snapshot};
use crate::{
    const_eval::{
        Limits,
        memory::{Range, Resolved},
    },
    sema::{self, model::Scalar, values::Evaluator},
    support::intrinsics::Intrinsic,
    test_source,
    types::{Kind, Primitive, TypeId},
};
fn with_array(
    length: u64,
    slots: usize,
    check: impl FnOnce(&mut Machine<'_, '_>, TypeId) -> Result<(), Failure>,
) -> Result<(), String> {
    with_storage(
        slots,
        |types| {
            Ok(types.intern(Kind::Array {
                element: TypeId::primitive(Primitive::U8),
                length,
            })?)
        },
        check,
    )
}
fn with_storage(
    slots: usize,
    make: impl FnOnce(&mut crate::types::Store) -> Result<TypeId, String>,
    check: impl FnOnce(&mut Machine<'_, '_>, TypeId) -> Result<(), Failure>,
) -> Result<(), String> {
    let report = test_source::resolve("", true)?;
    let mut facts = sema::declarations::analyze(&report).map_err(|e| e.to_string())?;
    let ty = make(&mut facts.types)?;
    let evaluator = Evaluator::new(&report, &mut facts);
    let mut machine = Machine::new(
        evaluator,
        std::iter::empty(),
        Limits {
            slots,
            ..Limits::default()
        },
    )?;
    check(&mut machine, ty).map_err(|error| error.message().to_owned())
}
fn ready_cell(
    machine: &mut Machine<'_, '_>,
    root: crate::sema::values::LocationId,
    path: &[u64],
) -> Result<Constant, Failure> {
    match machine.read_cell(
        root,
        path,
        crate::const_eval::static_reads::ReadPurpose::Value,
    )? {
        super::super::Read::Value(value) => Ok(value),
        super::super::Read::Pending => Err("private scalar read unexpectedly pending".into()),
    }
}
fn scalar(value: u128) -> Constant {
    Constant::scalar(Scalar::Integer(value))
}
#[test]
fn symbolic_shape_cache_warm_hits_need_no_new_storage_headroom() -> Result<(), String> {
    with_storage(
        1 << 20,
        |types| {
            Ok(types.intern(Kind::Reference {
                mutable: false,
                region: crate::types::RegionId(7),
                referent: TypeId::primitive(Primitive::U32),
            })?)
        },
        |machine, ty| {
            let key = machine.erased_word_type(ty)?;
            let retained = machine.memory.retained_cells();
            machine.limits.slots = retained;
            let before = machine.work.steps;
            for _ in 0..128 {
                assert_eq!(machine.erased_word_type(ty)?, key);
            }
            assert_eq!(machine.work.steps - before, 128);
            assert_eq!(machine.memory.retained_cells(), retained);
            assert_eq!(machine.slots, 0);
            Ok(())
        },
    )
}
#[test]
fn symbolic_selected_span_work_tracks_log_lookup_and_intersections() -> Result<(), String> {
    for width in [8usize, 32, 128] {
        let mut rows = Vec::new();
        for repeats in [8usize, 32, 128] {
            with_storage(
                1 << 20,
                |types| {
                    let pointer = types.intern(Kind::Pointer {
                        mutable: false,
                        pointee: TypeId::primitive(Primitive::U32),
                    })?;
                    Ok(types.intern(Kind::Array {
                        element: pointer,
                        length: width as u64,
                    })?)
                },
                |machine, ty| {
                    let mut allocations = Vec::new();
                    let leaf = machine.allocate(
                        &mut allocations,
                        Some(scalar(37)),
                        TypeId::primitive(Primitive::U32),
                    )?;
                    let values = machine.evaluator.repeat(leaf, width as u64)?;
                    let source = machine
                        .allocate(&mut allocations, Some(values), ty)?
                        .location()
                        .ok_or("span source")?;
                    let target = machine
                        .allocate(&mut allocations, Some(values), ty)?
                        .location()
                        .ok_or("span target")?;
                    machine.convert_byte_root(source)?;
                    machine.convert_byte_root(target)?;
                    let retained = machine.memory.retained_cells();
                    let before = machine.work.steps;
                    for _ in 0..repeats {
                        let snapshot = machine
                            .snapshot_byte_root(source, (width as u64 - 1) * 8, 8)?
                            .ok_or("selected symbolic snapshot")?;
                        assert_eq!(snapshot.fragments.as_ref().map_or(0, |tree| tree.len()), 1);
                        let previous = machine.slots;
                        machine.slots += snapshot.retained_slots();
                        let result = machine.commit_byte_snapshot(target, 0, &snapshot);
                        machine.slots = previous;
                        result?;
                    }
                    assert_eq!(machine.memory.retained_cells(), retained);
                    let work = machine.work.steps - before;
                    let ceiling =
                        repeats * (512 + 128 * (usize::BITS - width.leading_zeros()) as usize);
                    assert!(
                        work <= ceiling,
                        "width={width} repeats={repeats} work={work}"
                    );
                    rows.push(work);
                    println!(
                        "fragment-span width={width} repeats={repeats} steps={work} retained={retained}"
                    );
                    Ok(())
                },
            )?;
        }
        assert_eq!(rows[2] - rows[1], 4 * (rows[1] - rows[0]));
    }
    Ok(())
}
#[test]
fn byte_backing_clears_mask_without_typed_interpretation() -> Result<(), String> {
    with_array(4, 1 << 20, |machine, ty| {
        let mut allocations = Vec::new();
        let value = machine.evaluator.repeat(scalar(9), 4)?;
        let source = machine.allocate(&mut allocations, Some(value), ty)?;
        let target = machine.allocate(&mut allocations, Some(value), ty)?;
        let root = source.location().ok_or("source root")?;
        machine.commit_byte_snapshot(
            root,
            0,
            &Snapshot {
                bytes: vec![7, 0, 0, 0],
                initialized: vec![1],
                fragments: None,
            },
        )?;
        assert!(ready_cell(machine, root, &[0])? == scalar(7));
        assert!(ready_cell(machine, root, &[1]).is_err());
        assert!(
            machine
                .counted_copy(
                    Intrinsic::CopyNonoverlapping,
                    ty,
                    &[source, target, scalar(1)]
                )?
                .is_some()
        );
        let destination = target.location().ok_or("target root")?;
        assert!(ready_cell(machine, destination, &[0])? == scalar(7));
        assert!(ready_cell(machine, destination, &[1]).is_err());
        assert_eq!(
            machine.sequence_length(Resolved {
                address: target,
                ty,
                raw: None,
                union: None
            })?,
            4
        );
        assert_eq!(machine.slots, 0);
        Ok(())
    })
}
#[test]
fn byte_backing_aliases_overlap_and_ordinary_range_writes() -> Result<(), String> {
    with_array(5, 1 << 20, |machine, ty| {
        let mut allocations = Vec::new();
        let value = machine.evaluator.fields(
            &[scalar(1), scalar(2), scalar(3), scalar(4), scalar(5)],
            None,
        )?;
        let address = machine.allocate(&mut allocations, Some(value), ty)?;
        let root = address.location().ok_or("root")?;
        machine.convert_byte_root(root)?;
        let snapshot = machine.snapshot_byte_root(root, 0, 4)?.ok_or("snapshot")?;
        machine.commit_byte_snapshot(root, 1, &snapshot)?;
        for (index, expected) in [1, 1, 2, 3, 4].into_iter().enumerate() {
            assert!(ready_cell(machine, root, &[index as u64])? == scalar(expected));
        }
        let replacement = machine.evaluator.fields(&[scalar(11), scalar(13)], None)?;
        machine.write_range(root, &[], Range { from: 1, length: 2 }, replacement)?;
        assert!(ready_cell(machine, root, &[1])? == scalar(11));
        assert!(ready_cell(machine, root, &[2])? == scalar(13));
        let element = machine
            .evaluator
            .projection(address, 2, TypeId::primitive(Primitive::U8))?;
        machine.write(element, scalar(17))?;
        assert!(ready_cell(machine, root, &[2])? == scalar(17));
        assert_eq!(machine.memory.bytes.roots.len(), 1);
        let &cell = machine.memory.roots.get(&root).ok_or("root cell")?;
        assert!(matches!(machine.memory.cells[cell], Cell::Bytes(_)));
        println!(
            "Constant={} Cell={} Backing={} Snapshot={} WordTarget={} ImageFragment={} ImageView={} Fragment={}",
            size_of::<Constant>(),
            size_of::<Cell>(),
            size_of::<super::Backing>(),
            size_of::<Snapshot>(),
            size_of::<crate::sema::values::WordTarget>(),
            size_of::<crate::sema::values::ImageFragment>(),
            size_of::<crate::sema::values::ImageView<'_>>(),
            size_of::<super::relocations::Fragment>()
        );
        Ok(())
    })
}
#[test]
fn byte_backing_keeps_pointer_permission_and_expiry_checks() -> Result<(), String> {
    with_array(4, 1 << 20, |machine, ty| {
        let mut allocations = Vec::new();
        let value = machine.evaluator.repeat(scalar(7), 4)?;
        let source = machine.allocate(&mut allocations, Some(value), ty)?;
        let target = machine.allocate(&mut allocations, Some(value), ty)?;
        machine.convert_byte_root(source.location().ok_or("root")?)?;
        assert!(
            machine
                .counted_copy(
                    Intrinsic::CopyNonoverlapping,
                    ty,
                    &[source, target.restrict_shared(), scalar(1)]
                )
                .is_err()
        );
        assert!(
            machine
                .counted_copy(
                    Intrinsic::CopyNonoverlapping,
                    ty,
                    &[source, target, scalar(1)]
                )?
                .is_some()
        );
        assert_eq!(
            target.write_permission(),
            Some(crate::sema::values::WritePermission::Writable)
        );
        machine.memory.expire(&allocations);
        assert!(
            machine
                .counted_copy(Intrinsic::Copy, ty, &[source, target, scalar(1)])
                .is_err()
        );
        assert_eq!(machine.slots, 0);
        Ok(())
    })
}
#[test]
fn byte_backing_budget_failure_keeps_original_typed_root() -> Result<(), String> {
    with_array(4096, 32, |machine, ty| {
        let mut allocations = Vec::new();
        let value = machine.evaluator.repeat(scalar(7), 4096)?;
        let address = machine.allocate(&mut allocations, Some(value), ty)?;
        let root = address.location().ok_or("root")?;
        assert!(machine.convert_byte_root(root).is_err());
        assert!(machine.memory.bytes.roots.is_empty());
        assert!(ready_cell(machine, root, &[0])? == scalar(7));
        assert_eq!(
            Reader::new(machine.evaluator.facts()).location(root)?.ty,
            ty
        );
        assert_eq!(machine.slots, 0);
        Ok(())
    })
}
#[test]
fn symbolic_interval_edits_preserve_order_and_remove_only_overwritten_bytes() -> Result<(), String>
{
    with_array(16, 1 << 20, |machine, ty| {
        use super::relocations::{Fragment, Fragments};
        use crate::sema::values::WordTarget;
        let mut allocations = Vec::new();
        let value = machine.evaluator.repeat(scalar(7), 16)?;
        let address = machine.allocate(&mut allocations, Some(value), ty)?;
        let word = machine.intern_word(WordTarget::Value(address))?;
        let other = machine.intern_word(WordTarget::Value(address.restrict_shared()))?;
        assert!(word != other);
        assert!(word == machine.intern_word(WordTarget::Value(address))?);
        let mut tree: Option<Box<Fragments>> = None;
        let low = Fragment {
            offset: 0,
            source_offset: 0,
            length: 4,
            word,
        };
        let high = Fragment {
            offset: 0,
            source_offset: 4,
            length: 4,
            word,
        };
        machine.edit_fragments(&mut tree, 4, 4, &[high])?;
        machine.edit_fragments(&mut tree, 0, 4, &[low])?;
        let complete = tree.as_ref().ok_or("missing fragments")?;
        assert_eq!(complete.len(), 1);
        let fragment = complete.get(&0).ok_or("missing coalesced word")?;
        assert_eq!(fragment.length, 8);
        let clipped = machine.clip_fragments(complete, 2, 4)?;
        assert_eq!(clipped.len(), 1);
        assert_eq!(clipped[0].offset, 0);
        assert_eq!(clipped[0].source_offset, 2);
        assert_eq!(clipped[0].length, 4);
        machine.edit_fragments(&mut tree, 3, 2, &[])?;
        let split = tree.as_ref().ok_or("missing split fragments")?;
        assert_eq!(split.len(), 2);
        assert_eq!(split.get(&0).ok_or("prefix")?.length, 3);
        assert_eq!(split.get(&5).ok_or("suffix")?.source_offset, 5);
        let malformed = Fragment {
            offset: 0,
            source_offset: 7,
            length: 2,
            word: other,
        };
        assert!(
            machine
                .edit_fragments(&mut tree, 0, 2, &[malformed])
                .is_err()
        );
        let unchanged = tree.as_ref().ok_or("missing unchanged fragments")?;
        assert_eq!(unchanged.len(), 2);
        assert!(unchanged.get(&0).ok_or("unchanged prefix")?.word == word);
        assert_eq!(unchanged.get(&0).ok_or("unchanged prefix")?.length, 3);
        assert_eq!(machine.slots, 0);
        Ok(())
    })
}
#[test]
fn symbolic_word_address_is_independent_of_equivalent_projection_views() -> Result<(), String> {
    with_array(16, 1 << 20, |machine, ty| {
        use crate::sema::values::{WordTarget, WritePermission};
        let mut allocations = Vec::new();
        let value = machine.evaluator.repeat(scalar(7), 16)?;
        let root = machine.allocate(&mut allocations, Some(value), ty)?;
        let element = TypeId::primitive(Primitive::U8);
        let first = machine.evaluator.projection(root, 0, element)?;
        let other_first = machine.evaluator.projection(root, 0, element)?;
        let second = machine.evaluator.projection(root, 1, element)?;
        for (a, b) in [(root, first), (first, root), (first, other_first)] {
            let a = machine.intern_word(WordTarget::Value(a))?;
            let b = machine.intern_word(WordTarget::Value(b))?;
            assert!(a == b);
            let address = machine.word_address(a)?;
            assert_eq!(address.offset(), 0);
            assert_eq!(address.root(), root.location().ok_or("root")?);
            let decoded = machine.reconstruct_word(address, element)?;
            assert_eq!(decoded.write_permission(), Some(WritePermission::Writable));
            assert!(
                machine
                    .read(Resolved {
                        address: decoded,
                        ty: element,
                        raw: None,
                        union: None
                    })?
                    .is_some_and(|value| value == scalar(7))
            );
        }
        let first_word = machine.intern_word(WordTarget::Value(first))?;
        let second_word = machine.intern_word(WordTarget::Value(second))?;
        assert!(first_word != second_word);
        let shared_word = machine.intern_word(WordTarget::Value(first.restrict_shared()))?;
        assert!(shared_word != first_word);
        let shared =
            machine
                .evaluator
                .word_projection(machine.word_address(shared_word)?, &[0], element)?;
        assert_eq!(
            shared.write_permission(),
            Some(WritePermission::SharedDerived)
        );
        assert!(
            machine
                .evaluator
                .word_projection(machine.word_address(first_word)?, &[1], element)
                .is_err()
        );
        assert!(
            machine
                .evaluator
                .word_projection(machine.word_address(first_word)?, &[], element)
                .is_err()
        );
        let decoded =
            machine
                .evaluator
                .word_projection(machine.word_address(first_word)?, &[0], element)?;
        machine.memory.expire(&allocations);
        assert!(
            machine
                .read(Resolved {
                    address: decoded,
                    ty: element,
                    raw: None,
                    union: None
                })
                .is_err()
        );
        Ok(())
    })
}
#[test]
fn symbolic_subobject_lookup_is_indexed_and_retains_coincident_ambiguity() -> Result<(), String> {
    for width in [8usize, 32, 128] {
        with_storage(
            1 << 20,
            |types| {
                let fields = types.intern_list(&vec![TypeId::primitive(Primitive::U8); width])?;
                Ok(types.intern(Kind::Tuple(fields))?)
            },
            |machine, ty| {
                use crate::sema::values::WordTarget;
                let mut allocations = Vec::new();
                let fields = vec![scalar(7); width];
                let value = machine.evaluator.fields(&fields, None)?;
                let root = machine.allocate(&mut allocations, Some(value), ty)?;
                let element = TypeId::primitive(Primitive::U8);
                let mut measured = Vec::new();
                for index in [0, width / 2, width - 1] {
                    let pointer = machine.evaluator.projection(root, index as u64, element)?;
                    let word = machine.intern_word(WordTarget::Value(pointer))?;
                    let before = machine.work.steps;
                    let rebuilt = machine.reconstruct_word(machine.word_address(word)?, element)?;
                    measured.push(machine.work.steps - before);
                    assert!(
                        machine
                            .read(Resolved {
                                address: rebuilt,
                                ty: element,
                                raw: None,
                                union: None
                            })?
                            .is_some_and(|value| value == scalar(7))
                    );
                }
                // First query builds the family; later distinct offsets use its interval index.
                let logarithmic = 128 + 32 * (usize::BITS - width.leading_zeros()) as usize;
                assert!(
                    measured[1] <= logarithmic,
                    "width={width} work={measured:?}"
                );
                assert!(
                    measured[2] <= logarithmic,
                    "width={width} work={measured:?}"
                );
                Ok(())
            },
        )?;
    }
    with_storage(
        1 << 20,
        |types| {
            let fields = types.intern_list(&[TypeId::UNIT, TypeId::UNIT])?;
            Ok(types.intern(Kind::Tuple(fields))?)
        },
        |machine, ty| {
            use crate::sema::values::WordTarget;
            let mut allocations = Vec::new();
            let unit = machine.evaluator.fields(&[], None)?;
            let value = machine.evaluator.fields(&[unit, unit], None)?;
            let root = machine.allocate(&mut allocations, Some(value), ty)?;
            let first = machine.evaluator.projection(root, 0, TypeId::UNIT)?;
            let word = machine.intern_word(WordTarget::Value(first))?;
            let error = machine
                .reconstruct_word(machine.word_address(word)?, TypeId::UNIT)
                .err()
                .ok_or("coincident ZST ambiguity was accepted")?;
            assert_eq!(
                error.message(),
                "symbolic pointer selects ambiguous coincident subobjects"
            );
            Ok(())
        },
    )
}
#[test]
fn symbolic_pointer_bytes_survive_split_copies_without_numeric_exposure() -> Result<(), String> {
    with_storage(
        1 << 20,
        |types| {
            Ok(types.intern(Kind::Pointer {
                mutable: true,
                pointee: TypeId::primitive(Primitive::U8),
            })?)
        },
        |machine, ty| {
            let mut allocations = Vec::new();
            let leaf = machine.allocate(
                &mut allocations,
                Some(scalar(37)),
                TypeId::primitive(Primitive::U8),
            )?;
            let first = machine.allocate(&mut allocations, Some(leaf), ty)?;
            let second = machine.allocate(&mut allocations, Some(leaf), ty)?;
            let target = machine.allocate(&mut allocations, Some(leaf), ty)?;
            let first = first.location().ok_or("first pointer slot")?;
            let second = second.location().ok_or("second pointer slot")?;
            let target = target.location().ok_or("target pointer slot")?;
            machine.convert_byte_root(first)?;
            machine.convert_byte_root(second)?;
            machine.commit_byte_snapshot(
                target,
                0,
                &Snapshot {
                    bytes: vec![0; 8],
                    initialized: vec![255],
                    fragments: None,
                },
            )?;
            let upper = machine
                .snapshot_byte_root(second, 4, 4)?
                .ok_or("upper word snapshot")?;
            let lower = machine
                .snapshot_byte_root(first, 0, 4)?
                .ok_or("lower word snapshot")?;
            machine.commit_byte_snapshot(target, 4, &upper)?;
            assert!(ready_cell(machine, target, &[]).is_err());
            machine.commit_byte_snapshot(target, 0, &lower)?;
            let rebuilt = ready_cell(machine, target, &[])?;
            assert_eq!(rebuilt.write_permission(), leaf.write_permission());
            assert!(
                machine
                    .read(Resolved {
                        address: rebuilt,
                        ty: TypeId::primitive(Primitive::U8),
                        raw: None,
                        union: None
                    })?
                    .is_some_and(|value| value == scalar(37))
            );
            let backing = machine.byte_root(target).ok_or("target backing")?;
            assert!(
                machine
                    .decode_bytes(
                        backing,
                        0,
                        TypeId::primitive(Primitive::U8),
                        crate::const_eval::static_reads::ReadPurpose::Value
                    )
                    .is_err()
            );
            machine.commit_byte_snapshot(
                target,
                3,
                &Snapshot {
                    bytes: vec![0],
                    initialized: vec![1],
                    fragments: None,
                },
            )?;
            assert!(ready_cell(machine, target, &[]).is_err());
            assert_eq!(machine.slots, 0);
            Ok(())
        },
    )
}
fn with_static_references(
    width: usize,
    check: impl FnOnce(
        &mut Machine<'_, '_>,
        TypeId,
        &[crate::resolve::model::DefId],
    ) -> Result<(), Failure>,
) -> Result<(), String> {
    with_reference_storage(width, false, |machine, reference, _, definitions| {
        check(machine, reference, definitions)
    })
}
fn with_reference_storage(
    width: usize,
    raw_source: bool,
    check: impl FnOnce(
        &mut Machine<'_, '_>,
        TypeId,
        TypeId,
        &[crate::resolve::model::DefId],
    ) -> Result<(), Failure>,
) -> Result<(), String> {
    use std::fmt::Write;
    let mut source = String::new();
    for index in 0..width {
        write!(&mut source, "static S{index}:u32={};", index + 11)
            .map_err(|error| error.to_string())?;
    }
    let report = test_source::resolve(&source, true)?;
    let mut facts = sema::declarations::analyze(&report).map_err(|error| error.to_string())?;
    let definitions = facts
        .initializers
        .iter()
        .map(|initializer| initializer.definition)
        .collect::<Vec<_>>();
    assert_eq!(definitions.len(), width);
    let referent = TypeId::primitive(Primitive::U32);
    let reference = facts.types.intern(Kind::Reference {
        mutable: false,
        region: crate::types::RegionId::STATIC,
        referent,
    })?;
    let array = facts.types.intern(Kind::Array {
        element: reference,
        length: width as u64,
    })?;
    let raw = if raw_source {
        let element = facts.types.intern(Kind::Pointer {
            mutable: false,
            pointee: referent,
        })?;
        facts.types.intern(Kind::Array {
            element,
            length: width as u64,
        })?
    } else {
        array
    };
    let evaluator = Evaluator::new(&report, &mut facts);
    let mut machine = Machine::new(evaluator, std::iter::empty(), Limits::default())?;
    check(&mut machine, array, raw, &definitions).map_err(|error| error.message().to_owned())
}
#[test]
fn symbolic_reference_duties_resume_each_static_once_and_do_not_cache_later_reads()
-> Result<(), String> {
    for width in [8usize, 32, 128] {
        with_static_references(width, |machine, ty, definitions| {
            use crate::const_eval::{ReadOperation, memory::Read};
            let mut allocations = Vec::new();
            let mut references = Vec::new();
            for &definition in definitions {
                assert!(
                    Reader::new(machine.evaluator.facts())
                        .definition(definition)
                        .is_none()
                );
                references.push(
                    machine
                        .evaluator
                        .static_address(definition, TypeId::primitive(Primitive::U32))?,
                );
            }
            let value = machine.evaluator.fields(&references, None)?;
            let address = machine.allocate(&mut allocations, Some(value), ty)?;
            let root = address.location().ok_or("reference array root")?;
            let backing = machine.convert_byte_root(root)?;
            machine.read_operation = Some(ReadOperation {
                frame: 1,
                position: 0,
            });
            let mut retry_work = Vec::new();
            for (index, &definition) in definitions.iter().enumerate() {
                machine.waiting = None;
                let before = machine.work.steps;
                assert!(matches!(
                    machine.read_cell(
                        root,
                        &[],
                        crate::const_eval::static_reads::ReadPurpose::Value
                    )?,
                    Read::Pending
                ));
                retry_work.push(machine.work.steps - before);
                assert!(
                    matches!(machine.waiting, Some(crate::const_eval::Requirement::InitializerBody(actual)) if actual == definition)
                );
                assert_eq!(machine.byte_backing(backing)?.pending_reads, 1);
                let snapshot = machine
                    .snapshot_byte_root(root, 0, 8)?
                    .ok_or("pending source snapshot")?;
                assert!(machine.commit_byte_snapshot(root, 0, &snapshot).is_err());
                machine
                    .evaluator
                    .complete_definition(definition, scalar(index as u128 + 11));
            }
            assert_eq!(machine.memory.bytes.decoded, width + 1);
            assert_eq!(machine.memory.bytes.checked_duties, 2 * width - 1);
            assert_eq!(machine.memory.bytes.paid_duties, width - 1);
            machine.waiting = None;
            let before = machine.work.steps;
            let value = ready_cell(machine, root, &[])?;
            assert_eq!(machine.memory.bytes.decoded, width + 1);
            assert_eq!(machine.memory.bytes.checked_duties, 2 * width);
            assert_eq!(machine.memory.bytes.paid_duties, width);
            println!(
                "pending width={width} decoded={} attempts={} paid={}",
                machine.memory.bytes.decoded,
                machine.memory.bytes.checked_duties,
                machine.memory.bytes.paid_duties
            );
            retry_work.push(machine.work.steps - before);
            assert!(
                retry_work[1..].iter().all(|&work| work <= 512),
                "width={width} retry_work={retry_work:?}"
            );
            assert_eq!(machine.byte_backing(backing)?.pending_reads, 0);
            let first = machine.evaluator.field(value, 0)?;
            assert_eq!(first.location(), references[0].location());
            let second = machine
                .snapshot_byte_root(root, 8, 8)?
                .ok_or("second reference snapshot")?;
            machine.commit_byte_snapshot(root, 0, &second)?;
            // Reuse the exact instruction identity after a write: successful reads retain no cursor.
            let next = ready_cell(machine, root, &[])?;
            let first = machine.evaluator.field(next, 0)?;
            assert_eq!(first.location(), references[1].location());
            assert_eq!(machine.memory.bytes.decoded, 2 * (width + 1));
            assert_eq!(machine.memory.bytes.checked_duties, 3 * width);
            assert_eq!(machine.memory.bytes.paid_duties, 2 * width);
            assert_eq!(machine.slots, 0);
            Ok(())
        })?;
    }
    Ok(())
}

#[test]
fn symbolic_scoped_object_union_retains_demanded_field_shape() -> Result<(), String> {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?;
    let source =
        std::fs::read_to_string(root.join(
            "pa28/tests/spec/relocation-transport/vtable-borrowed-concrete-forward-scoped.rs",
        ))
        .map_err(|error| error.to_string())?;
    let report = test_source::resolve(&source, true)?;
    crate::pipeline::analyze_source(&report).map_err(|error| error.to_string())?;
    Ok(())
}

#[test]
fn symbolic_union_field_storage_preserves_mutability_shape_and_binder() -> Result<(), String> {
    let report = test_source::resolve("trait T{}fn f(_: *const dyn T){}", true)?;
    let mut facts = sema::declarations::analyze(&report).map_err(|error| error.to_string())?;
    let (shape, arguments, bound_regions) = (0..facts.types.len())
        .filter_map(|index| u32::try_from(index).ok())
        .find_map(|index| match facts.types.kind(TypeId(index)) {
            Some(Kind::Dynamic {
                shape,
                arguments,
                bound_regions,
                ..
            }) => Some((shape, arguments, bound_regions)),
            _ => None,
        })
        .ok_or("source dynamic type")?;
    let mut pointers = Vec::new();
    for (mutable, bound, region) in [
        (false, bound_regions, crate::types::RegionId::STATIC),
        (false, bound_regions, crate::types::RegionId(9)),
        (true, bound_regions, crate::types::RegionId(9)),
        (false, bound_regions + 1, crate::types::RegionId(9)),
    ] {
        let pointee = facts.types.intern(Kind::Dynamic {
            shape,
            arguments,
            bound_regions: bound,
            region,
        })?;
        pointers.push(facts.types.intern(Kind::Pointer { mutable, pointee })?);
    }
    let scalar = facts.types.intern(Kind::Pointer {
        mutable: false,
        pointee: TypeId::primitive(Primitive::U32),
    })?;
    let evaluator = Evaluator::new(&report, &mut facts);
    let machine = Machine::new(evaluator, std::iter::empty(), Limits::default())?;
    assert!(machine.image_field_compatible(pointers[0], pointers[1]));
    assert!(!machine.image_field_compatible(pointers[0], pointers[2]));
    assert!(!machine.image_field_compatible(pointers[0], pointers[3]));
    assert!(!machine.image_field_compatible(pointers[0], scalar));
    assert_eq!(machine.memory.retained_cells(), 0);
    Ok(())
}

#[path = "transmute_values.rs"]
mod transmute_values;
