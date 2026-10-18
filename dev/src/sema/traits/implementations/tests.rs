//! Closed application partitions must avoid pairwise scans of unrelated arguments.
use super::{Candidates, Implementation, Implementations};
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
        types: Vec::new(),
        constants: Vec::new(),
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
    assert_eq!(
        registry.candidates(
            &mut store,
            &mut context,
            &goal,
            super::Availability::Validated
        )?,
        Candidates::Ready(Vec::new())
    );
    let before = context.steps;
    assert_eq!(
        registry.candidates(
            &mut store,
            &mut context,
            &goal,
            super::Availability::Declarations
        )?,
        Candidates::Ready(vec![owner])
    );
    assert!(context.steps - before < 100);
    assert!(!registry.is_validated(owner));
    assert_eq!(registry.validated().count(), 0);
    registry.complete(owner, Vec::new(), Vec::new(), Vec::new())?;
    let before = context.steps;
    assert_eq!(
        registry.candidates(
            &mut store,
            &mut context,
            &goal,
            super::Availability::Validated
        )?,
        Candidates::Ready(vec![owner])
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
    registry.complete(DefId(10), Vec::new(), Vec::new(), Vec::new())?;
    assert!(registry.is_validated(DefId(10)));
    assert_eq!(registry.validated().count(), 1);
    assert!(
        registry
            .complete(DefId(10), Vec::new(), Vec::new(), Vec::new())
            .is_err()
    );
    Ok(())
}

#[test]
fn checked_constants_do_not_authorize_unvalidated_operations()
-> Result<(), Box<dyn std::error::Error>> {
    let mut store = Store::default();
    let mut registry = Implementations::default();
    let header = header(&mut store, 0, &[])?;
    let owner = header.owner;
    registry.prepare(&mut store, ItemId(1), header)?;
    let declaration = DefId(99);
    let selected = DefId(100);
    let request = super::ConstantRequest { owner, declaration };
    assert!(
        registry
            .complete_constant(
                super::ConstantRequest {
                    owner: DefId(999),
                    declaration
                },
                selected
            )
            .is_err()
    );
    assert!(!registry.constant_headers_ready());
    registry.complete_constant(request, selected)?;
    assert!(!registry.constant_headers_ready());
    assert!(registry.complete_constant(request, DefId(101)).is_err());
    assert!(!registry.is_validated(owner));
    assert_eq!(registry.method(owner, declaration), None);
    assert_eq!(registry.constant(owner, declaration), None);
    assert_eq!(
        registry.available_constant(owner, declaration, super::Availability::Validated),
        None
    );
    assert_eq!(
        registry.available_constant(owner, declaration, super::Availability::Declarations),
        Some(selected)
    );
    assert!(
        registry
            .complete(owner, Vec::new(), Vec::new(), vec![(declaration, None)])
            .is_err()
    );
    assert!(!registry.is_validated(owner));
    registry.complete(
        owner,
        Vec::new(),
        Vec::new(),
        vec![(declaration, Some(selected))],
    )?;
    assert_eq!(registry.constant(owner, declaration), Some(selected));
    Ok(())
}
