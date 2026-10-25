//! Candidate completeness is independent of source order and member conformance.
use super::{Candidates, Header, Pending};
use crate::{
    resolve::{
        model::DefId,
        unit::{FileId, NodeKey},
    },
    sema::traits::{
        heads::{Head, Subject},
        implementations::{Availability, Implementation, Implementations},
    },
    syntax::ast::NodeId,
    types::{
        ItemId, Kind, Primitive, Store, TypeId,
        infer::{Context, Instantiation},
        parameters::predicates::{Bound, Predicate},
        traits::Application,
    },
};

fn header(owner: u32, item: u32, head: Option<Head>) -> Header {
    Header {
        owner: DefId(owner),
        item: ItemId(item).into(),
        head,
    }
}
fn implementation(
    store: &mut Store,
    owner: u32,
    item: u32,
    ty: TypeId,
) -> Result<Implementation, &'static str> {
    Ok(Implementation {
        owner: DefId(owner),
        source: Some(NodeKey {
            file: FileId(0),
            node: NodeId::new(owner),
        }),
        predicate: Predicate {
            ty,
            bound: Bound::Trait(store.intern_trait_application(Application {
                item: ItemId(item),
                arguments: crate::types::GenericArgs::EMPTY,
            })?),
        },
        methods: Vec::new(),
        types: Vec::new(),
        constants: Vec::new(),
    })
}

#[test]
fn selection_waits_for_all_compatible_headers_without_unrelated_owners()
-> Result<(), Box<dyn std::error::Error>> {
    let mut store = Store::default();
    let mut registry = Implementations::default();
    registry.reserve(vec![
        header(10, 1, Some(Head::Primitive(Primitive::U8))),
        header(11, 1, Some(Head::Primitive(Primitive::U8))),
        header(12, 1, Some(Head::Primitive(Primitive::U16))),
        header(13, 1, None),
        header(14, 2, Some(Head::Primitive(Primitive::U8))),
    ])?;
    let first = implementation(&mut store, 10, 1, TypeId::primitive(Primitive::U8))?;
    let predicate = first.predicate;
    registry.prepare(&mut store, ItemId(1), first)?;
    let mut context = Context::default();
    let mut substitution = Instantiation::new(&mut context, &store, &[])?;
    let goal = context.instantiate_predicate(&mut store, predicate, &mut substitution)?;
    assert_eq!(
        registry.candidates(&mut store, &mut context, &goal, Availability::Declarations)?,
        Candidates::Pending(vec![DefId(11), DefId(13)])
    );
    assert!(registry.finish_constant_headers().is_err());
    assert!(!registry.constant_headers_ready());
    let projected = implementation(&mut store, 13, 1, TypeId::primitive(Primitive::U16))?;
    registry.prepare(&mut store, ItemId(1), projected)?;
    assert_eq!(
        registry.candidates(&mut store, &mut context, &goal, Availability::Declarations)?,
        Candidates::Pending(vec![DefId(11)])
    );
    let alternative = implementation(&mut store, 11, 1, predicate.ty)?;
    registry.prepare(&mut store, ItemId(1), alternative)?;
    assert_eq!(
        registry.candidates(&mut store, &mut context, &goal, Availability::Declarations)?,
        Candidates::Ready(vec![DefId(10), DefId(11)])
    );
    assert_eq!(
        registry.candidates(&mut store, &mut context, &goal, Availability::Validated)?,
        Candidates::Ready(Vec::new())
    );
    assert!(registry.may_have_subject(ItemId(2), Subject::Rigid(Head::Primitive(Primitive::U8))));
    assert!(!registry.may_have_subject(ItemId(2), Subject::Rigid(Head::Primitive(Primitive::U16))));
    assert!(registry.finish_constant_headers().is_err());
    for (owner, item, primitive) in [(12, 1, Primitive::U16), (14, 2, Primitive::U8)] {
        let value = implementation(&mut store, owner, item, TypeId::primitive(primitive))?;
        registry.prepare(&mut store, ItemId(item), value)?;
    }
    registry.finish_constant_headers()?;
    assert!(registry.constant_headers_ready());
    assert!(registry.pending.is_none());
    assert_eq!(registry.validated().count(), 0);
    Ok(())
}

#[test]
fn executable_candidates_distinguish_header_validation_and_completed_absence()
-> Result<(), Box<dyn std::error::Error>> {
    let mut query_work = None;
    for noise in [0, 128] {
        let mut store = Store::default();
        let mut registry = Implementations::default();
        let mut headers = vec![header(10, 1, Some(Head::Primitive(Primitive::U8)))];
        headers.extend(
            (0..noise).map(|index| header(index + 20, 2, Some(Head::Nominal(ItemId(index + 20))))),
        );
        registry.reserve(headers)?;
        let value = implementation(&mut store, 10, 1, TypeId::primitive(Primitive::U8))?;
        let mut context = Context::default();
        let mut substitution = Instantiation::new(&mut context, &store, &[])?;
        let goal = context.instantiate_predicate(&mut store, value.predicate, &mut substitution)?;
        assert_eq!(
            registry.candidates(&mut store, &mut context, &goal, Availability::Executable)?,
            Candidates::Pending(vec![DefId(10)]),
        );
        registry.prepare(&mut store, ItemId(1), value)?;
        let before = context.steps;
        assert_eq!(
            registry.candidates(&mut store, &mut context, &goal, Availability::Executable)?,
            Candidates::Unvalidated(vec![DefId(10)]),
        );
        let work = context.steps - before;
        if let Some(expected) = query_work {
            assert_eq!(work, expected);
        } else {
            query_work = Some(work);
        }
        assert_eq!(
            registry.candidates(&mut store, &mut context, &goal, Availability::Validated)?,
            Candidates::Ready(Vec::new()),
        );
        assert!(!registry.is_validated(DefId(10)));
        registry.complete(DefId(10), Vec::new(), Vec::new(), Vec::new())?;
        assert_eq!(
            registry.candidates(&mut store, &mut context, &goal, Availability::Executable)?,
            Candidates::Ready(vec![DefId(10)]),
        );
        let unrelated = implementation(&mut store, 11, 3, TypeId::primitive(Primitive::U8))?;
        let mut substitution = Instantiation::new(&mut context, &store, &[])?;
        let goal =
            context.instantiate_predicate(&mut store, unrelated.predicate, &mut substitution)?;
        assert_eq!(
            registry.candidates(&mut store, &mut context, &goal, Availability::Executable)?,
            Candidates::Ready(Vec::new()),
        );
    }
    Ok(())
}

#[test]
fn pending_queries_and_retirement_do_not_walk_unrelated_headers()
-> Result<(), Box<dyn std::error::Error>> {
    let mut headers: Vec<_> = (0..4096)
        .map(|id| header(id, 1, Some(Head::Nominal(ItemId(id)))))
        .collect();
    headers.push(header(4096, 1, None));
    let mut pending = Pending::new(headers)?;
    let mut context = Context::default();
    for id in (0..4096).rev() {
        let before = context.steps;
        assert_eq!(
            pending.candidates(
                &mut context,
                ItemId(1),
                Some(Subject::Rigid(Head::Nominal(ItemId(id))))
            )?,
            [DefId(id), DefId(4096)]
        );
        assert_eq!(context.steps - before, 4);
        pending.complete(DefId(id));
    }
    assert_eq!(pending.owners.len(), 1);
    assert_eq!(pending.heads.len(), 1);
    pending.complete(DefId(4096));
    assert!(pending.is_empty());
    assert!(pending.traits.is_empty());
    assert!(pending.heads.is_empty());
    Ok(())
}

#[test]
fn unknown_projection_and_numeric_domains_keep_every_possible_candidate()
-> Result<(), Box<dyn std::error::Error>> {
    let pending = Pending::new(vec![
        header(9, 1, None),
        header(8, 1, Some(Head::Primitive(Primitive::F64))),
        header(7, 1, Some(Head::Primitive(Primitive::I32))),
        header(6, 1, Some(Head::Primitive(Primitive::U8))),
        header(5, 1, Some(Head::Nominal(ItemId(2)))),
        header(4, 2, None),
    ])?;
    let mut context = Context::default();
    assert_eq!(
        pending.candidates(&mut context, ItemId(1), Some(Subject::Integer))?,
        [DefId(6), DefId(7), DefId(9)]
    );
    assert_eq!(
        pending.candidates(&mut context, ItemId(1), Some(Subject::Float))?,
        [DefId(8), DefId(9)]
    );
    for subject in [None, Some(Subject::Rigid(Head::Projection(ItemId(2))))] {
        assert_eq!(
            pending.candidates(&mut context, ItemId(1), subject)?,
            [DefId(5), DefId(6), DefId(7), DefId(8), DefId(9)]
        );
    }
    assert!(pending.may_have_subject(ItemId(1), Subject::Rigid(Head::Projection(ItemId(2)))));
    assert!(
        pending
            .candidates(&mut context, ItemId(3), None)?
            .is_empty()
    );
    Ok(())
}

#[test]
fn header_publication_checks_reserved_identity_before_removing_dependencies()
-> Result<(), Box<dyn std::error::Error>> {
    let mut store = Store::default();
    let mut registry = Implementations::default();
    registry.reserve(vec![header(10, 1, Some(Head::Primitive(Primitive::U8)))])?;
    assert!(registry.reserve(Vec::new()).is_err());
    for (owner, item, ty) in [
        (10, 2, Primitive::U8),
        (10, 1, Primitive::U16),
        (11, 1, Primitive::U8),
    ] {
        let value = implementation(&mut store, owner, item, TypeId::primitive(ty))?;
        assert!(registry.prepare(&mut store, ItemId(item), value).is_err());
        assert_eq!(registry.headers().count(), 0);
        assert!(registry.finish_constant_headers().is_err());
    }
    let value = implementation(&mut store, 10, 1, TypeId::primitive(Primitive::U8))?;
    registry.prepare(&mut store, ItemId(1), value)?;
    registry.finish_constant_headers()?;
    assert!(registry.reserve(Vec::new()).is_err());
    let extra = implementation(&mut store, 11, 1, TypeId::primitive(Primitive::U8))?;
    assert!(registry.prepare(&mut store, ItemId(1), extra).is_err());
    assert!(
        Pending::new(vec![
            header(1, 1, None),
            header(1, 2, Some(Head::canonical(Kind::Never)))
        ])
        .is_err()
    );
    Ok(())
}

fn collection_reference(
    store: &mut Store,
    referent: TypeId,
    mutable: bool,
) -> Result<TypeId, &'static str> {
    store.intern(Kind::Reference {
        referent,
        mutable,
        region: crate::types::RegionId::STATIC,
    })
}

#[test]
fn latent_collection_headers_keep_unknown_goals_pending_and_closed_goals_disjoint()
-> Result<(), Box<dyn std::error::Error>> {
    use crate::{
        support::iteration::{Collection, Trait},
        types::{
            RegionId,
            infer::{Constructor, Domain},
        },
    };
    let report = crate::test_source::resolve("fn main(){for _ in &[1u8]{}}", true)?;
    let catalog = report
        .resolution
        .builtin_traits
        .collections()
        .ok_or("collection identities")?;
    let iteration = report
        .resolution
        .builtin_traits
        .iteration()
        .ok_or("iteration identities")?;
    let registry = collection_registry(&report)?;
    assert!(registry.constant_headers_ready());
    assert!(!registry.selection_headers_ready());
    let mut store = Store::default();
    let item = ItemId(iteration.get(Trait::IntoIterator).definition.0);
    let element = TypeId::primitive(Primitive::U8);
    let slice = store.intern(Kind::Slice(element))?;
    let shared_slice = collection_reference(&mut store, slice, false)?;
    let shared_scalar = collection_reference(&mut store, element, false)?;
    let shared_mut = collection_reference(&mut store, slice, true)?;
    let mut context = Context::default();
    for (ty, ready) in [
        (shared_scalar, true),
        (shared_mut, false),
        (shared_slice, false),
    ] {
        let value = implementation(&mut store, 0, item.0, ty)?.predicate;
        let mut substitution = Instantiation::new(&mut context, &store, &[])?;
        let goal = context.instantiate_predicate(&mut store, value, &mut substitution)?;
        assert_eq!(
            registry.selection_goal_ready(&store, &mut context, &goal)?,
            ready
        );
        assert_eq!(
            registry.selection_predicate_ready(&store, &mut context, value)?,
            ready
        );
    }
    for mutable in [false, true] {
        let unknown = context.variable(Domain::Any)?;
        let term = context.construct(
            Constructor::Reference {
                mutable,
                region: RegionId::STATIC,
            },
            &[unknown],
        )?;
        let pending = registry.pending.as_deref().ok_or("pending")?;
        let owners = catalog
            .declarations()
            .filter(|d| d.kind.trait_() == Some(Trait::IntoIterator))
            .map(|d| d.definition)
            .collect::<Vec<_>>();
        let expected = catalog
            .declarations()
            .filter(|d| d.kind.trait_() == Some(Trait::IntoIterator) && d.kind.mutable() == mutable)
            .map(|d| d.definition)
            .collect::<Vec<_>>();
        let mut found = owners.clone();
        pending.filter_collections(&store, &mut context, term, &mut found)?;
        assert_eq!(found, expected);
        let exact = context.known(&store, slice)?;
        context.equate(&store, unknown, exact)?;
        let mut found = owners;
        pending.filter_collections(&store, &mut context, term, &mut found)?;
        let kind = if mutable {
            Collection::MutableSlice
        } else {
            Collection::SharedSlice
        };
        assert_eq!(
            found,
            [catalog.declaration(kind).ok_or("slice owner")?.definition]
        );
    }
    // Closed absence above never consumed or negatively cached either header.
    let matching = implementation(&mut store, 0, item.0, shared_slice)?.predicate;
    assert!(!registry.selection_predicate_ready(&store, &mut context, matching)?);
    Ok(())
}

fn collection_registry(
    report: &crate::resolve::expansion::Report,
) -> Result<Implementations, Box<dyn std::error::Error>> {
    let catalog = report
        .resolution
        .builtin_traits
        .collections()
        .ok_or("collections")?;
    let iteration = report
        .resolution
        .builtin_traits
        .iteration()
        .ok_or("iteration")?;
    let mut headers = Vec::new();
    for d in catalog.declarations() {
        if let Some(t) = d.kind.trait_() {
            headers.push(Header {
                owner: d.definition,
                item: ItemId(iteration.get(t).definition.0).into(),
                head: Some(if d.kind.next() {
                    Head::Nominal(ItemId(
                        catalog.iterator(d.kind).ok_or("iterator nominal")?.0,
                    ))
                } else {
                    Head::Reference(d.kind.mutable())
                }),
            });
        }
    }
    let mut registry = Implementations::default();
    registry.reserve(headers)?;
    registry.register_collection_heads(report)?;
    registry.finish_constant_headers()?;
    registry.finish_validation()?;
    Ok(registry)
}

#[test]
fn canonical_range_heads_are_exact_cold_and_do_not_widen_pending_records()
-> Result<(), Box<dyn std::error::Error>> {
    let report = crate::test_source::resolve("fn main(){let r=1u8..4u8;let _=r.start;}", true)?;
    let canonical = report.resolution.builtin_traits.ranges().ok_or("ranges")?;
    let iteration = report
        .resolution
        .builtin_traits
        .iteration()
        .ok_or("iteration")?;
    let trait_ = iteration
        .get(crate::support::iteration::Trait::Iterator)
        .definition;
    let nominal = ItemId(canonical.nominal.0);
    let rows = canonical.implementations.map(|d| Header {
        owner: d.definition,
        item: ItemId(trait_.0).into(),
        head: Some(Head::Nominal(nominal)),
    });
    let mut pending = Pending::new(rows.to_vec())?;
    pending.install_ranges(&report)?;
    assert!(pending.only_collections());
    let heads = pending.collections.as_deref().ok_or("heads")?;
    assert!(heads.iterator.is_none() && heads.entries.is_empty());
    assert_eq!(
        heads.ranges.as_deref().ok_or("range heads")?.owners.len(),
        12
    );
    assert!(pending.install_ranges(&report).is_err());
    let mut wrong = rows;
    wrong[0].head = Some(Head::Primitive(Primitive::U8));
    let mut wrong = Pending::new(wrong.to_vec())?;
    assert!(wrong.install_ranges(&report).is_err());
    assert!(wrong.collections.is_none());
    let lookalike = crate::test_source::resolve("struct Range;fn main(){let _=Range;}", true)?;
    assert!(
        Pending::new(Vec::new())?
            .install_ranges(&lookalike)
            .is_err()
    );
    eprintln!(
        "range pending sizes owner={} heads={} range_heads={}",
        size_of::<Pending>(),
        size_of::<super::collections::Heads>(),
        size_of::<super::progression_heads::Heads>()
    );
    Ok(())
}
