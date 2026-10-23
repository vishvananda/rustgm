//! Negative provenance and bounded completion remain independent checks.
use super::*;
use crate::{
    sema::traits::{assumptions::Assumptions, implementations::Availability},
    types::{
        GenericArgs, ItemId, Kind, Primitive, TypeId, lifetime_proof::ScopeId,
        parameters::predicates::Bound,
    },
};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

fn negative_declarations(
    captured: bool,
) -> Result<(crate::resolve::expansion::Report, Declarations)> {
    let mut source =
        crate::test_source::fixture("pa31/tests/spec/source-freeze/shared-default.rs")?;
    if captured {
        let (header, _) = source.split_once("struct N;").ok_or("provider header")?;
        source = format!("{header}struct N<'a>(&'a u8);impl<'a> !Freeze for N<'a>{{}}");
    }
    let report = crate::test_source::source_provider(&source, "negative_provenance")?;
    let facts = crate::sema::declarations::analyze(&report)?;
    Ok((report, facts))
}

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
    let (report, mut facts) = negative_declarations(false)?;
    let ty = facts.types.intern(Kind::Adt(
        ItemId(tests::definition(&report, "N")?.0),
        GenericArgs::EMPTY,
    ))?;
    let predicate = Predicate {
        ty,
        bound: freezing::bound(&mut facts)?,
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
    let (report, mut facts) = negative_declarations(false)?;
    let ty = facts.types.intern(Kind::Adt(
        ItemId(tests::definition(&report, "N")?.0),
        GenericArgs::EMPTY,
    ))?;
    let predicate = Predicate {
        ty,
        bound: freezing::bound(&mut facts)?,
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

#[test]
fn closed_negative_with_private_capture_keeps_unpaid_graph() -> Result {
    let (report, mut facts) = negative_declarations(true)?;
    let arguments =
        facts
            .types
            .intern_arguments(&[crate::types::arguments::Argument::Lifetime(
                crate::types::RegionId::STATIC,
            )])?;
    let ty = facts.types.intern(Kind::Adt(
        ItemId(tests::definition(&report, "N")?.0),
        arguments,
    ))?;
    let predicate = Predicate {
        ty,
        bound: freezing::bound(&mut facts)?,
    };
    let (mut context, goal) = tests::instantiate(&mut facts, predicate)?;
    let assumptions = Assumptions::default();
    let mut search = Search::new(
        tests::caller(&assumptions, None),
        Availability::Declarations,
    );
    let frame = Frame::new(&facts, &mut context, &goal, 1)?;
    let Decision::Refuted(root) = search.run(&mut facts, frame)? else {
        return Err("captured negative lost its result".into());
    };
    assert!(!search.query.nodes[root].debt_free);
    assert!(search.negative_child(Decision::Refuted(root)).is_err());
    let refutation = search.refutation(&facts, root, predicate)?;
    let super::super::Refutation::Conditional(refutation) = refutation else {
        return Err("captured refutation dropped its debt".into());
    };
    assert!(!refutation.proof.regions.is_empty());
    assert!(!refutation.proof.equalities.is_empty());
    assert!(
        super::super::Refutation::Conditional(refutation)
            .reject()
            .is_err()
    );
    Ok(())
}

#[test]
fn source_nominal_freeze_formation_retains_actual_sizing_obligations() -> Result {
    let source =
        crate::test_source::fixture("pa31/tests/spec/source-freeze/nominal-no-override.rs")?;
    let report = crate::test_source::source_provider(&source, "freeze_formation")?;
    let mut facts = crate::sema::declarations::analyze(&report)?;
    let n = tests::definition(&report, "N")?;
    let n = facts.definitions[n.0 as usize].ok_or("N")?;
    let w = tests::definition(&report, "W")?;
    let arguments = facts
        .types
        .intern_arguments(&[crate::types::arguments::Argument::Type(n)])?;
    let ty = facts.types.intern(Kind::Adt(ItemId(w.0), arguments))?;
    let bound = freezing::bound(&mut facts)?;
    let predicate = Predicate { ty, bound };
    let (mut context, goal) = tests::instantiate(&mut facts, predicate)?;
    let assumptions = Assumptions::default();
    let mut search = Search::new(tests::caller(&assumptions, None), Availability::Validated);
    let frame = Frame::new(&facts, &mut context, &goal, 1)?;
    let Decision::Proven(root) = search.run(&mut facts, frame)? else {
        return Err("nominal source Freeze formation did not complete".into());
    };
    assert_eq!(search.query.nodes[root].goal, predicate);
    assert!(search.query.nodes[root].children.len() >= 2);
    assert!(search.query.nodes.iter().any(|node| {
        node.source.is_none() && node.goal == predicate && !node.children.is_empty()
    }));
    assert!(
        search
            .query
            .nodes
            .iter()
            .any(|node| { matches!(node.goal.bound, Bound::Trait(_)) && node.goal.ty == n })
    );
    assert!(search.query.active.is_empty());
    let evidence = search.evidence(&facts, root)?;
    assert!(matches!(evidence.source, Source::Builtin));
    Ok(())
}
