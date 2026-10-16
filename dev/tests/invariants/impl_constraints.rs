//! Parameter determination follows dependency edges, regardless of declaration order.
use super::{Constraints, Uses};
use crate::types::{
    GenericArgs, ItemId, Kind, Store, TypeId,
    parameters::{
        Parameter, ParameterId, ParameterKind,
        predicates::{Bound, Predicate},
    },
    projection::{Application, Schema},
    traits,
};

#[test]
fn long_reverse_dependencies_and_unseeded_cycles_use_bounded_work() -> Result<(), &'static str> {
    let mut store = Store::default();
    let mut parameters = Vec::new();
    for index in 0..10_001 {
        let id = store.intern_parameter(Parameter {
            owner: ItemId(1),
            index,
            kind: ParameterKind::Type { sized: true },
        })?;
        parameters.push((id, store.intern(Kind::Parameter(id))?));
    }
    let header = Predicate {
        ty: parameters[0].1,
        bound: Bound::Sized,
    };
    let mut uses = Uses::constraining();
    uses.predicate(&store, header)?;
    let mut constraints = Constraints::new(&store, header, uses)?;
    let mut unseeded = Constraints::new(&store, header, Uses::constraining())?;
    for pair in parameters.windows(2).rev() {
        let equality = equality(&mut store, pair[0].1, pair[1].1)?;
        constraints.equality(&store, equality)?;
        unseeded.equality(&store, equality)?;
    }
    let back = equality(&mut store, parameters[10_000].1, parameters[0].1)?;
    unseeded.equality(&store, back)?;
    constraints.solve()?;
    unseeded.solve()?;
    assert!(parameters.iter().all(|(id, _)| constraints.determines(*id)));
    assert!(parameters.iter().all(|(id, _)| !unseeded.determines(*id)));
    assert!(constraints.waiting.is_empty());
    assert!(constraints.steps < 200_000, "{}", constraints.steps);
    assert!(unseeded.steps < 200_000, "{}", unseeded.steps);
    assert!(!constraints.determines(ParameterId(u32::MAX)));
    Ok(())
}

fn equality(store: &mut Store, input: TypeId, output: TypeId) -> Result<Predicate, &'static str> {
    store.declare_projection(
        ItemId(3),
        Schema {
            trait_item: ItemId(2),
            sized: true,
        },
    )?;
    let trait_application = store.intern_trait_application(traits::Application {
        item: ItemId(2),
        arguments: GenericArgs::EMPTY,
    })?;
    let ty = store.intern(Kind::Projection(Application {
        item: ItemId(3),
        subject: input,
        trait_application,
        arguments: GenericArgs::EMPTY,
    }))?;
    Ok(Predicate {
        ty,
        bound: Bound::ProjectionEqual(output),
    })
}
