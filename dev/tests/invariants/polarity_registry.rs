//! Closed application partitions must avoid pairwise scans of unrelated arguments.
use super::{Candidates, Implementation, Implementations};
use crate::{
    resolve::{
        model::DefId,
        unit::{FileId, NodeKey},
    },
    syntax::ast::NodeId,
    types::{
        ItemId, Kind, Primitive, RegionId, Store, TypeId,
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
        source: Some(NodeKey {
            file: FileId(0),
            node: NodeId::new(ordinal),
        }),
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
    assert!(store.shape_key_steps() < 50_000);
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

#[test]
fn header_polarity_status_preserves_entry_layout_and_rejects_missing_factory_owner()
-> Result<(), Box<dyn std::error::Error>> {
    assert_eq!(size_of::<super::Entry>(), 136);
    assert_eq!(size_of::<super::polarity::Status>(), 1);
    let mut store = Store::default();
    let mut registry = Implementations::default();
    let header = header(&mut store, 0, &[])?;
    let owner = header.owner;
    registry.prepare(&mut store, ItemId(1), header)?;
    assert!(
        registry
            .polarity(&crate::sema::associated::Inventory::default(), owner)
            .is_err()
    );
    Ok(())
}

#[test]
fn source_polarity_publication_checks_the_actual_inventory_row()
-> Result<(), Box<dyn std::error::Error>> {
    let source = crate::test_source::fixture("pa31/tests/spec/source-freeze/shared-default.rs")?;
    let report = crate::test_source::source_provider(&source, "negative_polarity")?;
    let mut facts = crate::sema::declarations::analyze(&report)?;
    let freeze = facts
        .source_bindings
        .as_deref()
        .ok_or("source bindings")?
        .validated_language(crate::support::bindings::LangItem::Freeze)?
        .ok_or("Freeze")?
        .definition;
    let original = facts
        .generics
        .implementations
        .as_ref()
        .ok_or("registry")?
        .headers()
        .find(|header| {
            header
                .predicate
                .bound
                .trait_parts()
                .is_some_and(|(application, _)| {
                    facts
                        .types
                        .trait_application(application)
                        .is_some_and(|a| a.item.0 == freeze.0)
                })
        })
        .ok_or("header")?;
    let implementation = Implementation {
        owner: original.owner,
        source: original.source,
        predicate: original.predicate,
        methods: Vec::new(),
        types: Vec::new(),
        constants: Vec::new(),
    };
    let owner = *facts
        .associated
        .owner(implementation.owner)
        .and_then(|id| facts.associated.get(id))
        .ok_or("owner")?;
    assert!(matches!(
        owner.kind,
        crate::sema::associated::OwnerKind::Impl { negative: true, .. }
    ));
    let mut forged = owner;
    if let crate::sema::associated::OwnerKind::Impl { negative, .. } = &mut forged.kind {
        *negative = false;
    }
    let mut registry = Implementations::default();
    assert!(
        registry
            .prepare_source(
                &mut facts.types,
                &facts.associated,
                super::Identity::User(ItemId(freeze.0)),
                forged,
                implementation
            )
            .is_err()
    );
    assert_eq!(registry.headers().count(), 0);
    Ok(())
}

#[test]
fn source_auto_override_incomplete_bucket_is_not_cached_absence()
-> Result<(), Box<dyn std::error::Error>> {
    use crate::sema::traits::{assumptions::Assumptions, selection};
    use crate::types::infer::{Context, Instantiation};
    let source = crate::test_source::fixture(
        "pa31/tests/spec/source-freeze/disjoint-reference-positive.rs",
    )?;
    let report = crate::test_source::source_provider(&source, "freeze_pending")?;
    let mut facts = crate::sema::declarations::analyze(&report)?;
    let role = facts
        .source_bindings
        .as_deref()
        .ok_or("catalog")?
        .validated_language(crate::support::bindings::LangItem::Freeze)?
        .ok_or("Freeze")?;
    let application = facts.types.intern_trait_application(Application {
        item: ItemId(role.definition.0),
        arguments: crate::types::GenericArgs::EMPTY,
    })?;
    for complete in [false, true] {
        facts
            .generics
            .implementations
            .as_mut()
            .ok_or("registry")?
            .headers = if complete {
            super::Headers::Validated
        } else {
            super::Headers::Constants
        };
        let n = report
            .resolution
            .definitions
            .iter()
            .position(|d| report.package.names.get(d.name) == "M")
            .ok_or("M")?;
        let referent = facts.definitions[n].ok_or("M type")?;
        let ty = facts.types.intern(Kind::Reference {
            mutable: false,
            region: RegionId::STATIC,
            referent,
        })?;
        let mut context = Context::default();
        let mut substitution = Instantiation::new(&mut context, &facts.types, &[])?;
        let goal = context.instantiate_predicate(
            &mut facts.types,
            Predicate {
                ty,
                bound: Bound::Trait(application),
            },
            &mut substitution,
        )?;
        let assumptions = Assumptions::default();
        let result = selection::select(
            &mut facts,
            &mut context,
            &goal,
            selection::Caller {
                assumptions: &assumptions,
                environment: None,
                first_region: 1,
                retain_sized: false,
            },
        );
        if complete {
            assert!(matches!(result?, selection::Selection::Absent));
        } else {
            assert!(
                result
                    .err()
                    .ok_or("incomplete header yielded absence")?
                    .contains("validated implementation headers")
            );
        }
    }
    Ok(())
}

#[test]
fn unavailable_negative_header_preserves_polarity_and_normal_completion()
-> Result<(), Box<dyn std::error::Error>> {
    let source = crate::test_source::fixture("pa31/tests/spec/source-freeze/shared-default.rs")?;
    let report = crate::test_source::source_provider(&source, "negative_early_profile")?;
    let mut facts = crate::sema::declarations::analyze(&report)?;
    let original = facts
        .generics
        .implementations
        .as_ref()
        .ok_or("registry")?
        .headers()
        .find(|header| {
            facts
                .associated
                .owner(header.owner)
                .and_then(|id| facts.associated.get(id))
                .is_some_and(|owner| {
                    matches!(
                        owner.kind,
                        crate::sema::associated::OwnerKind::Impl { negative: true, .. }
                    )
                })
        })
        .ok_or("actual negative header")?;
    let (definition, source, predicate) = (original.owner, original.source, original.predicate);
    let owner = *facts
        .associated
        .owner(definition)
        .and_then(|id| facts.associated.get(id))
        .ok_or("actual owner")?;
    let (application, _) = predicate.bound.trait_parts().ok_or("actual trait")?;
    let identity = super::Identity::User(
        facts
            .types
            .trait_application(application)
            .ok_or("application")?
            .item,
    );
    let mut registry = Implementations::default();
    registry.prepare_source(
        &mut facts.types,
        &facts.associated,
        identity,
        owner,
        Implementation {
            owner: definition,
            source,
            predicate,
            methods: Vec::new(),
            types: Vec::new(),
            constants: Vec::new(),
        },
    )?;
    let arena = facts.types.arena_identity();
    registry.mark_early_unavailable(
        &facts.types,
        &arena,
        definition,
        source.ok_or("actual source header")?,
        predicate,
    )?;
    assert!(registry.early_unavailable(&facts.types, definition)?);
    assert!(!registry.is_validated(definition));
    assert!(matches!(
        registry.polarity(&facts.associated, definition)?,
        super::Polarity::Negative
    ));
    registry.complete(definition, Vec::new(), Vec::new(), Vec::new())?;
    assert!(!registry.early_unavailable(&facts.types, definition)?);
    assert!(registry.is_validated(definition));
    assert!(matches!(
        registry.polarity(&facts.associated, definition)?,
        super::Polarity::Negative
    ));
    println!(
        "unavailable header layout Entry={} Status={}",
        size_of::<super::Entry>(),
        size_of::<super::polarity::Status>()
    );
    assert_eq!(size_of::<super::Entry>(), 136);
    assert_eq!(size_of::<super::polarity::Status>(), 1);
    Ok(())
}

#[test]
fn ordinary_completed_source_allocates_no_unavailable_arena_anchor()
-> Result<(), Box<dyn std::error::Error>> {
    let source =
        crate::test_source::fixture("pa31/tests/spec/early-copy-readiness/runtime-read.rs")?;
    let report = crate::test_source::source_provider(&source, "ordinary_profile_anchor")?;
    let facts = crate::sema::declarations::analyze(&report)?;
    let registry = facts.generics.implementations.as_ref().ok_or("registry")?;
    assert!(registry.profile_arena.is_none());
    assert!(
        registry
            .entries
            .iter()
            .all(|entry| !entry.status.early_unavailable())
    );
    println!(
        "ordinary completed registry={} optional unavailable anchor absent",
        size_of::<Implementations>()
    );
    Ok(())
}
