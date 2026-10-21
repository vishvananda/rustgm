//! Erased keys share only immutable structure, retaining every non-lifetime input.
use crate::types::{
    ItemId, Kind, Primitive, RegionId, Store, TypeId,
    arguments::{Argument, Constant},
    parameters::predicates::{Bound, Predicate},
    traits::Application,
};
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

fn predicate(
    store: &mut Store,
    subject: TypeId,
    item: ItemId,
    args: &[Argument],
) -> TestResult<Predicate> {
    let arguments = store.intern_arguments(args)?;
    let application = store.intern_trait_application(Application { item, arguments })?;
    Ok(Predicate {
        ty: subject,
        bound: Bound::Trait(application),
    })
}

#[test]
fn unrelated_queries_share_structural_work_and_retain_complete_inputs() -> TestResult {
    let byte = TypeId::primitive(Primitive::U8);
    for width in [64, 128, 256, 512] {
        let mut store = Store::default();
        let mut tail = byte;
        let mut roots = Vec::new();
        for _ in 0..width {
            tail = store.intern(Kind::Reference {
                mutable: false,
                region: RegionId(3),
                referent: tail,
            })?;
            roots.push(predicate(
                &mut store,
                byte,
                ItemId(1),
                &[Argument::Type(tail)],
            )?);
        }
        let before = store.len();
        let mut keys = Vec::new();
        let mut steps = 0;
        for &root in &roots {
            let (key, work) = store.shape_key(root)?;
            steps += work;
            keys.push(key);
        }
        assert!(steps <= 8 * width, "{width}: {steps}");
        assert!(store.len() - before <= width);
        for (&root, &key) in roots.iter().zip(&keys).rev() {
            assert_eq!(store.shape_key(root)?, (key, 0));
        }
        // A malformed new request must not discard already completed structural work.
        assert!(
            store
                .shape_key(Predicate {
                    ty: TypeId(u32::MAX),
                    bound: Bound::Copy
                })
                .is_err()
        );
        for (&root, &key) in roots.iter().zip(&keys) {
            assert_eq!(store.shape_key(root)?, (key, 0));
        }
    }
    Ok(())
}

#[test]
fn shape_equality_erases_lifetimes_but_never_type_const_or_trait_identity() -> TestResult {
    let mut store = Store::default();
    let byte = TypeId::primitive(Primitive::U8);
    let word = TypeId::primitive(Primitive::U32);
    let first = store.intern_constant(Constant {
        ty: TypeId::primitive(Primitive::Usize),
        bits: 1,
    })?;
    let second = store.intern_constant(Constant {
        ty: TypeId::primitive(Primitive::Usize),
        bits: 2,
    })?;
    let original = predicate(
        &mut store,
        byte,
        ItemId(1),
        &[
            Argument::Lifetime(RegionId(1)),
            Argument::Type(byte),
            Argument::Const(first),
        ],
    )?;
    let (key, _) = store.shape_key(original)?;
    for region in [
        RegionId::STATIC,
        RegionId(2),
        RegionId::bound(0, 0)?,
        RegionId::bound(1, 1)?,
    ] {
        let value = predicate(
            &mut store,
            byte,
            ItemId(1),
            &[
                Argument::Lifetime(region),
                Argument::Type(byte),
                Argument::Const(first),
            ],
        )?;
        assert_eq!(store.shape_key(value)?.0, key);
    }
    for (subject, item, ty, constant) in [
        (word, ItemId(1), byte, first),
        (byte, ItemId(2), byte, first),
        (byte, ItemId(1), word, first),
        (byte, ItemId(1), byte, second),
    ] {
        let value = predicate(
            &mut store,
            subject,
            item,
            &[
                Argument::Lifetime(RegionId(1)),
                Argument::Type(ty),
                Argument::Const(constant),
            ],
        )?;
        assert_ne!(store.shape_key(value)?.0, key);
    }
    Ok(())
}

#[test]
fn erased_callable_tails_do_not_repeat_work_at_each_binder_depth() -> TestResult {
    let mut observations = Vec::new();
    for width in [16, 32, 64, 128] {
        let mut store = Store::default();
        let byte = TypeId::primitive(Primitive::U8);
        let reference = store.intern(Kind::Reference {
            mutable: false,
            region: RegionId::bound(0, 0)?,
            referent: byte,
        })?;
        let parameters = store.intern_list(&[reference])?;
        let mut tail = TypeId::UNIT;
        let mut steps = 0;
        for _ in 0..width {
            tail = store.intern(Kind::Function {
                parameters,
                result: tail,
                bound_regions: 1,
                unsafe_: false,
                convention: crate::types::callable::Convention::RUST,
            })?;
            let goal = predicate(&mut store, byte, ItemId(1), &[Argument::Type(tail)])?;
            steps += store.shape_key(goal)?.1;
        }
        println!("shape-callables width={width} steps={steps}");
        observations.push((width, steps));
    }
    for (width, steps) in observations {
        assert!(steps <= 12 * width + 16, "{width}: {steps}");
    }
    Ok(())
}
