//! Closed application partitions must avoid pairwise scans of unrelated arguments.
use super::{Implementation, Implementations};
use crate::{
    resolve::{
        model::DefId,
        unit::{FileId, NodeKey},
    },
    syntax::ast::NodeId,
    types::{
        ItemId, Primitive, RegionId, Store, TypeId,
        arguments::{Argument, Constant},
        parameters::predicates::{Bound, Predicate},
        traits::Application,
    },
};

fn header(
    store: &mut Store,
    ordinal: u32,
    arguments: &[Argument],
) -> Result<Implementation, &'static str> {
    let arguments = store.intern_arguments(arguments)?;
    let application = store.intern_trait_application(Application {
        item: ItemId(1),
        arguments,
    })?;
    Ok(Implementation {
        owner: DefId(ordinal + 10),
        source: NodeKey {
            file: FileId(0),
            node: NodeId(ordinal),
        },
        predicate: Predicate {
            ty: TypeId::primitive(Primitive::U8),
            bound: Bound::Trait(application),
        },
        methods: Vec::new(),
    })
}

#[test]
fn unrelated_closed_arguments_do_not_create_overlap_probes()
-> Result<(), Box<dyn std::error::Error>> {
    let mut store = Store::default();
    let mut registry = Implementations::default();
    for ordinal in 0..4096 {
        let value = store.intern_constant(Constant {
            ty: TypeId::primitive(Primitive::Usize),
            bits: u128::from(ordinal),
        })?;
        let header = header(&mut store, ordinal, &[Argument::Const(value)])?;
        registry.prepare(&mut store, ItemId(1), header)?;
    }
    assert!(registry.take_overlap_pairs().is_empty());
    assert_eq!(registry.headers().count(), 4096);
    assert_eq!(registry.validated().count(), 0);
    assert!(
        registry
            .shapes
            .as_ref()
            .is_some_and(|folder| folder.steps < 50_000)
    );
    let owner = DefId(4105);
    let predicate = registry
        .header(owner)
        .ok_or("missing closed header")?
        .predicate;
    let mut context = crate::types::infer::Context::default();
    let mut substitution = crate::types::infer::Instantiation::new(&mut context, &store, &[])?;
    let goal = context.instantiate_predicate(&mut store, predicate, &mut substitution)?;
    assert!(
        registry
            .candidates(&mut store, &mut context, &goal)?
            .is_empty()
    );
    registry.complete(owner, Vec::new())?;
    let before = context.steps;
    assert_eq!(
        registry.candidates(&mut store, &mut context, &goal)?,
        [owner]
    );
    assert!(context.steps - before < 100, "{}", context.steps - before);
    Ok(())
}

#[test]
fn erased_lifetime_arguments_still_require_an_overlap_probe()
-> Result<(), Box<dyn std::error::Error>> {
    let mut store = Store::default();
    let mut registry = Implementations::default();
    for (ordinal, region) in [RegionId::bound(0, 0)?, RegionId::STATIC]
        .into_iter()
        .enumerate()
    {
        let header = header(
            &mut store,
            u32::try_from(ordinal)?,
            &[Argument::Lifetime(region)],
        )?;
        registry.prepare(&mut store, ItemId(1), header)?;
    }
    assert_eq!(registry.take_overlap_pairs().len(), 1);
    assert!(!registry.is_validated(DefId(10)));
    registry.complete(DefId(10), Vec::new())?;
    assert!(registry.is_validated(DefId(10)));
    assert_eq!(registry.validated().count(), 1);
    assert!(registry.complete(DefId(10), Vec::new()).is_err());
    Ok(())
}
