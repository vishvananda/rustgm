//! Negative Unsize answers require completed source and target formation too.
use super::super::tests as helpers;
use super::*;
use crate::{
    sema::traits::{assumptions::Assumptions, implementations::Availability},
    types::{
        GenericArgs, Kind, Primitive, RegionId, TypeId, arguments::Argument as Typed,
        infer::Domain, parameters::predicates::Predicate,
    },
};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
fn fixture() -> Result<(crate::resolve::expansion::Report, Declarations)> {
    let source =
        crate::test_source::fixture("pa31/tests/spec/unsizing-roles/formation-declarations.rs")?;
    let report = crate::test_source::source_provider(&source, "coercion_formation")?;
    let facts = crate::sema::declarations::analyze(&report)?;
    Ok((report, facts))
}
fn application(facts: &mut Declarations, source: TypeId, target: TypeId) -> Result<Predicate> {
    let role = facts
        .source_bindings
        .as_deref()
        .ok_or("catalog")?
        .validated_language(LangItem::Unsize)?
        .ok_or("Unsize")?;
    let args = facts.types.intern_arguments(&[Typed::Type(target)])?;
    Ok(helpers::predicate(facts, role.definition, source, args)?)
}
fn nominal(
    report: &crate::resolve::expansion::Report,
    facts: &mut Declarations,
    name: &str,
    args: &[Typed],
) -> Result<TypeId> {
    let args = if args.is_empty() {
        GenericArgs::EMPTY
    } else {
        facts.types.intern_arguments(args)?
    };
    Ok(facts.types.intern(Kind::Adt(
        crate::types::ItemId(helpers::definition(report, name)?.0),
        args,
    ))?)
}
#[test]
fn source_unsize_shape_absence_does_not_skip_nominal_or_region_inputs() -> Result {
    for warm in [false, true] {
        for target_side in [false, true] {
            let (report, mut facts) = fixture()?;
            let input = nominal(
                &report,
                &mut facts,
                "Borrowed",
                &[Typed::Lifetime(RegionId(1)), Typed::Lifetime(RegionId(2))],
            )?;
            let primitive = TypeId::primitive(Primitive::U8);
            let predicate = application(
                &mut facts,
                if target_side { primitive } else { input },
                if target_side { input } else { primitive },
            )?;
            let assumptions = Assumptions::default();
            let mut caller = helpers::caller(&assumptions, None);
            caller.first_region = 3;
            let mut search = Search::new(caller, Availability::Validated);
            if warm {
                let role = facts
                    .source_bindings
                    .as_deref()
                    .ok_or("catalog")?
                    .language(LangItem::PointeeSized)
                    .ok_or("pointee")?;
                let wf =
                    helpers::predicate(&mut facts, role.definition, input, GenericArgs::EMPTY)?;
                let (mut context, goal) = helpers::instantiate(&mut facts, wf)?;
                let frame = Frame::new(&facts, &mut context, &goal, 3)?;
                let Decision::Proven(root) = search.run(&mut facts, frame)? else {
                    return Err("formation prefix".into());
                };
                // Canonical PointeeSized formation is trait-only. The Unsize
                // consumer must still demand the separate nominal lifetime facts.
                assert!(search.query.nodes[root].debt_free);
                search.query.resume_sibling_for_test()?;
            }
            let (mut context, goal) = helpers::instantiate(&mut facts, predicate)?;
            let frame = Frame::new(&facts, &mut context, &goal, 3)?;
            let result = search.run(&mut facts, frame);
            println!(
                "region input warm={warm} target_side={target_side} result={result:?} nodes={} debt_nodes={}",
                search.query.nodes.len(),
                search.query.nodes.iter().filter(|n| !n.debt_free).count()
            );
            let error = result
                .err()
                .ok_or("unpaid input became a completed answer")?;
            assert!(error.contains("formation"), "{error}");
            assert!(!matches!(
                search.query.completed.get(&predicate),
                Some(Decision::Absent)
            ));
        }
    }
    Ok(())
}
#[test]
fn source_unsize_distinguishes_valid_mismatch_from_malformed_nominal_inputs() -> Result {
    let (report, mut facts) = fixture()?;
    let plain = nominal(&report, &mut facts, "Plain", &[])?;
    let missing = nominal(&report, &mut facts, "Missing", &[])?;
    let invalid = nominal(&report, &mut facts, "Checked", &[Typed::Type(missing)])?;
    let scalar = TypeId::primitive(Primitive::U8);
    for (source, target, valid) in [
        (plain, scalar, true),
        (scalar, plain, true),
        (invalid, scalar, false),
        (scalar, invalid, false),
    ] {
        let predicate = application(&mut facts, source, target)?;
        let (mut context, goal) = helpers::instantiate(&mut facts, predicate)?;
        let assumptions = Assumptions::default();
        let mut search = Search::new(helpers::caller(&assumptions, None), Availability::Validated);
        let frame = Frame::new(&facts, &mut context, &goal, 1)?;
        let result = search.run(&mut facts, frame);
        if valid {
            assert!(matches!(result?, Decision::Absent));
        } else {
            assert!(result.is_err());
        }
    }
    Ok(())
}
#[test]
fn source_unsize_deferred_heads_validate_the_known_other_input_without_caching_absence() -> Result {
    let (report, mut facts) = fixture()?;
    let missing = nominal(&report, &mut facts, "Missing", &[])?;
    let invalid = nominal(&report, &mut facts, "Checked", &[Typed::Type(missing)])?;
    let role = facts
        .source_bindings
        .as_deref()
        .ok_or("catalog")?
        .language(LangItem::Unsize)
        .ok_or("Unsize")?;
    for (target, valid) in [(TypeId::primitive(Primitive::U8), true), (invalid, false)] {
        let mut context = crate::types::infer::Context::default();
        let unknown = context.variable(Domain::Any)?;
        let target = context.known(&facts.types, target)?;
        let goal = Goal::trait_application(
            unknown,
            crate::types::ItemId(role.definition.0),
            vec![Argument::Type(target)],
        );
        let assumptions = Assumptions::default();
        let mut search = Search::new(helpers::caller(&assumptions, None), Availability::Validated);
        let frame = Frame::new(&facts, &mut context, &goal, 1)?;
        let result = search.run(&mut facts, frame);
        println!("deferred known_input_valid={valid} result={result:?}");
        if valid {
            assert!(matches!(result?, Decision::Deferred));
        } else {
            assert!(result.is_err());
        }
        assert!(!search.query.completed.iter().any(|(predicate, decision)| {
            *decision == Decision::Absent
                && predicate
                    .bound
                    .trait_parts()
                    .and_then(|(app, _)| facts.types.trait_application(app))
                    .is_some_and(|app| app.item.0 == role.definition.0)
        }));
    }
    Ok(())
}

fn projection(
    report: &crate::resolve::expansion::Report,
    facts: &mut Declarations,
) -> Result<TypeId> {
    let subject = nominal(
        report,
        facts,
        "Projected",
        &[Typed::Lifetime(RegionId(1)), Typed::Lifetime(RegionId(2))],
    )?;
    let trait_application =
        facts
            .types
            .intern_trait_application(crate::types::traits::Application {
                item: crate::types::ItemId(helpers::definition(report, "Project")?.0),
                arguments: GenericArgs::EMPTY,
            })?;
    Ok(facts
        .types
        .intern(Kind::Projection(crate::types::projection::Application {
            item: crate::types::ItemId(helpers::definition(report, "Out")?.0),
            subject,
            trait_application,
            arguments: GenericArgs::EMPTY,
        }))?)
}
#[test]
fn completed_unsizing_absence_does_not_erase_a_new_projection_prefix() -> Result {
    for warm in [false, true] {
        let (report, mut facts) = fixture()?;
        let projected = projection(&report, &mut facts)?;
        let scalar = TypeId::primitive(Primitive::U8);
        let assumptions = Assumptions::default();
        let mut caller = helpers::caller(&assumptions, None);
        caller.first_region = 3;
        let mut search = Search::new(caller, Availability::Validated);
        if !warm {
            let predicate = application(&mut facts, projected, scalar)?;
            let (mut context, goal) = helpers::instantiate(&mut facts, predicate)?;
            let frame = Frame::new(&facts, &mut context, &goal, 3)?;
            let error = search
                .run(&mut facts, frame)
                .err()
                .ok_or("cold projection prefix vanished")?;
            assert!(
                error.contains("formation") || error.contains("normalization"),
                "{error}"
            );
            continue;
        }
        let plain = nominal(&report, &mut facts, "Plain", &[])?;
        let predicate = application(&mut facts, plain, scalar)?;
        let (mut context, goal) = helpers::instantiate(&mut facts, predicate)?;
        let frame = Frame::new(&facts, &mut context, &goal, 3)?;
        assert!(matches!(search.run(&mut facts, frame)?, Decision::Absent));
        let mut cached = Frame::new(&facts, &mut context, &goal, 3)?;
        cached.canonical = Some(predicate);
        search.remember(&cached, Decision::Absent);
        assert!(matches!(
            search.query.completed.get(&predicate),
            Some(Decision::Absent)
        ));
        search.query.resume_sibling_for_test()?;
        let pointee = facts
            .source_bindings
            .as_deref()
            .ok_or("catalog")?
            .language(LangItem::PointeeSized)
            .ok_or("pointee")?;
        let wf = helpers::predicate(
            &mut facts,
            pointee.definition,
            projected,
            GenericArgs::EMPTY,
        )?;
        let (mut context, goal) = helpers::instantiate(&mut facts, wf)?;
        let frame = Frame::new(&facts, &mut context, &goal, 3)?;
        let Decision::Proven(root) = search.run(&mut facts, frame)? else {
            return Err("projection prefix".into());
        };
        let prefix = *search.query.nodes[root]
            .children
            .first()
            .ok_or("retained normalization prefix")?;
        assert!(!search.query.nodes[prefix].debt_free);
        search.query.resume_sibling_for_test()?;
        let (mut context, goal) = helpers::instantiate(&mut facts, predicate)?;
        let mut frame = Frame::new(&facts, &mut context, &goal, 3)?;
        frame.normalized_inputs = Some(super::super::frames::NormalizedInputs {
            proof: Some(prefix),
        });
        let error = search
            .run(&mut facts, frame)
            .err()
            .ok_or("warm absence erased its new prefix")?;
        assert!(error.contains("formation"), "{error}");
    }
    Ok(())
}

#[test]
fn deferred_unsizing_keeps_known_lifetime_debts_without_trait_or_absence_export() -> Result {
    let (report, mut facts) = fixture()?;
    let target = nominal(
        &report,
        &mut facts,
        "Borrowed",
        &[Typed::Lifetime(RegionId(1)), Typed::Lifetime(RegionId(2))],
    )?;
    let role = facts
        .source_bindings
        .as_deref()
        .ok_or("catalog")?
        .language(LangItem::Unsize)
        .ok_or("Unsize")?;
    let mut context = crate::types::infer::Context::default();
    let unknown = context.variable(Domain::Any)?;
    let target_term = context.known(&facts.types, target)?;
    let goal = Goal::trait_application(
        unknown,
        crate::types::ItemId(role.definition.0),
        vec![Argument::Type(target_term)],
    );
    let assumptions = Assumptions::default();
    let mut caller = helpers::caller(&assumptions, None);
    caller.first_region = 3;
    let mut search = Search::new(caller, Availability::Validated);
    let frame = Frame::new(&facts, &mut context, &goal, 3)?;
    let decision = search.run(&mut facts, frame)?;
    assert!(matches!(decision, Decision::Deferred));
    let (prefix, node) = search
        .query
        .nodes
        .iter()
        .enumerate()
        .find(|(_, node)| {
            node.source.is_none()
                && node.goal.ty == target
                && node.outlives.contains(&(RegionId(1), RegionId(2)))
        })
        .ok_or("actual known endpoint debt was dropped")?;
    assert!(!node.debt_free);
    assert_eq!(
        search.query.node_scope(prefix),
        crate::types::lifetime_proof::ScopeId::ROOT
    );
    assert!(!search.query.completed.iter().any(|(p, d)| {
        *d == Decision::Absent
            && p.bound
                .trait_parts()
                .and_then(|(a, _)| facts.types.trait_application(a))
                .is_some_and(|a| a.item.0 == role.definition.0)
    }));
    assert!(!search.query.nodes.iter().any(|n| {
        n.source.is_some()
            && n.goal
                .bound
                .trait_parts()
                .and_then(|(a, _)| facts.types.trait_application(a))
                .is_some_and(|a| a.item.0 == role.definition.0)
    }));
    assert!(matches!(
        search.result(&mut facts, &mut context, &goal, decision)?,
        crate::sema::traits::selection::Selection::Deferred
    ));
    Ok(())
}
