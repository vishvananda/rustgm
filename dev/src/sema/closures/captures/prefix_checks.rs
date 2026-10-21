//! Prefix semantics, disjoint places, bounded work and nonrecursive deep storage.
use super::{Graph, Mode, Step};
use crate::resolve::model::DefId;

#[test]
fn ancestor_uses_inherit_the_strongest_descendant_mode() -> Result<(), &'static str> {
    for ancestor_first in [false, true] {
        let mut graph = Graph::default();
        let root = graph.root(DefId(7))?;
        let field = graph.project(root, Step::Field(crate::sema::members::Selection::Tuple(0)))?;
        let leaf = graph.project(
            field,
            Step::Field(crate::sema::members::Selection::Tuple(1)),
        )?;
        if ancestor_first {
            graph.require(root, Mode::Shared)?;
        }
        graph.require(leaf, Mode::Value)?;
        graph.require(field, Mode::Mutable)?;
        graph.require(root, Mode::Shared)?;
        let plan = graph.finish()?;
        assert_eq!(
            plan.captures,
            [super::Capture {
                path: root,
                mode: Mode::Value
            }]
        );
    }
    Ok(())
}

#[test]
fn disjoint_places_keep_unique_immutable_permission() -> Result<(), &'static str> {
    let mut graph = Graph::default();
    let root = graph.root(DefId(7))?;
    let first = graph.project(root, Step::Field(crate::sema::members::Selection::Tuple(0)))?;
    let second = graph.project(root, Step::Field(crate::sema::members::Selection::Tuple(1)))?;
    let other = graph.root(DefId(8))?;
    graph.require(first, Mode::Shared)?;
    graph.require(second, Mode::UniqueImmutable)?;
    graph.require(other, Mode::Mutable)?;
    graph.require(first, Mode::Shared)?;
    let plan = graph.finish()?;
    assert_eq!(plan.captures.len(), 3);
    assert_eq!(plan.captures[0].mode, Mode::Shared);
    assert_eq!(plan.captures[1].mode, Mode::UniqueImmutable);
    assert_eq!(plan.captures[2].mode, Mode::Mutable);
    assert_eq!(
        plan.parent(second),
        Some((root, Step::Field(crate::sema::members::Selection::Tuple(1))))
    );
    assert_eq!(plan.root(first), Some(DefId(7)));
    Ok(())
}

#[test]
fn deep_paths_and_repeated_uses_do_not_repeat_ancestor_scans() -> Result<(), &'static str> {
    for depth in [64, 256, 1024, 16_384] {
        let mut graph = Graph::default();
        let root = graph.root(DefId(7))?;
        let mut leaf = root;
        for _ in 0..depth {
            leaf = graph.project(leaf, Step::Field(crate::sema::members::Selection::Tuple(0)))?;
        }
        for _ in 0..depth {
            graph.require(leaf, Mode::UniqueImmutable)?;
        }
        let plan = graph.finish()?;
        assert_eq!(plan.path_count(), depth + 1);
        assert_eq!(plan.captures.len(), 1);
        assert!(plan.work <= depth * 4 + 3);
        assert_eq!(plan.root(leaf), Some(DefId(7)));
        assert_eq!(plan.covering(leaf), plan.captures.first());
        assert!(plan.covering(root).is_none());
    }
    Ok(())
}

#[test]
fn visiting_a_path_is_not_a_capture_and_failed_queries_cannot_publish() -> Result<(), &'static str>
{
    let mut graph = Graph::default();
    let root = graph.root(DefId(7))?;
    graph.project(root, Step::Dereference { mutable: false })?;
    assert!(graph.finish()?.captures.is_empty());
    let mut graph = Graph {
        limit: 2,
        ..Graph::default()
    };
    let root = graph.root(DefId(7))?;
    graph.require(root, Mode::Shared)?;
    assert!(graph.require(root, Mode::Value).is_err());
    assert!(graph.finish().is_err());
    Ok(())
}

#[test]
fn a_covering_reference_borrow_retains_unique_immutable_permission() -> Result<(), &'static str> {
    for ancestor_first in [false, true] {
        let mut graph = Graph::default();
        let root = graph.root(DefId(7))?;
        let reference =
            graph.project(root, Step::Field(crate::sema::members::Selection::Tuple(0)))?;
        let referent = graph.project(reference, Step::Dereference { mutable: true })?;
        let field = graph.project(
            referent,
            Step::Field(crate::sema::members::Selection::Tuple(1)),
        )?;
        if ancestor_first {
            graph.require(root, Mode::Shared)?;
        }
        graph.require(field, Mode::Mutable)?;
        graph.require(root, Mode::Shared)?;
        let plan = graph.finish()?;
        assert_eq!(plan.captures.len(), 1);
        assert_eq!(plan.captures[0].mode, Mode::UniqueImmutable);
        for place in [root, reference, referent, field] {
            assert_eq!(plan.covering(place), plan.captures.first());
        }
    }
    Ok(())
}

#[test]
fn ancestor_mutation_and_ownership_are_stronger_than_referent_uniqueness()
-> Result<(), &'static str> {
    for mode in [Mode::Mutable, Mode::Value] {
        let mut graph = Graph::default();
        let root = graph.root(DefId(7))?;
        let referent = graph.project(root, Step::Dereference { mutable: true })?;
        graph.require(root, mode)?;
        graph.require(referent, Mode::Mutable)?;
        let plan = graph.finish()?;
        assert_eq!(plan.captures.len(), 1);
        assert_eq!(plan.captures[0].mode, mode);
        assert_eq!(plan.covering(referent), plan.captures.first());
    }
    Ok(())
}

#[test]
fn narrower_referent_capture_keeps_its_mutable_permission() -> Result<(), &'static str> {
    let mut graph = Graph::default();
    let root = graph.root(DefId(7))?;
    let referent = graph.project(root, Step::Dereference { mutable: true })?;
    let unused = graph.project(root, Step::Field(crate::sema::members::Selection::Tuple(0)))?;
    graph.require(referent, Mode::Mutable)?;
    let plan = graph.finish()?;
    assert_eq!(plan.captures[0].mode, Mode::Mutable);
    assert!(plan.covering(root).is_none());
    assert!(plan.covering(unused).is_none());
    assert_eq!(plan.covering(referent), plan.captures.first());
    Ok(())
}

#[test]
fn covering_a_deep_use_does_not_copy_or_rescan_its_prefix() -> Result<(), &'static str> {
    for depth in [64, 256, 1024, 16_384] {
        let mut graph = Graph::default();
        let root = graph.root(DefId(7))?;
        graph.require(root, Mode::Shared)?;
        let mut paths = vec![root];
        let mut parent = root;
        for _ in 0..depth {
            parent = graph.project(
                parent,
                Step::Field(crate::sema::members::Selection::Tuple(0)),
            )?;
            paths.push(parent);
        }
        graph.require(parent, Mode::Mutable)?;
        let plan = graph.finish()?;
        assert_eq!(plan.captures.len(), 1);
        assert_eq!(plan.captures[0].mode, Mode::Mutable);
        assert!(plan.work <= depth * 3 + 5);
        for path in paths {
            assert_eq!(plan.covering(path), plan.captures.first());
        }
    }
    Ok(())
}
