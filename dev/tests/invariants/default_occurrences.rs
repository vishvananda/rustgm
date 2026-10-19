//! Default parameter indexing shares prefixes without losing mixed or projected uses.
use super::Sole;
use crate::types::{
    GenericArgs, ItemId, Kind, Primitive, Store, TypeId,
    arguments::{Argument, ConstKind},
    parameters::{
        Parameter, ParameterId, ParameterKind,
        predicates::{Bound, Predicate},
        uses::Uses,
    },
    projection, traits,
};
type Result<T = ()> = std::result::Result<T, &'static str>;

fn parameter(store: &mut Store, index: u32) -> Result<(ParameterId, TypeId)> {
    let id = store.intern_parameter(Parameter {
        owner: ItemId(1),
        index,
        kind: ParameterKind::Type { sized: true },
    })?;
    Ok((id, store.intern(Kind::Parameter(id))?))
}

#[test]
fn default_occurrences_share_all_growing_prefixes() -> Result {
    for width in [16, 128, 1024, 4096] {
        let mut store = Store::default();
        let (id, mut ty) = parameter(&mut store, 0)?;
        let mut query = Sole::default();
        let mut prior_work = 0;
        for _ in 0..width {
            let list = store.intern_list(&[ty])?;
            ty = store.intern(Kind::Tuple(list))?;
            let predicate = Predicate {
                ty,
                bound: Bound::Copy,
            };
            assert_eq!(query.predicate(&store, predicate)?, Some(id));
            if width <= 128 {
                let mut prior = Uses::default();
                prior.predicate(&store, predicate)?;
                prior_work += prior.steps;
            }
        }
        eprintln!(
            "default prefixes={width} shared={} prior={prior_work}",
            query.steps
        );
        assert!(query.steps < 16 * width + 8);
        assert!(query.completed.len() <= 2 * width + 1);
    }
    Ok(())
}

#[test]
fn default_occurrences_share_argument_lists_across_distinct_owners() -> Result {
    let mut store = Store::default();
    let (id, ty) = parameter(&mut store, 0)?;
    let arguments = store.intern_arguments(&vec![Argument::Type(ty); 4096])?;
    let mut query = Sole::default();
    for index in 0..4096 {
        let application = store.intern_trait_application(traits::Application {
            item: ItemId(index),
            arguments,
        })?;
        assert_eq!(
            query.predicate(
                &store,
                Predicate {
                    ty: TypeId::UNIT,
                    bound: Bound::Trait(application),
                }
            )?,
            Some(id)
        );
    }
    assert!(query.steps < 20 * 4096);
    eprintln!("default argument groups=4096 shared={}", query.steps);
    Ok(())
}

#[test]
fn default_occurrences_include_constants_results_and_noninjective_inputs() -> Result {
    let mut store = Store::default();
    let (first, a) = parameter(&mut store, 0)?;
    let (_, b) = parameter(&mut store, 1)?;
    let count = store.intern_parameter(Parameter {
        owner: ItemId(1),
        index: 2,
        kind: ParameterKind::Const {
            ty: TypeId::primitive(Primitive::Usize),
        },
    })?;
    let length = store.intern_const(ConstKind::Parameter(count))?;
    let array = store.intern(Kind::ArrayTerm {
        element: TypeId::UNIT,
        length,
    })?;
    let mixed = store.intern(Kind::ArrayTerm { element: a, length })?;
    let trait_application = store.intern_trait_application(traits::Application {
        item: ItemId(2),
        arguments: GenericArgs::EMPTY,
    })?;
    store.declare_projection(
        ItemId(3),
        projection::Schema {
            trait_item: ItemId(2),
            sized: true,
            copies: crate::types::copy::Properties::default(),
        },
    )?;
    let arguments = store.intern_arguments(&[Argument::Type(b)])?;
    let projected = store.intern(Kind::Projection(projection::Application {
        item: ItemId(3),
        subject: a,
        trait_application,
        arguments,
    }))?;
    let mut query = Sole::default();
    for (ty, bound, expected) in [
        (a, Bound::Copy, Some(first)),
        (array, Bound::Copy, Some(count)),
        (mixed, Bound::Copy, None),
        (TypeId::UNIT, Bound::ProjectionEqual(a), Some(first)),
        (a, Bound::ProjectionEqual(b), None),
        (projected, Bound::Copy, None),
        (a, Bound::ProjectionEqual(a), Some(first)),
    ] {
        assert_eq!(query.predicate(&store, Predicate { ty, bound })?, expected);
    }
    Ok(())
}
