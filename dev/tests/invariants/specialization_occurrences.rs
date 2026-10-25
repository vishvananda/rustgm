//! One demanded DAG counts repeated paths without parameter-by-node summaries.
use super::Multiplicities;
use crate::types::{
    ItemId, Kind, Store,
    arguments::Argument,
    parameters::{
        Parameter, ParameterKind,
        predicates::{Bound, Predicate},
        uses::Sole,
    },
};
use std::cell::Cell;
type Result<T = ()> = std::result::Result<T, &'static str>;

#[test]
fn specialization_counts_shared_diamonds_many_parameters_and_failed_work() -> Result {
    for width in [8_u32, 32, 128] {
        let mut store = Store::default();
        let mut arguments = Vec::new();
        for index in 0..width {
            let id = store.intern_parameter(Parameter {
                owner: ItemId(1),
                index,
                kind: ParameterKind::Type { sized: true },
            })?;
            arguments.push(Argument::Type(store.intern(Kind::Parameter(id))?));
        }
        let roots = store.intern_arguments(&arguments)?;
        let work = Cell::new(0_usize);
        let bytes = Cell::new(0_usize);
        let charge = |steps, retained| {
            work.set(work.get().saturating_add(steps));
            bytes.set(bytes.get().saturating_add(retained));
            Ok(())
        };
        let separate = Multiplicities::arguments(&store, roots, charge)?;
        assert!(!separate.repeated(charge)?);
        assert_eq!(separate.parameters.len(), width as usize);
        assert!(work.get() < 64 * width as usize + 64);
        let Argument::Type(mut diamond) = arguments[0] else {
            return Err("missing parameter type");
        };
        for _ in 0..width {
            let list = store.intern_list(&[diamond, diamond])?;
            diamond = store.intern(Kind::Tuple(list))?;
        }
        let roots = store.intern_arguments(&[Argument::Type(diamond), Argument::Type(diamond)])?;
        work.set(0);
        bytes.set(0);
        let shared = Multiplicities::arguments(&store, roots, charge)?;
        assert!(shared.repeated(charge)?);
        assert_eq!(shared.parameters.len(), 1);
        assert!(work.get() < 256 * width as usize + 256);
        assert!(shared.nodes.len() < 4 * width as usize + 8);
        let mut sole = Sole::default();
        assert_eq!(
            sole.predicate(
                &store,
                Predicate {
                    ty: diamond,
                    bound: Bound::Copy
                }
            )?,
            shared.parameters.keys().find_map(|key| match key {
                super::Parameter::Declared(id) => Some(*id),
                super::Parameter::Capture(_) => None,
            })
        );
        assert!(sole.steps < 16 * width as usize + 16);
        eprintln!(
            "specialization DAG width={width} nodes={} edges={} work={} logical_bytes={} Sole={}",
            shared.nodes.len(),
            shared.edges.len(),
            work.get(),
            bytes.get(),
            size_of::<Sole>()
        );
        let attempts = Cell::new(0_usize);
        let rejected = Multiplicities::arguments(&store, roots, |steps, _| {
            attempts.set(attempts.get().saturating_add(steps));
            Err("frozen occurrence budget exhausted")
        });
        assert!(matches!(
            rejected,
            Err("frozen occurrence budget exhausted")
        ));
        assert_eq!(attempts.get(), 1);
    }
    scoped_captures_and_function_item_inputs_are_counted_once()?;
    Ok(())
}

fn scoped_captures_and_function_item_inputs_are_counted_once() -> Result {
    use crate::types::{RegionId, TypeId};
    let mut store = Store::default();
    let mut references = Vec::new();
    for depth in [0, 1] {
        references.push(store.intern(Kind::Reference {
            mutable: false,
            region: RegionId::bound(depth, 0)?,
            referent: TypeId::UNIT,
        })?);
    }
    let parameters = store.intern_list(&[references[1], references[0]])?;
    let callback = store.intern(Kind::Function {
        parameters,
        result: TypeId::UNIT,
        bound_regions: 1,
        unsafe_: false,
        convention: crate::types::callable::Convention::RUST,
    })?;
    let roots = store.intern_arguments(&[Argument::Type(callback)])?;
    let once = Multiplicities::arguments(&store, roots, |_, _| Ok(()))?;
    assert_eq!(once.parameters.get(&super::Parameter::Capture(0)), Some(&1));
    assert!(!once.repeated(|_, _| Ok(()))?);
    let roots =
        store.intern_arguments(&[Argument::Type(callback), Argument::Type(references[0])])?;
    let twice = Multiplicities::arguments(&store, roots, |_, _| Ok(()))?;
    assert_eq!(
        twice.parameters.get(&super::Parameter::Capture(0)),
        Some(&2)
    );
    assert!(twice.repeated(|_, _| Ok(()))?);
    let id = store.intern_parameter(Parameter {
        owner: ItemId(8),
        index: 0,
        kind: ParameterKind::Type { sized: true },
    })?;
    let ty = store.intern(Kind::Parameter(id))?;
    let arguments = store.intern_arguments(&[Argument::Type(ty)])?;
    let parameters = store.intern_list(&[ty])?;
    let signature = store.intern(Kind::Function {
        parameters,
        result: ty,
        bound_regions: 0,
        unsafe_: false,
        convention: crate::types::callable::Convention::RUST,
    })?;
    let item = store.intern(Kind::FunctionItem {
        item: ItemId(9),
        arguments,
        signature,
    })?;
    let roots = store.intern_arguments(&[Argument::Type(item)])?;
    let logical = Multiplicities::arguments(&store, roots, |_, _| Ok(()))?;
    assert_eq!(
        logical.parameters.get(&super::Parameter::Declared(id)),
        Some(&1)
    );
    assert!(!logical.repeated(|_, _| Ok(()))?);
    let mut sole = Sole::default();
    assert_eq!(
        sole.predicate(
            &store,
            Predicate {
                ty: item,
                bound: Bound::Copy
            }
        )?,
        Some(id)
    );
    Ok(())
}
