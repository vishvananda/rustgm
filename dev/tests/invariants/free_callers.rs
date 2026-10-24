//! Identity summaries retain exact caller ownership, captures and replay work.
use super::{Environment, Summary};
use crate::types::{
    GenericArgs, ItemId, Kind, RegionId, Store, TypeId,
    arguments::Argument,
    parameters::{
        clauses::{Clause, Quantified},
        predicates::{Bound as Requirement, Predicate},
    },
    substitute::Bound,
    traits::Application,
};
use std::sync::Arc;

fn predicate(
    types: &mut Store,
    item: u32,
    regions: &[RegionId],
) -> Result<Predicate, &'static str> {
    let arguments: Vec<_> = regions.iter().copied().map(Argument::Lifetime).collect();
    let arguments = types.intern_arguments(&arguments)?;
    let application = types.intern_trait_application(Application {
        item: ItemId(item),
        arguments,
    })?;
    Ok(Predicate {
        ty: TypeId::UNIT,
        bound: Requirement::Trait(application),
    })
}
fn environment(
    types: &mut Store,
    width: u32,
    regions: u32,
) -> Result<Arc<Environment>, &'static str> {
    let regions: Vec<_> = (1..=regions).map(RegionId).collect();
    let clauses = (0..width)
        .map(|item| predicate(types, item, &regions))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Arc::new(
        Environment::new(types, &[])?.with_predicates(types, &clauses)?,
    ))
}
fn cached(environment: &Environment) -> Option<&Arc<Summary>> {
    environment.traits.as_ref()?.opening.get()
}

#[test]
fn exact_identity_summary_bounds_width_regions_queries_and_combined_replay()
-> Result<(), &'static str> {
    for (width, regions, queries) in [
        (8, 1, 8),
        (32, 1, 32),
        (128, 1, 128),
        (8, 8, 8),
        (8, 32, 8),
        (8, 128, 8),
        (8, 8, 32),
        (8, 8, 128),
        (32, 32, 32),
        (128, 128, 128),
    ] {
        let mut types = Store::default();
        let environment = environment(&mut types, width, regions)?;
        let mut cold = 0;
        let mut replay = 0;
        for query in 0..queries {
            let mut fold = Bound::open_regions(regions + 10 + query)?;
            assert!(environment.identity_region_opening(&mut types, &mut fold)?);
            assert_eq!(fold.next_region(), Some(regions + 10 + query));
            assert_eq!(fold.opened_regions().count(), regions as usize);
            assert!(fold.opened_regions().all(|(left, right)| left == right));
            if query == 0 {
                cold = fold.steps;
            } else {
                replay += fold.steps;
            }
        }
        assert!(cold <= (width as usize + 1) * (regions as usize + 1) * 32);
        assert_eq!(replay, (queries - 1) as usize * (regions as usize + 1));
        let retained = environment.opening_summary_allocation(&types);
        assert!(retained.bytes >= regions as usize * size_of::<RegionId>());
        assert!(retained.bytes <= 512 + regions as usize * size_of::<RegionId>());
        println!(
            "identity-opening width={width} regions={regions} queries={queries} cold={cold} replay={replay} bytes={}",
            retained.bytes
        );
    }
    Ok(())
}

#[test]
fn failed_replay_and_tiny_budget_charge_attempts_without_poisoning_the_summary()
-> Result<(), &'static str> {
    let mut types = Store::default();
    let environment = environment(&mut types, 8, 2)?;
    let mut opening = Bound::open_regions(10)?;
    assert!(environment.identity_region_opening(&mut types, &mut opening)?);
    let summary = Arc::clone(cached(&environment).ok_or("missing summary")?);
    let mut overlapping = Bound::open_regions(2)?;
    assert_eq!(
        environment.identity_region_opening(&mut types, &mut overlapping),
        Err("existing free lifetime overlaps the fresh region interval")
    );
    assert_eq!(overlapping.steps, 3);
    let mut exhausted = Bound::open_regions(10)?;
    exhausted.steps = 1_000_000;
    assert!(
        environment
            .identity_region_opening(&mut types, &mut exhausted)
            .is_err()
    );
    assert_eq!(exhausted.steps, 1_000_001);
    assert!(Arc::ptr_eq(
        &summary,
        cached(&environment).ok_or("lost summary")?
    ));
    let mut valid = Bound::open_regions(20)?;
    assert!(environment.identity_region_opening(&mut types, &mut valid)?);
    assert_eq!(valid.steps, 3);
    Ok(())
}

#[test]
fn foreign_store_fork_and_mutation_cannot_consume_another_owners_summary()
-> Result<(), &'static str> {
    let mut types = Store::default();
    let original = environment(&mut types, 8, 1)?;
    assert!(original.identity_region_opening(&mut types, &mut Bound::open_regions(10)?)?);
    let fork = Arc::new((*original).clone());
    assert!(!fork.identity_region_opening(&mut types, &mut Bound::open_regions(10)?)?);
    let mut foreign = Store::default();
    assert!(!original.identity_region_opening(&mut foreign, &mut Bound::open_regions(10)?)?);
    let extra = predicate(&mut types, 999, &[RegionId(5)])?;
    let changed = Arc::new((*original).clone().with_predicates(&types, &[extra])?);
    assert!(cached(&changed).is_none());
    let mut opening = Bound::open_regions(10)?;
    assert!(changed.identity_region_opening(&mut types, &mut opening)?);
    assert_eq!(opening.opened_regions().count(), 2);
    let mut substitution = Bound::new(vec![]);
    let mapped = Arc::new(original.substitute_regions(&mut types, &mut substitution)?);
    assert!(cached(&mapped).is_none());
    assert!(mapped.identity_region_opening(&mut types, &mut Bound::open_regions(10)?)?);
    Ok(())
}

#[test]
fn nested_locals_are_private_but_captures_and_producer_metadata_require_remapping()
-> Result<(), &'static str> {
    for captured in [false, true] {
        let mut types = Store::default();
        let region = RegionId::bound(u16::from(captured), 0)?;
        let reference = types.intern(Kind::Reference {
            mutable: false,
            region,
            referent: TypeId::UNIT,
        })?;
        let application = types.intern_trait_application(Application {
            item: ItemId(1),
            arguments: GenericArgs::EMPTY,
        })?;
        let clause = Clause::ForAll(types.intern_quantified(Quantified {
            bound_regions: 1,
            predicate: Predicate {
                ty: reference,
                bound: Requirement::Trait(application),
            },
        })?);
        let free = predicate(&mut types, 2, &[RegionId(1)])?;
        let environment =
            Arc::new(Environment::new(&types, &[])?.with_clauses(&types, &[clause, free.into()])?);
        assert_eq!(
            environment.identity_region_opening(&mut types, &mut Bound::open_regions(10)?)?,
            !captured
        );
        assert_eq!(
            cached(&environment)
                .ok_or("missing capture summary")?
                .captured,
            captured
        );
    }
    let mut types = Store::default();
    let original = environment(&mut types, 8, 1)?;
    let authored = Arc::new((*original).clone().with_body_aliases(vec![RegionId(1)])?);
    assert!(!authored.identity_region_opening(&mut types, &mut Bound::open_regions(10)?)?);
    Ok(())
}

#[test]
fn all_region_bearing_partitions_prevent_an_invalid_identity_shortcut() -> Result<(), &'static str>
{
    for partition in 0..4 {
        let mut types = Store::default();
        let original = environment(&mut types, 8, 1)?;
        let region = RegionId::bound(u16::from(partition == 3), 0)?;
        let reference = types.intern(Kind::Reference {
            mutable: false,
            region,
            referent: TypeId::UNIT,
        })?;
        let clause = if partition == 0 || partition >= 2 {
            Predicate {
                ty: reference,
                bound: Requirement::Copy,
            }
        } else {
            types.declare_projection(
                ItemId(100),
                crate::types::projection::Schema {
                    trait_item: ItemId(101),
                    sized: false,
                    copies: crate::types::copy::Properties::default(),
                },
            )?;
            let application = types.intern_trait_application(Application {
                item: ItemId(101),
                arguments: GenericArgs::EMPTY,
            })?;
            let projected =
                types.intern(Kind::Projection(crate::types::projection::Application {
                    item: ItemId(100),
                    subject: TypeId::UNIT,
                    trait_application: application,
                    arguments: GenericArgs::EMPTY,
                }))?;
            Predicate {
                ty: projected,
                bound: Requirement::ProjectionEqual(reference),
            }
        };
        let clauses = if partition == 3 {
            let clause = Clause::ForAll(types.intern_quantified(Quantified {
                bound_regions: 1,
                predicate: clause,
            })?);
            vec![clause]
        } else if partition == 2 {
            vec![]
        } else {
            vec![clause.into()]
        };
        let mut changed = (*original).clone().with_clauses(&types, &clauses)?;
        if partition == 2 {
            let equalities = crate::types::relations::RegionEqualities::default()
                .with_aliases(&[RegionId(3)])
                .map_err(|_| "invalid equality fixture")?;
            changed = changed.with_region_equalities(equalities);
        }
        let changed = Arc::new(changed);
        assert!(!changed.identity_region_opening(&mut types, &mut Bound::open_regions(10)?)?);
    }
    Ok(())
}

#[test]
fn failed_first_inspection_retains_no_summary_and_records_attempted_work()
-> Result<(), &'static str> {
    let mut types = Store::default();
    let source = environment(&mut types, 8, 1)?;
    let mut invalid = (*source).clone().with_predicates(
        &types,
        &[Predicate {
            ty: TypeId::UNIT,
            bound: Requirement::Copy,
        }],
    )?;
    Arc::make_mut(
        invalid
            .assumptions
            .as_mut()
            .ok_or("missing ordinary assumptions")?,
    )
    .sized
    .insert(TypeId(u32::MAX));
    let invalid = Arc::new(invalid);
    for _ in 0..2 {
        let mut fold = Bound::open_regions(10)?;
        assert!(
            invalid
                .identity_region_opening(&mut types, &mut fold)
                .is_err()
        );
        assert!(fold.steps > 8);
        assert!(cached(&invalid).is_none());
    }
    Ok(())
}

#[test]
fn mapped_index_rejects_a_same_shaped_foreign_environment_index() -> Result<(), &'static str> {
    use crate::types::parameters::premise_index::{ClauseIndex, Opening};
    let mut types = Store::default();
    let source = environment(&mut types, 8, 1)?;
    let other = environment(&mut types, 8, 1)?;
    assert!(!Arc::ptr_eq(&source, &other));
    let index = source
        .trait_index(&types)?
        .0
        .ok_or("source canonical index")?;
    let foreign = other
        .trait_index(&types)?
        .0
        .ok_or("other canonical index")?;
    assert!(!Arc::ptr_eq(&index, &foreign));
    assert_eq!(
        source.trait_clauses()?.collect::<Vec<_>>(),
        other.trait_clauses()?.collect::<Vec<_>>()
    );
    let opening = Arc::new(Opening::new(
        Arc::clone(&source),
        &mut types,
        vec![(RegionId(1), RegionId(1))],
    ));
    let mut work = 0;
    assert!(ClauseIndex::opened(foreign, Arc::clone(&opening), &types, &mut work).is_err());
    assert!(work > 0);
    let mut work = 0;
    assert!(ClauseIndex::opened(index, opening, &types, &mut work).is_ok());
    assert!(work > 0);
    Ok(())
}
