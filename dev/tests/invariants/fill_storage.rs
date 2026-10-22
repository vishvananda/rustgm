//! Numeric spans alter only selected bytes, initialized bits and word fragments.
use super::*;
use crate::{
    const_eval::{Constant, Limits},
    sema::{self, model::Scalar, values::Evaluator},
    test_source,
    types::{Kind, Primitive, TypeId},
};
#[test]
fn fill_mask_marks_exactly_the_selected_bits_at_word_edges() {
    for start in 0..130 {
        for end in start..=130 {
            for initial in [0, 0x5555_5555_5555_5555] {
                let mut mask = [initial; 3];
                initialize_mask(&mut mask, start, end);
                for index in 0..192 {
                    assert_eq!(
                        mask[index / 64] & (1 << (index % 64)) != 0,
                        (start..end).contains(&index) || initial & (1 << (index % 64)) != 0
                    );
                }
            }
        }
    }
}
#[test]
fn fill_fragment_edits_are_local_and_repeated_small_writes_are_bounded() -> Result<(), String> {
    for width in [8u64, 32, 128] {
        let report = test_source::resolve("", true)?;
        let mut facts = sema::declarations::analyze(&report).map_err(|e| e.to_string())?;
        let leaf_ty = TypeId::primitive(Primitive::U32);
        let pointer_ty = facts.types.intern(Kind::Pointer {
            mutable: false,
            pointee: leaf_ty,
        })?;
        let array_ty = facts.types.intern(Kind::Array {
            element: pointer_ty,
            length: width,
        })?;
        let evaluator = Evaluator::new(&report, &mut facts);
        let mut machine = Machine::new(evaluator, std::iter::empty(), Limits::default())?;
        let mut allocations = Vec::new();
        let leaf = machine
            .allocate(
                &mut allocations,
                Some(Constant::scalar(Scalar::Integer(37))),
                leaf_ty,
            )
            .map_err(Failure::message)?;
        let value = machine
            .evaluator
            .repeat(leaf.restrict_shared(), width)
            .map_err(Failure::message)?;
        let root = machine
            .allocate(&mut allocations, Some(value), array_ty)
            .map_err(Failure::message)?
            .location()
            .ok_or("root")?;
        let id = machine.convert_byte_root(root).map_err(Failure::message)?;
        let offset = (width - 1) * 8 + 2;
        machine
            .fill_byte_root(root, offset, 2, 0xa7)
            .map_err(Failure::message)?;
        let backing = machine.byte_backing(id).map_err(Failure::message)?;
        let tree = backing.fragments.as_ref().ok_or("fragments")?;
        assert_eq!(
            tree.len(),
            usize::try_from(width).map_err(|_| "host width")? + 1
        );
        assert_eq!(
            tree.get(&((width - 1) * 8))
                .map(|f| (f.source_offset, f.length)),
            Some((0, 2))
        );
        assert_eq!(
            tree.get(&(offset + 2)).map(|f| (f.source_offset, f.length)),
            Some((4, 4))
        );
        let word = tree.get(&0).ok_or("untouched word")?.word;
        assert!(
            matches!(machine.word_target(word).map_err(Failure::message)?,
            crate::sema::values::WordTarget::Value(value) if value == leaf.restrict_shared())
        );
        let retained = machine.memory.retained_cells();
        let before = machine.work.steps;
        for _ in 0..128 {
            machine
                .fill_byte_root(root, offset, 2, 0xa7)
                .map_err(Failure::message)?;
        }
        let steps = machine.work.steps - before;
        assert!(steps <= 128 * (64 + 16 * (u64::BITS - width.leading_zeros()) as usize));
        assert_eq!(machine.memory.retained_cells(), retained);
        let backing = machine.byte_backing(id).map_err(Failure::message)?;
        let offset = usize::try_from(offset).map_err(|_| "host offset")?;
        assert_eq!(&backing.bytes[offset..offset + 2], &[0xa7; 2]);
        assert!(backing.initialized(offset) && backing.initialized(offset + 1));
        println!("fill-fragments width={width} repeats=128 steps={steps} retained={retained}");
    }
    Ok(())
}
