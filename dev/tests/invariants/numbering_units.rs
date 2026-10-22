//! Canonical insertion bounds preorder queries without forbidding later empty joins.
use super::*;
use crate::{resolve::unit::FileId, sema::model::BodyRegionOrigin, syntax::ast::NodeId};
use std::cell::Cell;

thread_local! {
    static PARENT_EDGES: Cell<usize> = const { Cell::new(0) };
}

pub(super) fn followed_parent() {
    PARENT_EDGES.with(|edges| edges.set(edges.get() + 1));
}

fn key(index: u32) -> NodeKey {
    NodeKey {
        file: FileId(0),
        node: NodeId::new(index),
    }
}

fn annotation(body: NodeKey) -> BodyRegion {
    BodyRegion {
        body,
        source: body,
        origin: BodyRegionOrigin::Annotation,
    }
}

#[test]
fn preorder_depth_times_regions_has_bounded_lookup_work() -> Result<(), &'static str> {
    for depth in [8_u32, 32, 128] {
        for regions_per_body in [1, 4, 16] {
            let mut regions = BodyRegions::default();
            PARENT_EDGES.with(|edges| edges.set(0));
            for index in 1..=depth {
                regions.nest(key(index), key(index - 1))?;
                assert_eq!(regions.parents.get(&key(index)), Some(&key(0)));
            }
            for index in 1..=depth {
                let body = key(index);
                for _ in 0..regions_per_body {
                    let id = regions.allocate(annotation(body))?;
                    let fact = regions.get(body, id).ok_or("missing own region")?;
                    assert_eq!((fact.body, fact.source), (body, body));
                    assert_eq!(regions.frontier(body)?, id.0 + 1);
                    assert_eq!(regions.get_mut(body, id).map(|fact| fact.body), Some(body));
                    // The unit root finds the same identity, without changing its owner.
                    assert_eq!(regions.get(key(0), id).map(|fact| fact.body), Some(body));
                }
            }
            let visits = PARENT_EDGES.with(Cell::get);
            let allocations = depth as usize * regions_per_body;
            assert!(visits <= 2 * depth as usize + 4 * allocations);
            assert_eq!(regions.log().len(), allocations);
        }
    }
    Ok(())
}

#[test]
fn later_empty_unit_nesting_keeps_old_roots_as_aliases() -> Result<(), &'static str> {
    let mut regions = BodyRegions::default();
    regions.nest(key(2), key(1))?;
    regions.nest(key(1), key(0))?;
    assert_eq!(regions.unit_of(key(2)), key(0));
    assert_eq!(regions.parents.get(&key(2)), Some(&key(1)));
    // New children use the resolved unit, while the existing alias remains valid.
    regions.nest(key(3), key(2))?;
    assert_eq!(regions.parents.get(&key(3)), Some(&key(0)));
    let id = regions.allocate(annotation(key(2)))?;
    for body in [key(0), key(1), key(2), key(3)] {
        let fact = regions.get(body, id).ok_or("alias lost its unit")?;
        assert_eq!((fact.body, fact.source), (key(2), key(2)));
        assert!(matches!(fact.origin, BodyRegionOrigin::Annotation));
        assert_eq!(regions.frontier(body)?, id.0 + 1);
    }
    regions.nest(key(2), key(0))?;
    regions.nest(key(0), key(2))?;
    assert_eq!(regions.unit_of(key(2)), key(0));
    assert_eq!(
        regions.nest(key(1), key(9)),
        Err("nested body changed region numbering unit")
    );
    assert_eq!(
        regions.nest(key(0), key(9)),
        Err("nested body already numbered its own regions")
    );
    Ok(())
}

#[test]
fn nonempty_units_cannot_merge_or_change_original_owner() -> Result<(), &'static str> {
    let mut regions = BodyRegions::default();
    let left = regions.allocate(annotation(key(0)))?;
    let right = regions.allocate(annotation(key(1)))?;
    assert_eq!(left, right);
    assert_eq!(
        regions.nest(key(1), key(0)),
        Err("nested body already numbered its own regions")
    );
    assert_eq!(
        regions.get(key(0), left).map(|fact| fact.body),
        Some(key(0))
    );
    assert_eq!(
        regions.get(key(1), right).map(|fact| fact.body),
        Some(key(1))
    );
    assert!(regions.get(key(2), left).is_none());
    regions.nest(key(2), key(1))?;
    assert_eq!(
        regions.get(key(2), right).map(|fact| fact.body),
        Some(key(1))
    );
    assert_eq!(regions.unit_of(key(0)), key(0));
    assert_eq!(regions.unit_of(key(1)), key(1));
    Ok(())
}

#[test]
fn activated_empty_root_joins_the_existing_parent_cache() -> Result<(), &'static str> {
    let mut regions = BodyRegions::default();
    let original = regions.allocate(annotation(key(0)))?;
    regions.activate(key(1));
    assert_eq!(regions.frontier(key(1))?, 1);
    regions.nest(key(1), key(0))?;
    assert_eq!(regions.frontier(key(1))?, original.0 + 1);
    assert_eq!(
        regions.get(key(1), original).map(|fact| fact.body),
        Some(key(0))
    );
    let joined = regions.allocate(annotation(key(1)))?;
    assert_eq!(joined.0, original.0 + 1);
    assert_eq!(
        regions.get(key(1), joined).map(|fact| fact.body),
        Some(key(1))
    );
    assert_eq!(
        regions.get(key(0), original).map(|fact| fact.body),
        Some(key(0))
    );
    assert_eq!(
        regions.nest(key(0), key(2)),
        Err("nested body already numbered its own regions")
    );
    assert_eq!(regions.frontier(key(0))?, joined.0 + 1);
    Ok(())
}
