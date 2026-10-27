//! Pure consequence folds preserve exact identity, binder opacity and cumulative payment.
use super::*;
use crate::types::{
    GenericArgs, ItemId, Kind, Primitive, RegionId,
    parameters::predicates::Predicate,
    projection::{Application, Schema},
    traits,
};
use std::cell::Cell;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
fn projected(store: &mut Store, subject: TypeId) -> Result<TypeId> {
    store.declare_projection(
        ItemId(10),
        Schema {
            trait_item: ItemId(1),
            sized: true,
            copies: crate::types::copy::Properties::default(),
        },
    )?;
    let trait_application = store.intern_trait_application(traits::Application {
        item: ItemId(1),
        arguments: GenericArgs::EMPTY,
    })?;
    Ok(store.intern(Kind::Projection(Application {
        item: ItemId(10),
        subject,
        trait_application,
        arguments: GenericArgs::EMPTY,
    }))?)
}
fn exact_equations(store: &mut Store) -> Result {
    let byte = TypeId::primitive(Primitive::U8);
    let word = TypeId::primitive(Primitive::U16);
    let first = projected(store, byte)?;
    let other = projected(store, word)?;
    let mut index = Index::default();
    index.equalities.insert(
        ItemId(10),
        vec![Clause::Predicate(Predicate {
            ty: first,
            bound: Bound::ProjectionEqual(byte),
        })],
    );
    let (mut map, _) = equations::<true>(&index, &|_, _| Ok(()))?;
    let mut fold = Normalizer::default();
    assert_eq!(
        rewrite::<true>(store, &mut fold, first, &map, &|_, _| Ok(()))
            .map_err(NormalizationFailure::message)?,
        byte
    );
    assert_eq!(
        rewrite::<true>(store, &mut fold, other, &map, &|_, _| Ok(()))
            .map_err(NormalizationFailure::message)?,
        other
    );
    index
        .equalities
        .get_mut(&ItemId(10))
        .ok_or("actual equations")?
        .push(Clause::Predicate(Predicate {
            ty: first,
            bound: Bound::ProjectionEqual(word),
        }));
    (map, _) = equations::<true>(&index, &|_, _| Ok(()))?;
    assert_eq!(map.get(&first), Some(&None));
    assert_eq!(
        rewrite::<true>(store, &mut Normalizer::default(), first, &map, &|_, _| Ok(
            ()
        ))
        .map_err(NormalizationFailure::message)?,
        first
    );
    map.insert(first, Some(other));
    map.insert(other, Some(first));
    assert_eq!(
        rewrite::<true>(store, &mut Normalizer::default(), first, &map, &|_, _| Ok(
            ()
        )),
        Err(NormalizationFailure::Cycle)
    );
    let parameters = store.intern_list(&[first])?;
    let callable = store.intern(Kind::Function {
        bound_regions: 1,
        parameters,
        result: byte,
        unsafe_: false,
        convention: crate::types::callable::Convention::RUST,
    })?;
    assert_eq!(
        rewrite::<true>(
            store,
            &mut Normalizer::default(),
            callable,
            &map,
            &|_, _| Ok(())
        )
        .map_err(NormalizationFailure::message)?,
        callable
    );
    // A region-bearing subject remains a different full projection identity.
    let referent = store.intern(Kind::Reference {
        mutable: false,
        region: RegionId::STATIC,
        referent: byte,
    })?;
    let region_projection = projected(store, referent)?;
    assert_eq!(
        rewrite::<true>(
            store,
            &mut Normalizer::default(),
            region_projection,
            &map,
            &|_, _| Ok(())
        )
        .map_err(NormalizationFailure::message)?,
        region_projection
    );
    Ok(())
}
fn shared_paid_roots(store: &mut Store) -> Result {
    let byte = TypeId::primitive(Primitive::U8);
    let projection = projected(store, byte)?;
    let mut map = HashMap::default();
    map.insert(projection, Some(byte));
    let mut root = projection;
    for _ in 0..32 {
        let list = store.intern_list(&[root, root])?;
        root = store.intern(Kind::Tuple(list))?;
    }
    let work = Cell::new(0_usize);
    let bytes = Cell::new(0_usize);
    let charge = |steps, growth| {
        work.set(work.get().saturating_add(steps));
        bytes.set(bytes.get().saturating_add(growth));
        Ok(())
    };
    let mut fold = Normalizer::default();
    let result = rewrite::<true>(store, &mut fold, root, &map, &charge)
        .map_err(NormalizationFailure::message)?;
    assert!(fold.steps() < 1_000);
    let prior = (work.get(), bytes.get());
    assert_eq!(
        rewrite::<true>(store, &mut fold, root, &map, &charge)
            .map_err(NormalizationFailure::message)?,
        result
    );
    assert!(work.get() > prior.0);
    assert_eq!(bytes.get(), prior.1);
    for _ in 0..2 {
        let before = work.get();
        let rejected = |steps, growth| {
            charge(steps, growth)?;
            Err("consequence payment exhausted")
        };
        assert!(rewrite::<true>(store, &mut Normalizer::default(), root, &map, &rejected).is_err());
        assert!(work.get() > before);
    }
    // The uncharged public adapter retains precisely the same result.
    let mut ordinary = Normalizer::default();
    let mut step = ordinary.start(store, root)?;
    loop {
        match step {
            Step::Complete(actual) => {
                assert_eq!(actual, result);
                break;
            }
            Step::Resolve(request) => {
                step = ordinary.resume(store, request, Resolution::Type(byte))?;
            }
        }
    }
    println!(
        "Result consequence shared32 work={} paid={} Normalizer={}",
        work.get(),
        bytes.get(),
        size_of::<Normalizer>()
    );
    Ok(())
}
pub(in crate::sema) fn controls() -> Result {
    let mut store = Store::default();
    exact_equations(&mut store)?;
    shared_paid_roots(&mut store)?;
    retained::controls()
}

#[path = "consequence_retention.rs"]
mod retained;
