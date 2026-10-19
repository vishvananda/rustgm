//! Family equality selection matches complete inputs without exporting failed probes.
use super::{BYTE, TestResult, reference};
use crate::{
    sema::traits::equalities::{Selection, select},
    types::{
        GenericArgs, ItemId, Kind, Primitive, RegionId, Store, TypeId,
        arguments::{Argument, Constant},
        infer::{
            self, Context, Instantiation, arguments::Argument as Inferred, projections::Application,
        },
        parameters::{
            clauses::{Clause, Quantified},
            environment::Environment,
            predicates::{Bound, Predicate},
        },
        projection, traits,
    },
};
const WORD: TypeId = TypeId::primitive(Primitive::U16);
const SIZE: TypeId = TypeId::primitive(Primitive::Usize);

#[test]
fn repeated_family_inputs_share_graph_import_and_relation_work() -> TestResult {
    let mut previous = None;
    for width in [16, 64, 256] {
        let mut store = Store::default();
        let mut context = Context::default();
        let hole = context.variable(infer::Domain::Any)?;
        let mut term = hole;
        let mut canonical = BYTE;
        for _ in 0..width {
            term = context.construct(infer::Constructor::Tuple, &[term, term])?;
            let list = store.intern_list(&[canonical, canonical])?;
            canonical = store.intern(Kind::Tuple(list))?;
        }
        let candidate = application(
            &mut store,
            &[Argument::Type(canonical)],
            &vec![Argument::Type(canonical); width],
        )?;
        let subject = context.known(&store, BYTE)?;
        let root = context.projected(
            &store,
            ItemId(2),
            subject,
            &[Inferred::Type(term)],
            &vec![Inferred::Type(term); width],
        )?;
        let goal = context.projection_application(&store, root)?;
        let before = context.steps;
        let mut probe = Context::default();
        let goal = probe.import_projection(&store, &mut context, &goal)?;
        probe.equate_projection_inputs(&mut store, &goal, candidate)?;
        let work = probe.steps + context.steps - before;
        eprintln!("family_width={width} depth={width} complete_input_work={work}");
        if let Some(previous) = previous {
            assert!(
                work <= previous * 5,
                "complete family input matching rescans shared graphs"
            );
        }
        previous = Some(work);
        assert!(matches!(
            context.resolved(&mut store, hole),
            Err(infer::Error::Unresolved)
        ));
    }
    Ok(())
}

fn constant(store: &mut Store, bits: u128) -> TestResult<Argument> {
    Ok(Argument::Const(
        store.intern_constant(Constant { ty: SIZE, bits })?,
    ))
}
fn application(
    store: &mut Store,
    parent: &[Argument],
    family: &[Argument],
) -> TestResult<projection::Application> {
    store.declare_projection(
        ItemId(2),
        projection::Schema {
            trait_item: ItemId(1),
            sized: false,
            copies: crate::types::copy::Properties::default(),
        },
    )?;
    let arguments = store.intern_arguments(parent)?;
    let trait_application = store.intern_trait_application(traits::Application {
        item: ItemId(1),
        arguments,
    })?;
    Ok(projection::Application {
        item: ItemId(2),
        subject: BYTE,
        trait_application,
        arguments: store.intern_arguments(family)?,
    })
}
fn equation(
    store: &mut Store,
    application: projection::Application,
    ty: TypeId,
) -> TestResult<Predicate> {
    Ok(Predicate {
        ty: store.intern(Kind::Projection(application))?,
        bound: Bound::ProjectionEqual(ty),
    })
}
fn query(
    store: &mut Store,
    context: &mut Context,
    value: projection::Application,
) -> TestResult<Application> {
    let ty = store.intern(Kind::Projection(value))?;
    let term = context.known(store, ty)?;
    Ok(context.projection_application(store, term)?)
}
fn select_clause(
    store: &mut Store,
    context: &mut Context,
    query: &Application,
    clauses: &[Clause],
) -> TestResult<Selection> {
    let environment = Environment::new(store, &[])?.with_clauses(store, clauses)?;
    Ok(select(store, context, query, Some(&environment), &[], 100)?)
}

#[test]
fn each_member_type_and_const_argument_selects_only_its_own_equation() -> TestResult {
    let mut store = Store::default();
    let three = constant(&mut store, 3)?;
    let five = constant(&mut store, 5)?;
    let variants = [
        ([Argument::Type(BYTE), three], BYTE),
        ([Argument::Type(WORD), five], WORD),
    ];
    let mut clauses = Vec::new();
    for (arguments, result) in variants {
        let value = application(&mut store, &[], &arguments)?;
        clauses.push(equation(&mut store, value, result)?.into());
    }
    for (arguments, result) in variants {
        let value = application(&mut store, &[], &arguments)?;
        let mut context = Context::default();
        let goal = query(&mut store, &mut context, value)?;
        let Selection::Proven(evidence) = select_clause(&mut store, &mut context, &goal, &clauses)?
        else {
            return Err("lost complete family equation".into());
        };
        assert_eq!(evidence.ty, result);
    }
    for arguments in [[Argument::Type(BYTE), five], [Argument::Type(WORD), three]] {
        let value = application(&mut store, &[], &arguments)?;
        let mut context = Context::default();
        let goal = query(&mut store, &mut context, value)?;
        assert!(matches!(
            select_clause(&mut store, &mut context, &goal, &clauses)?,
            Selection::Absent
        ));
    }
    Ok(())
}

#[test]
fn shared_parent_and_member_holes_survive_failed_probes_without_splitting() -> TestResult {
    let mut store = Store::default();
    let three = constant(&mut store, 3)?;
    let five = constant(&mut store, 5)?;
    let parent = [Argument::Type(BYTE), three];
    let valid = application(&mut store, &parent, &parent)?;
    let correct = equation(&mut store, valid, WORD)?;
    for family in [[Argument::Type(WORD), three], [Argument::Type(BYTE), five]] {
        let wrong = application(&mut store, &parent, &family)?;
        let wrong = equation(&mut store, wrong, BYTE)?;
        let mut context = Context::default();
        let subject = context.known(&store, BYTE)?;
        let ty = context.variable(infer::Domain::Any)?;
        let count = context.const_variable(&store, SIZE)?;
        let arguments = [Inferred::Type(ty), Inferred::Const(count)];
        let term = context.projected(&store, ItemId(2), subject, &arguments, &arguments)?;
        let goal = context.projection_application(&store, term)?;
        assert!(matches!(
            select_clause(&mut store, &mut context, &goal, &[wrong.into()])?,
            Selection::Absent
        ));
        assert!(matches!(
            context.resolved(&mut store, ty),
            Err(infer::Error::Unresolved)
        ));
        assert!(matches!(
            context.resolved_const(&mut store, count),
            Err(infer::Error::Unresolved)
        ));
        assert!(context.region_equalities.is_empty());
        let Selection::Proven(evidence) = select_clause(
            &mut store,
            &mut context,
            &goal,
            &[wrong.into(), correct.into()],
        )?
        else {
            return Err("shared holes admitted incompatible candidate".into());
        };
        assert_eq!(evidence.ty, WORD);
        assert_eq!(context.resolved(&mut store, ty)?, BYTE);
        let Argument::Const(expected) = three else {
            return Err("missing constant".into());
        };
        assert_eq!(context.resolved_const(&mut store, count)?, expected);
    }
    Ok(())
}

#[test]
fn equal_outputs_of_different_family_inputs_remain_ambiguous() -> TestResult {
    let mut store = Store::default();
    let mut clauses = Vec::new();
    for ty in [BYTE, WORD] {
        let value = application(&mut store, &[], &[Argument::Type(ty)])?;
        clauses.push(equation(&mut store, value, BYTE)?.into());
    }
    let mut context = Context::default();
    let subject = context.known(&store, BYTE)?;
    let hole = context.variable(infer::Domain::Any)?;
    let term = context.projected(&store, ItemId(2), subject, &[], &[Inferred::Type(hole)])?;
    let goal = context.projection_application(&store, term)?;
    assert!(matches!(
        select_clause(&mut store, &mut context, &goal, &clauses)?,
        Selection::Ambiguous
    ));
    assert!(matches!(
        context.resolved(&mut store, hole),
        Err(infer::Error::Unresolved)
    ));
    assert!(context.region_equalities.is_empty());
    Ok(())
}

#[test]
fn a_quantified_family_equality_retains_correlated_inputs_and_fresh_result_scope() -> TestResult {
    let mut store = Store::default();
    let local = RegionId::bound(0, 0)?;
    let value = application(
        &mut store,
        &[Argument::Lifetime(local)],
        &[Argument::Lifetime(local)],
    )?;
    let output = reference(&mut store, local)?;
    let predicate = equation(&mut store, value, output)?;
    let clause = Clause::ForAll(store.intern_quantified(Quantified {
        bound_regions: 1,
        predicate,
    })?);
    for first in [100, 200] {
        let value = application(
            &mut store,
            &[Argument::Lifetime(RegionId(7))],
            &[Argument::Lifetime(RegionId(8))],
        )?;
        let mut context = Context::default();
        let goal = query(&mut store, &mut context, value)?;
        let environment = Environment::new(&store, &[])?.with_clauses(&store, &[clause])?;
        let Selection::Proven(evidence) = select(
            &mut store,
            &mut context,
            &goal,
            Some(&environment),
            &[],
            first,
        )?
        else {
            return Err("lost quantified family equation".into());
        };
        assert_eq!(evidence.premise.regions, first..first + 1);
        assert_eq!(evidence.ty, reference(&mut store, RegionId(first))?);
        for original in [RegionId(7), RegionId(8)] {
            assert!(
                context
                    .region_equalities
                    .iter()
                    .any(|&(a, b)| (a == original && b == RegionId(first))
                        || (b == original && a == RegionId(first)))
            );
        }
    }
    Ok(())
}

#[test]
fn nested_opaque_family_inputs_are_not_equated_through_their_results() -> TestResult {
    let mut store = Store::default();
    let mut inner = application(&mut store, &[], &[])?;
    let first = store.intern(Kind::Projection(inner))?;
    inner.subject = WORD;
    let second = store.intern(Kind::Projection(inner))?;
    let value = application(&mut store, &[], &[Argument::Type(first)])?;
    let predicate = equation(&mut store, value, BYTE)?;
    let value = application(&mut store, &[], &[Argument::Type(second)])?;
    let mut context = Context::default();
    let goal = query(&mut store, &mut context, value)?;
    assert!(matches!(
        select_clause(&mut store, &mut context, &goal, &[predicate.into()])?,
        Selection::Absent
    ));
    assert!(context.region_equalities.is_empty());
    Ok(())
}

#[test]
fn proven_equality_transport_matches_family_inputs_before_committing_result() -> TestResult {
    let mut store = Store::default();
    let value = application(&mut store, &[], &[Argument::Type(BYTE)])?;
    let predicate = equation(&mut store, value, WORD)?;
    let mut context = Context::default();
    let mut substitution = Instantiation::new(&mut context, &store, &[])?;
    let goal = context.instantiate_predicate(&mut store, predicate, &mut substitution)?;
    context.equate_proved_predicate(&mut store, &goal, predicate)?;
    let arguments = store.intern_arguments(&[Argument::Type(WORD)])?;
    let wrong = equation(
        &mut store,
        projection::Application { arguments, ..value },
        WORD,
    )?;
    assert!(matches!(
        context.equate_proved_predicate(&mut store, &goal, wrong),
        Err(infer::Error::Mismatch)
    ));
    assert_ne!(value.arguments, GenericArgs::EMPTY);
    Ok(())
}
