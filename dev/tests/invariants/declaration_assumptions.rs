//! Declaration ambiguity has no implementation candidates or escaping scratch equations.
use super::{Assumptions, Bound, Clause, Context, ItemId, Predicate, Selection, Store};
use crate::types::{
    GenericArgs, Kind, Primitive, RegionId, TypeId,
    arguments::Argument,
    infer::{Domain, Instantiation},
    parameters::{Parameter, ParameterKind, clauses::Quantified},
    traits::Application,
};

type TestResult = Result<(), Box<dyn std::error::Error>>;
const BYTE: TypeId = TypeId::primitive(Primitive::U8);

fn predicate(
    store: &mut Store,
    item: ItemId,
    ty: TypeId,
    regions: &[RegionId],
) -> Result<Predicate, &'static str> {
    let arguments = store.intern_arguments(
        &regions
            .iter()
            .copied()
            .map(Argument::Lifetime)
            .collect::<Vec<_>>(),
    )?;
    Ok(Predicate {
        ty,
        bound: Bound::Trait(store.intern_trait_application(Application { item, arguments })?),
    })
}

fn clause(store: &mut Store, count: u32, predicate: Predicate) -> Result<Clause, &'static str> {
    Ok(Clause::ForAll(store.intern_quantified(Quantified {
        bound_regions: count,
        predicate,
    })?))
}

#[test]
fn unrelated_quantifiers_leave_an_ordinary_declaration_unambiguous() -> TestResult {
    let mut store = Store::default();
    let ordinary = predicate(&mut store, ItemId(1), BYTE, &[])?;
    let body = predicate(&mut store, ItemId(2), BYTE, &[RegionId::bound(0, 0)?])?;
    let universal = clause(&mut store, 1, body)?;
    let mut assumptions = Assumptions::default();
    assumptions.insert(&store, ordinary)?;
    assumptions.insert_clause(&store, universal)?;
    let mut context = Context::default();
    let unknown = context.variable(Domain::Any)?;
    context.region_equalities.push((RegionId(3), RegionId(4)));
    assert!(assumptions.unambiguous_scoped_declaration(
        &mut store,
        &mut context,
        ordinary.into(),
        ordinary,
        20,
    )?);
    assert_eq!(context.region_equalities, [(RegionId(3), RegionId(4))]);
    assert!(context.head(&store, unknown)?.is_none());
    Ok(())
}

#[test]
fn ordinary_and_universal_alternatives_remain_ambiguous_without_leaking_equations() -> TestResult {
    for reverse in [false, true] {
        let mut store = Store::default();
        let ordinary = predicate(&mut store, ItemId(1), BYTE, &[RegionId::STATIC])?;
        let body = predicate(&mut store, ItemId(1), BYTE, &[RegionId::bound(0, 0)?])?;
        let universal = clause(&mut store, 1, body)?;
        let clauses = if reverse {
            [universal, ordinary.into()]
        } else {
            [ordinary.into(), universal]
        };
        let mut assumptions = Assumptions::default();
        for value in clauses {
            assumptions.insert_clause(&store, value)?;
        }
        let mut context = Context::default();
        assert!(!assumptions.unambiguous_scoped_declaration(
            &mut store,
            &mut context,
            ordinary.into(),
            ordinary,
            20,
        )?);
        assert!(context.region_equalities.is_empty());
    }
    Ok(())
}

#[test]
fn equivalent_unused_binders_preserve_one_actual_premise_and_its_interval() -> TestResult {
    for live in [false, true] {
        for reverse in [false, true] {
            let mut store = Store::default();
            let region = if live {
                RegionId::bound(0, 0)?
            } else {
                RegionId::STATIC
            };
            let body = predicate(&mut store, ItemId(1), BYTE, &[region])?;
            let one = clause(&mut store, 1, body)?;
            let three = clause(&mut store, 3, body)?;
            let clauses = if reverse { [three, one] } else { [one, three] };
            let mut assumptions = Assumptions::default();
            for value in clauses {
                assumptions.insert_clause(&store, value)?;
            }
            let goal = predicate(
                &mut store,
                ItemId(1),
                BYTE,
                &[if live { RegionId(7) } else { RegionId::STATIC }],
            )?;
            let mut context = Context::default();
            let mut substitution = Instantiation::new(&mut context, &store, &[])?;
            let goal = context.instantiate_predicate(&mut store, goal, &mut substitution)?;
            let Selection::Proven(evidence) =
                assumptions.select_scoped(&mut store, &mut context, &goal, 20)?
            else {
                return Err("equivalent unused binders became ambiguous".into());
            };
            assert_eq!(evidence.clause, clauses[0]);
            assert_eq!(evidence.regions, 20..if reverse { 23 } else { 21 });
            if live {
                assert_eq!(context.region_equalities, [(RegionId(7), RegionId(20))]);
            } else {
                assert!(context.region_equalities.is_empty());
            }
        }
    }
    Ok(())
}

#[test]
fn unused_quantifiers_and_an_ordinary_identical_goal_share_declaration_evidence() -> TestResult {
    let mut store = Store::default();
    let value = predicate(&mut store, ItemId(1), BYTE, &[RegionId::STATIC])?;
    let first = clause(&mut store, 1, value)?;
    let second = clause(&mut store, 3, value)?;
    let mut assumptions = Assumptions::default();
    for source in [first, value.into(), second] {
        assumptions.insert_clause(&store, source)?;
    }
    let mut context = Context::default();
    for source in [first, value.into(), second] {
        assert!(assumptions.unambiguous_scoped_declaration(
            &mut store,
            &mut context,
            source,
            value,
            20
        )?);
        assert!(context.region_equalities.is_empty());
    }
    assert!(
        assumptions
            .unambiguous_scoped_declaration(&mut store, &mut context, first, value, 0)
            .is_err()
    );
    let absent = predicate(&mut store, ItemId(2), BYTE, &[])?;
    assert!(
        assumptions
            .unambiguous_scoped_declaration(&mut store, &mut context, absent.into(), absent, 20)
            .is_err()
    );
    Ok(())
}

#[test]
fn correlated_and_independent_premises_are_ambiguous_when_demanded() -> TestResult {
    let mut store = Store::default();
    let correlated = predicate(
        &mut store,
        ItemId(1),
        BYTE,
        &[RegionId::bound(0, 0)?, RegionId::bound(0, 0)?],
    )?;
    let independent = predicate(
        &mut store,
        ItemId(1),
        BYTE,
        &[RegionId::bound(0, 0)?, RegionId::bound(0, 1)?],
    )?;
    let correlated = clause(&mut store, 1, correlated)?;
    let independent = clause(&mut store, 2, independent)?;
    let mut assumptions = Assumptions::default();
    for value in [correlated, independent] {
        assumptions.insert_clause(&store, value)?;
    }
    let goal = predicate(&mut store, ItemId(1), BYTE, &[RegionId(7), RegionId(7)])?;
    let mut context = Context::default();
    let mut substitution = Instantiation::new(&mut context, &store, &[])?;
    let goal = context.instantiate_predicate(&mut store, goal, &mut substitution)?;
    assert!(matches!(
        assumptions.select_scoped(&mut store, &mut context, &goal, 20)?,
        Selection::Deferred
    ));
    assert!(context.region_equalities.is_empty());
    Ok(())
}

#[test]
fn independent_scoped_declarations_do_not_scan_all_other_subjects() -> TestResult {
    for width in [64, 128, 256, 512, 1024] {
        let mut store = Store::default();
        let application = store.intern_trait_application(Application {
            item: ItemId(1),
            arguments: GenericArgs::EMPTY,
        })?;
        let mut assumptions = Assumptions::default();
        let mut queries = Vec::new();
        for index in 0..width {
            let parameter = store.intern_parameter(Parameter {
                owner: ItemId(2),
                index,
                kind: ParameterKind::Type { sized: true },
            })?;
            let value = Predicate {
                ty: store.intern(Kind::Parameter(parameter))?,
                bound: Bound::Trait(application),
            };
            let source = clause(&mut store, 1, value)?;
            assumptions.insert_clause(&store, source)?;
            queries.push((source, value));
        }
        let before = store.len();
        let mut context = Context::default();
        for (source, goal) in queries {
            assert!(assumptions.unambiguous_scoped_declaration(
                &mut store,
                &mut context,
                source,
                goal,
                20
            )?);
        }
        println!(
            "declaration ambiguity width={width} steps={}",
            context.steps
        );
        assert!(context.steps <= 128 * width as usize + 128);
        assert_eq!(store.len(), before);
        assert!(context.region_equalities.is_empty());
    }
    Ok(())
}
