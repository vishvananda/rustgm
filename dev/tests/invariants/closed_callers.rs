//! Opening a request preserves a closed caller's shared permissions and local binders.
use super::{Availability, Declarations, Environment, Predicate, prepare};
use crate::sema::traits::assumptions::Selection;
use crate::types::{
    GenericArgs, ItemId, RegionId, Store, TypeId,
    arguments::Argument,
    infer::{self, Instantiation},
    parameters::{
        clauses::{Clause, Quantified},
        predicates::Bound,
    },
    traits::Application,
};
use std::sync::Arc;

struct NoFiles;
impl crate::resolve::load::Sources for NoFiles {
    fn read(
        &mut self,
        _: &std::path::Path,
    ) -> Result<Option<crate::resolve::load::InputFile>, String> {
        Ok(None)
    }
}

fn empty_declarations() -> Result<Declarations, &'static str> {
    let input = crate::resolve::load::InputFile {
        identity: "opening.rs".into(),
        path: "opening.rs".into(),
        source: crate::support::source::SourceFile::new(Vec::new())
            .map_err(|_| "empty test source failed")?,
    };
    let expanded = crate::resolve::expansion::analyze(
        input,
        &mut NoFiles,
        &crate::expand::config::Config::default(),
    )
    .map_err(|_| "empty test crate expansion failed")?;
    let report = crate::resolve::analyze_expanded(expanded)
        .map_err(|_| "empty test crate resolution failed")?;
    Declarations::new(&report).map_err(|_| "empty test declarations failed")
}

fn predicate(store: &mut Store, item: u32, region: RegionId) -> Result<Predicate, &'static str> {
    let arguments = store.intern_arguments(&[Argument::Lifetime(region)])?;
    let application = store.intern_trait_application(Application {
        item: ItemId(item),
        arguments,
    })?;
    Ok(Predicate {
        ty: TypeId::UNIT,
        bound: Bound::Trait(application),
    })
}

fn assert_absent(
    facts: &mut Declarations,
    state: &mut super::State,
    predicate: Predicate,
) -> Result<(), infer::Error> {
    let mut substitution = Instantiation::new(&mut state.context, &facts.types, &[])?;
    let goal =
        state
            .context
            .instantiate_predicate(&mut facts.types, predicate, &mut substitution)?;
    assert!(matches!(
        state.assumptions.select_scoped(
            &mut facts.types,
            &mut state.context,
            &goal,
            state.obligations.regions.end,
        )?,
        Selection::Absent
    ));
    Ok(())
}

#[test]
fn open_requests_share_closed_quantified_environments_and_keep_private_aliases()
-> Result<(), &'static str> {
    let mut opening_work = None;
    for width in [16, 64, 256, 1024] {
        let mut facts = empty_declarations()?;
        let mut clauses = Vec::new();
        for item in 0..width {
            let predicate = predicate(&mut facts.types, item, RegionId::bound(0, 0)?)?;
            clauses.push(Clause::ForAll(facts.types.intern_quantified(
                Quantified {
                    bound_regions: 1,
                    predicate,
                },
            )?));
        }
        let environment =
            Arc::new(Environment::new(&facts.types, &[])?.with_clauses(&facts.types, &clauses)?);
        assert!(!environment.requires_region_mapping());
        let surrounding = RegionId::bound(0, 0)?;
        let requested = predicate(&mut facts.types, width, surrounding)?;
        for (request, first) in [100, 200, 300].into_iter().enumerate() {
            let (mut state, opened, arguments) = prepare(
                &mut facts,
                Some(environment.clone()),
                (requested, GenericArgs::EMPTY),
                first,
                false,
                Availability::Declarations,
            )?;
            assert_eq!(arguments, GenericArgs::EMPTY);
            assert!(Arc::ptr_eq(
                state.environment.as_ref().ok_or("missing caller")?,
                &environment
            ));
            assert_eq!(opened, predicate(&mut facts.types, width, RegionId(first))?);
            assert_eq!(state.obligations.regions, first..first + 1);
            assert_eq!(
                state.obligations.equalities,
                [(RegionId(first), surrounding)]
            );
            // Index construction is charged once, including all caller clauses.
            // Later queries only borrow that index and perform their own opening.
            let construction = if request == 0 { width as usize } else { 0 };
            let query_work = state.context.steps - construction;
            assert_eq!(*opening_work.get_or_insert(query_work), query_work);
            // The request cannot become a caller assumption merely by opening it.
            assert_absent(&mut facts, &mut state, opened)
                .map_err(|_| "invalid scoped assumption lookup")?;
        }
        assert_eq!(environment.trait_clauses()?.collect::<Vec<_>>(), clauses);
    }
    Ok(())
}

#[test]
fn captured_environments_still_share_the_requests_mapping_without_mutating_the_caller()
-> Result<(), &'static str> {
    let mut facts = empty_declarations()?;
    let surrounding = RegionId::bound(0, 0)?;
    let original = predicate(&mut facts.types, 1, surrounding)?;
    let environment =
        Arc::new(Environment::new(&facts.types, &[])?.with_predicates(&facts.types, &[original])?);
    assert!(environment.requires_region_mapping());
    let (state, opened, arguments) = prepare(
        &mut facts,
        Some(environment.clone()),
        (original, GenericArgs::EMPTY),
        100,
        false,
        Availability::Declarations,
    )?;
    assert_eq!(arguments, GenericArgs::EMPTY);
    let mapped = state
        .environment
        .as_ref()
        .ok_or("missing mapped environment")?;
    assert!(!Arc::ptr_eq(mapped, &environment));
    assert!(mapped.has_trait_opening());
    assert!(mapped.trait_clauses().is_err());
    assert_eq!(opened, predicate(&mut facts.types, 1, RegionId(100))?);
    // A later non-Open consumer explicitly materializes the current namespace.
    let materialized = mapped.substitute_regions(
        &mut facts.types,
        &mut crate::types::substitute::Bound::shift(0),
    )?;
    assert_eq!(
        materialized.trait_clauses()?.collect::<Vec<_>>(),
        [Clause::Predicate(opened)]
    );
    assert_eq!(
        environment.trait_clauses()?.collect::<Vec<_>>(),
        [Clause::Predicate(original)]
    );
    assert_eq!(state.obligations.regions, 100..101);
    assert_eq!(state.obligations.equalities, [(RegionId(100), surrounding)]);
    Ok(())
}

#[test]
fn family_and_parent_aliases_share_one_mapping_without_opening_callable_locals()
-> Result<(), &'static str> {
    use crate::types::Kind;
    for shared_parent in [false, true] {
        let mut facts = empty_declarations()?;
        let outer = RegionId::bound(0, 0)?;
        let other = RegionId::bound(0, 1)?;
        let parent = predicate(
            &mut facts.types,
            1,
            if shared_parent {
                outer
            } else {
                RegionId::STATIC
            },
        )?;
        let local = facts.types.intern(Kind::Reference {
            region: RegionId::bound(0, 0)?,
            mutable: false,
            referent: TypeId::UNIT,
        })?;
        let captured = facts.types.intern(Kind::Reference {
            region: RegionId::bound(1, 0)?,
            mutable: false,
            referent: TypeId::UNIT,
        })?;
        let parameters = facts.types.intern_list(&[local, captured])?;
        let function = facts.types.intern(Kind::Function {
            parameters,
            result: TypeId::UNIT,
            bound_regions: 1,
            unsafe_: false,
            convention: crate::types::callable::Convention::RUST,
        })?;
        let own = facts.types.intern_arguments(&[
            Argument::Lifetime(outer),
            Argument::Lifetime(other),
            Argument::Type(function),
        ])?;
        let environment = Arc::new(Environment::new(&facts.types, &[])?);
        let (state, opened, arguments) = prepare(
            &mut facts,
            Some(Arc::clone(&environment)),
            (parent, own),
            100,
            false,
            Availability::Declarations,
        )?;
        assert_eq!(
            opened,
            predicate(
                &mut facts.types,
                1,
                if shared_parent {
                    RegionId(100)
                } else {
                    RegionId::STATIC
                }
            )?
        );
        assert!(Arc::ptr_eq(
            state.environment.as_ref().ok_or("missing caller")?,
            &environment
        ));
        assert_eq!(state.obligations.regions, 100..102);
        assert_eq!(
            state.obligations.equalities,
            [(RegionId(100), outer), (RegionId(101), other)]
        );
        let captured = facts.types.intern(Kind::Reference {
            region: RegionId(100),
            mutable: false,
            referent: TypeId::UNIT,
        })?;
        let parameters = facts.types.intern_list(&[local, captured])?;
        let function = facts.types.intern(Kind::Function {
            parameters,
            result: TypeId::UNIT,
            bound_regions: 1,
            unsafe_: false,
            convention: crate::types::callable::Convention::RUST,
        })?;
        let expected = facts.types.intern_arguments(&[
            Argument::Lifetime(RegionId(100)),
            Argument::Lifetime(RegionId(101)),
            Argument::Type(function),
        ])?;
        assert_eq!(arguments, expected);
        let mut restore = state.finish().restoration()?;
        assert_eq!(restore.apply_arguments(&mut facts.types, arguments)?, own);
    }
    Ok(())
}

#[test]
fn free_callers_share_structure_but_requests_arguments_and_restoration_stay_fresh()
-> Result<(), &'static str> {
    for width in [8, 32, 128] {
        let mut facts = empty_declarations()?;
        let clauses = (0..width)
            .map(|item| predicate(&mut facts.types, item, RegionId(1)))
            .collect::<Result<Vec<_>, _>>()?;
        let environment =
            Arc::new(Environment::new(&facts.types, &[])?.with_predicates(&facts.types, &clauses)?);
        let original = predicate(&mut facts.types, width, RegionId::bound(0, 0)?)?;
        let own = facts.types.intern_arguments(&[
            Argument::Lifetime(RegionId(1)),
            Argument::Lifetime(RegionId::bound(0, 0)?),
        ])?;
        let mut shared_index = None;
        let mut warm_work = None;
        for query in 0..128 {
            let first = 1000 + query * 2;
            let (state, opened, args) = prepare(
                &mut facts,
                Some(Arc::clone(&environment)),
                (original, own),
                first,
                false,
                Availability::Declarations,
            )?;
            assert!(Arc::ptr_eq(
                state
                    .environment
                    .as_ref()
                    .ok_or("missing identity caller")?,
                &environment
            ));
            assert_eq!(state.obligations.regions, first..first + 1);
            assert_eq!(
                state.obligations.equalities,
                [
                    (RegionId(1), RegionId(1)),
                    (RegionId(first), RegionId::bound(0, 0)?)
                ]
            );
            let index = environment
                .trait_index(&facts.types)?
                .0
                .ok_or("missing trait index")?;
            if let Some(shared) = &shared_index {
                assert!(Arc::ptr_eq(shared, &index));
            } else {
                shared_index = Some(index);
            }
            if query > 0 {
                assert_eq!(
                    *warm_work.get_or_insert(state.context.steps),
                    state.context.steps
                );
            }
            let mut restoration = state.finish().restoration()?;
            assert_eq!(restoration.predicate(&mut facts.types, opened)?, original);
            assert_eq!(restoration.apply_arguments(&mut facts.types, args)?, own);
        }
        assert!(environment.opening_summary_allocation(&facts.types).bytes > 0);
    }
    Ok(())
}

#[test]
fn closed_ordinary_callers_need_no_opening_cache() -> Result<(), &'static str> {
    let mut facts = empty_declarations()?;
    let application = facts.types.intern_trait_application(Application {
        item: ItemId(1),
        arguments: GenericArgs::EMPTY,
    })?;
    let clause = Predicate {
        ty: TypeId::UNIT,
        bound: Bound::Trait(application),
    };
    let environment =
        Arc::new(Environment::new(&facts.types, &[])?.with_predicates(&facts.types, &[clause])?);
    for _ in 0..8 {
        let (state, opened, arguments) = prepare(
            &mut facts,
            Some(Arc::clone(&environment)),
            (clause, GenericArgs::EMPTY),
            1,
            false,
            Availability::Declarations,
        )?;
        assert!(Arc::ptr_eq(
            state.environment.as_ref().ok_or("missing closed caller")?,
            &environment
        ));
        assert_eq!(opened, clause);
        assert_eq!(arguments, GenericArgs::EMPTY);
        assert!(state.obligations.equalities.is_empty());
        assert_eq!(
            environment.opening_summary_allocation(&facts.types).bytes,
            0
        );
    }
    Ok(())
}

#[test]
fn actual_original_opening_work_retains_width_sites_and_noise_controls()
-> Result<(), Box<dyn std::error::Error>> {
    let mut widths = Vec::new();
    for (axis, count, noise) in [
        ("subjects", 8, 0),
        ("subjects", 32, 0),
        ("subjects", 128, 0),
        ("sites", 8, 0),
        ("sites", 32, 0),
        ("sites", 128, 0),
        ("subjects", 8, 128),
    ] {
        let path = format!(
            "pa31/tests/spec/trait-default-effects/projections/body-opening/opening-{axis}-{count}-noise-{noise}.rs"
        );
        let source = crate::test_source::fixture(&path)?;
        let report = crate::test_source::source_provider(&source, "opening_work")?;
        let crate::pipeline::Analysis {
            mut facts,
            prepared,
        } = crate::pipeline::analyze_source(&report)?;
        let work = crate::lower::instances::inspect_prepared(
            &report,
            &mut facts,
            prepared,
            |facts, _| facts.body_work.canonical_steps,
        )?;
        println!(
            "actual-original-opening axis={axis} count={count} noise={noise} canonical_steps={work}"
        );
        if axis == "subjects" && noise == 0 {
            widths.push(work);
        }
    }
    assert_eq!(widths.len(), 3);
    for pair in widths.windows(2) {
        assert!(pair[1] <= pair[0].saturating_mul(6));
    }
    assert_eq!(size_of::<Environment>(), 64);
    assert_eq!(size_of::<super::State>(), 800);
    assert_eq!(size_of::<crate::body::ir::Body>(), 456);
    assert_eq!(size_of::<crate::body::regions::Contract>(), 184);
    Ok(())
}

#[test]
fn distinct_captured_query_views_share_canonical_buckets_but_keep_their_full_maps()
-> Result<(), &'static str> {
    let mut facts = empty_declarations()?;
    let original = predicate(&mut facts.types, 1, RegionId::bound(0, 0)?)?;
    let caller =
        Arc::new(Environment::new(&facts.types, &[])?.with_predicates(&facts.types, &[original])?);
    let mut prior_index = None;
    for first in [100, 200] {
        let (mut state, opened, _) = prepare(
            &mut facts,
            Some(Arc::clone(&caller)),
            (original, GenericArgs::EMPTY),
            first,
            false,
            Availability::Declarations,
        )?;
        let environment = state
            .environment
            .as_ref()
            .ok_or("missing mapped caller")?
            .clone();
        assert!(environment.has_trait_opening());
        assert!(environment.trait_clauses().is_err());
        let index = environment
            .trait_index(&facts.types)?
            .0
            .ok_or("missing mapped index")?;
        if let Some(prior) = prior_index {
            assert!(!Arc::ptr_eq(&index, &prior));
            assert!(std::ptr::eq(index.canonical(), prior.canonical()));
        }
        prior_index = Some(Arc::clone(&index));
        assert!(state.prove(&mut facts, opened, false)?);
        // Search's builtin child proof reconstructs assumptions through exactly
        // this environment; it must not receive empty materialized maps.
        let mut child = crate::sema::nominals::Proof::new(Some(Arc::clone(&environment)));
        assert!(child.prove(&mut facts, opened)?);
        let missing = predicate(&mut facts.types, 2, RegionId(first))?;
        assert!(!child.prove(&mut facts, missing)?);
        assert_eq!(state.obligations.regions, first..first + 1);
        assert_eq!(
            state.obligations.equalities,
            [(RegionId(first), RegionId::bound(0, 0)?)]
        );
        let materialized = environment.substitute_regions(
            &mut facts.types,
            &mut crate::types::substitute::Bound::shift(0),
        )?;
        assert!(!materialized.has_trait_opening());
        assert_eq!(
            materialized.trait_clauses()?.collect::<Vec<_>>(),
            [Clause::Predicate(opened)]
        );
        assert_eq!(
            caller.trait_clauses()?.collect::<Vec<_>>(),
            [Clause::Predicate(original)]
        );
    }
    Ok(())
}

#[test]
fn captured_view_checks_arena_budget_and_later_full_materialization() -> Result<(), &'static str> {
    let mut facts = empty_declarations()?;
    let original = predicate(&mut facts.types, 1, RegionId::bound(0, 0)?)?;
    let caller =
        Arc::new(Environment::new(&facts.types, &[])?.with_predicates(&facts.types, &[original])?);
    let (state, opened, _) = prepare(
        &mut facts,
        Some(Arc::clone(&caller)),
        (original, GenericArgs::EMPTY),
        100,
        false,
        Availability::Declarations,
    )?;
    let environment = state
        .environment
        .as_ref()
        .ok_or("missing opened environment")?;
    let index = environment
        .trait_index(&facts.types)?
        .0
        .ok_or("missing opened index")?;
    let view = index.opening().ok_or("missing actual view")?;
    let mut foreign = Store::default();
    let mut context = crate::types::infer::Context::default();
    assert!(matches!(
        view.predicate(&mut foreign, &mut context, original),
        Err(infer::Error::InvalidIdentity)
    ));
    assert!(context.steps > 0);
    let mut context = crate::types::infer::Context::default();
    context.steps = 1_000_000;
    assert!(matches!(
        view.predicate(&mut facts.types, &mut context, original),
        Err(infer::Error::Budget)
    ));
    assert!(context.steps > 1_000_000);
    let mut context = crate::types::infer::Context::default();
    assert_eq!(
        view.predicate(&mut facts.types, &mut context, original)
            .map_err(|_| "valid caller mapping failed")?,
        opened
    );
    let materialized = environment.substitute_regions(
        &mut facts.types,
        &mut crate::types::substitute::Bound::shift(1),
    )?;
    assert_eq!(
        materialized.trait_clauses()?.collect::<Vec<_>>(),
        [Clause::Predicate(opened)]
    );
    assert!(!materialized.has_trait_opening());
    assert!(
        environment
            .clone()
            .as_ref()
            .clone()
            .with_predicates(&facts.types, &[opened])
            .is_err()
    );
    Ok(())
}

#[test]
fn unrequested_captures_and_quantified_clauses_use_the_complete_eager_fallback()
-> Result<(), &'static str> {
    let mut facts = empty_declarations()?;
    let original = predicate(&mut facts.types, 1, RegionId::bound(0, 0)?)?;
    let request = predicate(&mut facts.types, 2, RegionId::STATIC)?;
    let caller =
        Arc::new(Environment::new(&facts.types, &[])?.with_predicates(&facts.types, &[original])?);
    let (state, opened, _) = prepare(
        &mut facts,
        Some(caller),
        (request, GenericArgs::EMPTY),
        100,
        false,
        Availability::Declarations,
    )?;
    let environment = state
        .environment
        .as_ref()
        .ok_or("missing fallback caller")?;
    assert!(!environment.has_trait_opening());
    assert_eq!(opened, request);
    assert_eq!(
        environment.trait_clauses()?.collect::<Vec<_>>(),
        [Clause::Predicate(predicate(
            &mut facts.types,
            1,
            RegionId(100)
        )?)]
    );
    assert_eq!(
        state.obligations.equalities,
        [(RegionId(100), RegionId::bound(0, 0)?)]
    );
    let captured = predicate(&mut facts.types, 1, RegionId::bound(1, 0)?)?;
    let clause = Clause::ForAll(facts.types.intern_quantified(Quantified {
        bound_regions: 1,
        predicate: captured,
    })?);
    let caller =
        Arc::new(Environment::new(&facts.types, &[])?.with_clauses(&facts.types, &[clause])?);
    let (state, _, _) = prepare(
        &mut facts,
        Some(caller),
        (original, GenericArgs::EMPTY),
        200,
        false,
        Availability::Declarations,
    )?;
    let environment = state
        .environment
        .as_ref()
        .ok_or("missing quantified caller")?;
    assert!(!environment.has_trait_opening());
    assert_eq!(environment.trait_clauses()?.count(), 1);
    Ok(())
}

#[test]
fn captured_index_views_bound_clause_width_query_count_and_actual_region_replay()
-> Result<(), &'static str> {
    let mut width_work = Vec::new();
    let mut width_bytes = Vec::new();
    for (width, regions, queries) in [
        (8, 1, 8),
        (32, 1, 32),
        (128, 1, 128),
        (8, 8, 8),
        (8, 32, 8),
        (8, 128, 8),
        (8, 8, 32),
        (8, 8, 128),
    ] {
        let mut facts = empty_declarations()?;
        let lifetime_inputs = (0..regions)
            .map(|slot| RegionId::bound(0, slot).map(Argument::Lifetime))
            .collect::<Result<Vec<_>, _>>()?;
        let arguments = facts.types.intern_arguments(&lifetime_inputs)?;
        let mut clauses = Vec::new();
        for item in 0..width {
            let application = facts.types.intern_trait_application(Application {
                item: ItemId(item),
                arguments,
            })?;
            clauses.push(Predicate {
                ty: TypeId::UNIT,
                bound: Bound::Trait(application),
            });
        }
        let caller =
            Arc::new(Environment::new(&facts.types, &[])?.with_predicates(&facts.types, &clauses)?);
        let mut observed = crate::types::parameters::premise_index::Retained::default();
        let mut indexes: Vec<Arc<crate::types::parameters::premise_index::ClauseIndex>> =
            Vec::new();
        let mut selected_work = 0;
        let mut retained_bytes = 0;
        let before = facts.body_work.canonical_steps;
        for query in 0..queries {
            let original = clauses[query as usize % clauses.len()];
            let first = 1000 + query * (u32::from(regions) + 1);
            let (mut state, opened, _) = prepare(
                &mut facts,
                Some(Arc::clone(&caller)),
                (original, GenericArgs::EMPTY),
                first,
                false,
                Availability::Declarations,
            )?;
            let environment = state
                .environment
                .as_ref()
                .ok_or("missing mapped scale caller")?
                .clone();
            assert!(environment.has_trait_opening());
            assert!(state.prove(&mut facts, opened, false)?);
            selected_work += state.context.steps;
            let index = environment
                .trait_index(&facts.types)?
                .0
                .ok_or("missing mapped scale index")?;
            if let Some(previous) = indexes.last() {
                assert!(std::ptr::eq(index.canonical(), previous.canonical()));
            }
            retained_bytes += index.retain_allocation(&mut observed).bytes;
            indexes.push(index);
            assert_eq!(state.obligations.regions, first..first + u32::from(regions));
        }
        let preparation = facts.body_work.canonical_steps - before;
        println!(
            "captured-index width={width} regions={regions} queries={queries} preparation={preparation} selection={selected_work} retained={retained_bytes}"
        );
        assert!(
            preparation
                <= (width as usize + queries as usize + 1) * (usize::from(regions) + 1) * 128
        );
        if regions == 1 && width == queries {
            width_work.push(preparation + selected_work);
            width_bytes.push(retained_bytes);
        }
    }
    for values in [width_work, width_bytes] {
        for pair in values.windows(2) {
            assert!(pair[1] <= pair[0].saturating_mul(6));
        }
    }
    Ok(())
}

#[test]
fn mapped_premise_alternatives_preserve_ambiguity_and_unknown_subject_deferral()
-> Result<(), &'static str> {
    let mut facts = empty_declarations()?;
    let original = predicate(&mut facts.types, 1, RegionId::bound(0, 0)?)?;
    let other = predicate(&mut facts.types, 1, RegionId::STATIC)?;
    let caller = Arc::new(
        Environment::new(&facts.types, &[])?.with_predicates(&facts.types, &[original, other])?,
    );
    for first in [100, 200] {
        let (mut state, opened, _) = prepare(
            &mut facts,
            Some(Arc::clone(&caller)),
            (original, GenericArgs::EMPTY),
            first,
            false,
            Availability::Declarations,
        )?;
        assert!(
            state
                .environment
                .as_ref()
                .ok_or("missing ambiguous view")?
                .has_trait_opening()
        );
        // Exact nominal membership remains positive before any general trial;
        // the general inference selection below must still report ambiguity.
        let mut child = crate::sema::nominals::Proof::new(state.environment.clone());
        assert!(child.prove(&mut facts, opened)?);
        let mut substitution = Instantiation::new(&mut state.context, &facts.types, &[])
            .map_err(|_| "invalid test instantiation")?;
        let goal = state
            .context
            .instantiate_predicate(&mut facts.types, opened, &mut substitution)
            .map_err(|_| "invalid test goal")?;
        assert!(matches!(
            state
                .assumptions
                .select_scoped(&mut facts.types, &mut state.context, &goal, first + 1)
                .map_err(|_| "invalid ambiguous lookup")?,
            Selection::Deferred
        ));
        let unknown = state
            .context
            .variable(infer::Domain::Any)
            .map_err(|_| "unknown test subject")?;
        let goal = goal
            .map_arguments(|argument| {
                Ok(match argument {
                    crate::types::infer::arguments::Argument::Type(_) => {
                        crate::types::infer::arguments::Argument::Type(unknown)
                    }
                    argument => argument,
                })
            })
            .map_err(|_| "invalid open test goal")?;
        assert!(matches!(
            state
                .assumptions
                .select_scoped(&mut facts.types, &mut state.context, &goal, first + 1)
                .map_err(|_| "invalid deferred lookup")?,
            Selection::Deferred
        ));
    }
    Ok(())
}
