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
            shared.parameters.keys().next().copied()
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
    Ok(())
}
