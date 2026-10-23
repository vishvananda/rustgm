//! Header waits retain inference, nested goals and completed proof nodes.
#[path = "selection_headers/normalization.rs"]
mod normalization;
use super::{Decision, Declarations, Frame, Search, continuation::Progress, tests};
use crate::{
    resolve::model::DefId,
    sema::traits::{
        assumptions::Assumptions,
        heads::Head,
        implementations::{Availability, Implementation, Implementations, PendingHeader},
    },
    types::{
        GenericArgs, ItemId, Kind, Primitive, TypeId,
        arguments::Argument,
        infer::{Context, Domain, Instantiation, arguments::Argument as Inferred},
        parameters::{Parameter, ParameterKind, predicates::Bound},
        traits::Application,
    },
};
use std::fmt::Write;

#[test]
fn nested_header_waits_keep_private_lifetimes_disjoint_from_declaration_allocations()
-> Result<(), Box<dyn std::error::Error>> {
    use crate::types::{RegionId, outlives::Queries, relations::Constraints};
    for depth in [8, 32, 128] {
        let mut source = String::from("trait B0<'a>{}impl<'a>B0<'a>for u8{}\n");
        for i in 1..=depth {
            writeln!(
                source,
                "trait B{i}<'a>{{}}impl<'a,T:B{}<'a>>B{i}<'a>for T{{}}",
                i - 1
            )?;
        }
        let (report, mut facts) = tests::declarations(&source)?;
        let traits: Vec<_> = (0..=depth)
            .map(|i| tests::definition(&report, &format!("B{i}")))
            .collect::<Result<_, _>>()?;
        let arguments = facts
            .types
            .intern_arguments(&[Argument::Lifetime(RegionId::STATIC)])?;
        let goal = tests::predicate(
            &mut facts,
            *traits.last().ok_or("missing root")?,
            TypeId::primitive(Primitive::U8),
            arguments,
        )?;
        let (mut context, goal) = tests::instantiate(&mut facts, goal)?;
        let assumptions = Assumptions::default();
        let mut delayed = delay(&mut facts, &traits)?;
        let mut search = Search::new(
            tests::caller(&assumptions, None),
            Availability::Declarations,
        );
        assert!(search.resume_at(1).is_err());
        let first = Frame::new(&facts, &mut context, &goal, 1)?;
        let mut progress = search.begin(&mut facts, first)?;
        let mut foreign = Vec::new();
        while let Progress::Pending(crate::sema::traits::selection::Dependency::Header(owner)) =
            progress
        {
            let first = search.query.allocations.end();
            if first > 1 {
                assert!(search.resume_at(first - 1).is_err());
            }
            stable_poll(&mut search, &mut facts, owner)?;
            foreign.push(first..first + 3);
            search.resume_at(first + 3)?;
            let index = delayed
                .iter()
                .position(|header| header.owner == owner)
                .ok_or("unknown pending header")?;
            publish(&mut facts, delayed.swap_remove(index))?;
            progress = search.poll(&mut facts)?;
        }
        let Progress::Ready(Decision::Proven(root)) = progress else {
            return Err("interleaved lifetime query was not proved".into());
        };
        assert!(search.resume_at(search.query.allocations.end()).is_err());
        let evidence = search.evidence(&facts, root)?;
        assert_eq!(evidence.proof.owned_regions().count(), depth + 1);
        let mut restore = evidence.proof.body_restoration()?;
        for interval in &foreign {
            for id in interval.clone() {
                assert!(!evidence.proof.owns_region(RegionId(id)));
                assert!(restore.region(RegionId(id), 0).is_err());
            }
        }
        let mut constraints =
            Constraints::new(&facts.types, &facts.lifetimes.variances, 0, 100_000)?;
        let mut queries = Queries::new(
            &facts.types,
            &facts.lifetimes.nominal_requirements,
            &facts.lifetimes.callables,
        );
        evidence.proof.require(&mut constraints, &mut queries)?;
        let identities = constraints.prove()?.identities;
        assert_eq!(identities.variables().len(), depth + 1);
        for id in foreign.into_iter().flatten() {
            assert!(identities.get(RegionId(id)).is_none());
        }
    }
    Ok(())
}

fn trait_item(facts: &Declarations, header: &Implementation) -> Result<ItemId, &'static str> {
    let Bound::Trait(application) = header.predicate.bound else {
        return Err("missing header trait");
    };
    Ok(facts
        .types
        .trait_application(application)
        .ok_or("invalid header trait")?
        .item)
}

pub(super) fn delay(
    facts: &mut Declarations,
    traits: &[DefId],
) -> Result<Vec<Implementation>, &'static str> {
    let mut registry = Implementations::default();
    let headers: Vec<_> = facts
        .generics
        .implementations()
        .map(|header| Implementation {
            owner: header.owner,
            source: header.source,
            predicate: header.predicate,
            methods: header.methods.clone(),
            types: header.types.clone(),
            constants: header.constants.clone(),
        })
        .collect();
    let mut seeds = Vec::new();
    for header in &headers {
        let item = trait_item(facts, header)?;
        let head = facts
            .types
            .kind(header.predicate.ty)
            .ok_or("missing header type")?;
        seeds.push(PendingHeader {
            owner: header.owner,
            item: item.into(),
            head: (!matches!(head, Kind::Parameter(_) | Kind::Projection(_)))
                .then_some(Head::canonical(head)),
        });
    }
    registry.reserve(seeds)?;
    let mut delayed = Vec::new();
    for header in headers {
        let item = trait_item(facts, &header)?;
        if traits.contains(&DefId(item.0)) {
            delayed.push(header);
        } else {
            registry.prepare(&mut facts.types, item, header)?;
        }
    }
    facts.generics.implementations = Some(Box::new(registry));
    Ok(delayed)
}

pub(super) fn publish(
    facts: &mut Declarations,
    header: Implementation,
) -> Result<(), &'static str> {
    let item = trait_item(facts, &header)?;
    facts
        .generics
        .implementations
        .as_mut()
        .ok_or("missing registry")?
        .prepare(&mut facts.types, item, header)
}

pub(super) fn stable_poll(
    search: &mut Search<'_>,
    facts: &mut Declarations,
    owner: DefId,
) -> Result<(), Box<dyn std::error::Error>> {
    let before = (
        search.query.steps,
        search.query.nodes.len(),
        search.query.active.len(),
    );
    for _ in 0..8 {
        assert_eq!(
            search.poll(facts)?,
            Progress::Pending(crate::sema::traits::selection::Dependency::Header(owner))
        );
        assert_eq!(
            (
                search.query.steps,
                search.query.nodes.len(),
                search.query.active.len()
            ),
            before
        );
    }
    Ok(())
}

#[test]
fn nested_conditions_resume_after_header_publication_without_replaying_ancestors()
-> Result<(), Box<dyn std::error::Error>> {
    for depth in [8, 32, 128] {
        let mut source = String::from(
            "trait B0{}impl B0 for u8{}
",
        );
        for i in 1..=depth {
            writeln!(source, "trait B{i}{{}}impl<T:B{}>B{i} for T{{}}", i - 1)?;
        }
        let (report, mut facts) = tests::declarations(&source)?;
        let mut traits = Vec::new();
        for i in 0..=depth {
            traits.push(tests::definition(&report, &format!("B{i}"))?);
        }
        let root = tests::predicate(
            &mut facts,
            *traits.last().ok_or("missing root")?,
            TypeId::primitive(Primitive::U8),
            GenericArgs::EMPTY,
        )?;
        let assumptions = Assumptions::default();
        let (mut context, goal) = tests::instantiate(&mut facts, root)?;
        let mut baseline = Search::new(
            tests::caller(&assumptions, None),
            Availability::Declarations,
        );
        let first = Frame::new(&facts, &mut context, &goal, 1)?;
        assert!(matches!(
            baseline.run(&mut facts, first)?,
            Decision::Proven(_)
        ));
        let mut delayed = delay(&mut facts, &traits)?;
        let mut search = Search::new(
            tests::caller(&assumptions, None),
            Availability::Declarations,
        );
        let first = Frame::new(&facts, &mut context, &goal, 1)?;
        let mut progress = search.begin(&mut facts, first)?;
        let mut waits = 0;
        while let Progress::Pending(crate::sema::traits::selection::Dependency::Header(owner)) =
            progress
        {
            stable_poll(&mut search, &mut facts, owner)?;
            let index = delayed
                .iter()
                .position(|header| header.owner == owner)
                .ok_or("unknown wait")?;
            publish(&mut facts, delayed.swap_remove(index))?;
            waits += 1;
            progress = search.poll(&mut facts)?;
        }
        assert!(matches!(progress, Progress::Ready(Decision::Proven(_))));
        assert_eq!(waits, depth + 1);
        assert!(delayed.is_empty());
        assert!(search.query.active.is_empty());
        assert_eq!(search.query.nodes.len(), baseline.query.nodes.len());
        assert!(
            search.query.steps <= baseline.query.steps + 32 * (depth + 1),
            "{} vs {} at {depth}",
            search.query.steps,
            baseline.query.steps
        );
    }
    Ok(())
}

#[test]
fn pending_alternative_does_not_commit_the_first_candidates_inference()
-> Result<(), Box<dyn std::error::Error>> {
    let (report, mut facts) =
        tests::declarations("trait A<T>{}impl A<u8>for u8{}impl A<u16>for u8{}")?;
    let item = tests::definition(&report, "A")?;
    let parameter = facts.types.intern_parameter(Parameter {
        owner: ItemId(1_000_000),
        index: 0,
        kind: ParameterKind::Type { sized: true },
    })?;
    let ty = facts.types.intern(Kind::Parameter(parameter))?;
    let arguments = facts.types.intern_arguments(&[Argument::Type(ty)])?;
    let root = tests::predicate(
        &mut facts,
        item,
        TypeId::primitive(Primitive::U8),
        arguments,
    )?;
    let mut context = Context::default();
    let unknown = context.variable(Domain::Any)?;
    let mut substitution = Instantiation::new(
        &mut context,
        &facts.types,
        &[(parameter, Inferred::Type(unknown))],
    )?;
    let goal = context.instantiate_predicate(&mut facts.types, root, &mut substitution)?;
    let mut delayed = delay(&mut facts, &[item])?;
    publish(&mut facts, delayed.pop().ok_or("missing first header")?)?;
    let header = delayed.pop().ok_or("missing alternative")?;
    let assumptions = Assumptions::default();
    let mut search = Search::new(
        tests::caller(&assumptions, None),
        Availability::Declarations,
    );
    let first = Frame::new(&facts, &mut context, &goal, 1)?;
    assert_eq!(
        search.begin(&mut facts, first)?,
        Progress::Pending(crate::sema::traits::selection::Dependency::Header(
            header.owner
        ))
    );
    stable_poll(&mut search, &mut facts, header.owner)?;
    assert!(search.query.nodes.is_empty());
    assert!(search.query.completed.is_empty());
    publish(&mut facts, header)?;
    assert_eq!(
        search.poll(&mut facts)?,
        Progress::Ready(Decision::Deferred)
    );
    assert!(matches!(
        context.resolved(&mut facts.types, unknown),
        Err(crate::types::infer::Error::Unresolved)
    ));
    Ok(())
}

#[test]
fn input_normalization_suspends_for_its_actual_trait_premise()
-> Result<(), Box<dyn std::error::Error>> {
    let (report, mut facts) = tests::declarations(
        "trait P{type Out;}impl P for u8{type Out=u16;}trait A{}impl A for u16{}",
    )?;
    let item = tests::definition(&report, "P")?;
    let application = facts.types.intern_trait_application(Application {
        item: ItemId(item.0),
        arguments: GenericArgs::EMPTY,
    })?;
    let ty = facts
        .types
        .intern(Kind::Projection(crate::types::projection::Application {
            item: ItemId(tests::definition(&report, "Out")?.0),
            subject: TypeId::primitive(Primitive::U8),
            trait_application: application,
            arguments: GenericArgs::EMPTY,
        }))?;
    let root = tests::predicate(
        &mut facts,
        tests::definition(&report, "A")?,
        ty,
        GenericArgs::EMPTY,
    )?;
    let mut delayed = delay(&mut facts, &[item])?;
    let header = delayed.pop().ok_or("missing projected prerequisite")?;
    let assumptions = Assumptions::default();
    let (mut context, goal) = tests::instantiate(&mut facts, root)?;
    let mut search = Search::new(
        tests::caller(&assumptions, None),
        Availability::Declarations,
    );
    let first = Frame::new(&facts, &mut context, &goal, 1)?;
    assert_eq!(
        search.begin(&mut facts, first)?,
        Progress::Pending(crate::sema::traits::selection::Dependency::Header(
            header.owner
        ))
    );
    stable_poll(&mut search, &mut facts, header.owner)?;
    publish(&mut facts, header)?;
    let Progress::Ready(Decision::Proven(root)) = search.poll(&mut facts)? else {
        return Err("normalization did not resume after its header dependency".into());
    };
    let proof = search.evidence(&facts, root)?;
    assert!(matches!(proof.source, super::Source::Implementation { .. }));
    Ok(())
}
