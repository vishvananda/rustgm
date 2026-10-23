//! Negative provenance and bounded completion remain independent checks.
use super::*;
use crate::{
    sema::traits::{assumptions::Assumptions, implementations::Availability},
    types::{
        GenericArgs, ItemId, Kind, Primitive, TypeId, auto_traits::AutoTrait,
        lifetime_proof::ScopeId, parameters::predicates::Bound,
    },
};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

fn node(children: Vec<usize>, first: u32, end: u32) -> Node {
    Node {
        debt_free: false,
        cycle_dependent: false,
        provisional_size: false,
        goal: Predicate {
            ty: TypeId::primitive(Primitive::U8),
            bound: Bound::Sized,
        },
        source: Some(Source::Builtin),
        children,
        first,
        end,
        equalities: Vec::new(),
        outlives: Vec::new(),
        type_outlives: Vec::new(),
        sized: Vec::new(),
    }
}
#[test]
fn completion_visits_each_produced_node_and_immediate_edge_once() {
    for depth in [8, 32, 128] {
        for noise in [0, 128] {
            let assumptions = Assumptions::default();
            let mut search = Search::new(
                tests::caller(&assumptions, None),
                Availability::Declarations,
            );
            for _ in 0..noise {
                search
                    .query
                    .push(node(Vec::new(), 1, 1), ScopeId::ROOT, 1..1);
            }
            let first = search.query.steps;
            let mut parent = None;
            for _ in 0..depth {
                let index = search.query.push(
                    node(parent.into_iter().collect(), 1, 1),
                    ScopeId::ROOT,
                    1..1,
                );
                assert!(search.query.nodes[index].debt_free);
                parent = Some(index);
            }
            assert_eq!(search.query.steps - first, 2 * (2 * depth - 1));
            let index = search.query.push(
                node(parent.into_iter().collect(), 1, 2),
                ScopeId::ROOT,
                1..2,
            );
            assert!(!search.query.nodes[index].debt_free);
            let first = search.query.steps;
            let index = search
                .query
                .push(node(vec![index], 2, 2), ScopeId::ROOT, 2..2);
            assert_eq!(search.query.steps - first, 4);
            assert!(!search.query.nodes[index].debt_free);
            assert!(search.negative_child(Decision::Refuted(index)).is_err());
        }
    }
    assert_eq!(size_of::<Node>(), 160);
    assert_eq!(
        size_of::<Option<frames::Chosen>>(),
        size_of::<Option<usize>>()
    );
}
#[test]
fn negative_owner_cannot_enter_positive_extraction_or_reuse() -> Result {
    let (report, mut facts) = tests::declarations("struct N;impl !Unpin for N{}")?;
    let ty = facts.types.intern(Kind::Adt(
        ItemId(tests::definition(&report, "N")?.0),
        GenericArgs::EMPTY,
    ))?;
    let predicate = Predicate {
        ty,
        bound: Bound::Auto(AutoTrait::Unpin),
    };
    for export in [false, true] {
        let (mut context, goal) = tests::instantiate(&mut facts, predicate)?;
        let assumptions = Assumptions::default();
        let mut search = Search::new(
            tests::caller(&assumptions, None),
            Availability::Declarations,
        );
        if export {
            search.query.reuse = Some(completed_proofs::Active::new(Reuse::default()));
        }
        let frame = Frame::new(&facts, &mut context, &goal, 1)?;
        let Decision::Refuted(root) = search.run(&mut facts, frame)? else {
            return Err("negative impl was not retained as refutation".into());
        };
        assert!(search.query.nodes[root].debt_free);
        if export {
            assert!(
                search
                    .query
                    .export_completed(&facts, Decision::Proven(root))
                    .is_err()
            );
        } else {
            assert!(search.evidence(&facts, root).is_err());
        }
    }
    Ok(())
}
#[test]
fn refutation_keeps_exact_goal_and_caller_frontier() -> Result {
    let (report, mut facts) = tests::declarations("struct N;impl !Unpin for N{}")?;
    let ty = facts.types.intern(Kind::Adt(
        ItemId(tests::definition(&report, "N")?.0),
        GenericArgs::EMPTY,
    ))?;
    let predicate = Predicate {
        ty,
        bound: Bound::Auto(AutoTrait::Unpin),
    };
    let assumptions = Assumptions::default();
    let (mut context, goal) = tests::instantiate(&mut facts, predicate)?;
    let mut search = Search::new(
        tests::caller(&assumptions, None),
        Availability::Declarations,
    );
    let frame = Frame::new(&facts, &mut context, &goal, 1)?;
    let Decision::Refuted(root) = search.run(&mut facts, frame)? else {
        return Err("refutation".into());
    };
    let result = search.refutation(&facts, root, predicate)?;
    result.validate(
        predicate,
        &tests::caller(&assumptions, None),
        Availability::Declarations,
    )?;
    let mut wrong = tests::caller(&assumptions, None);
    wrong.first_region += 1;
    assert!(
        result
            .validate(predicate, &wrong, Availability::Declarations)
            .is_err()
    );
    assert!(
        result
            .validate(
                Predicate {
                    ty: TypeId::UNIT,
                    ..predicate
                },
                &tests::caller(&assumptions, None),
                Availability::Declarations
            )
            .is_err()
    );
    let environment = std::sync::Arc::new(crate::types::parameters::environment::Environment::new(
        &facts.types,
        &[],
    )?);
    let mut wrong = tests::caller(&assumptions, Some(&environment));
    assert!(
        result
            .validate(predicate, &wrong, Availability::Declarations)
            .is_err()
    );
    wrong.first_region += 1;
    assert!(
        result
            .validate(predicate, &wrong, Availability::Validated)
            .is_err()
    );
    result.reject()?;
    Ok(())
}
