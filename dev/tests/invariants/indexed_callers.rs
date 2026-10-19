//! Cached candidate structure belongs to its exact immutable caller clauses.
use super::{
    Assumptions, BYTE, Clause, Context, ItemId, Kind, Parameter, ParameterKind, RegionId, Store,
    TestResult, clause, predicate,
};
use crate::types::{
    arguments::Argument,
    parameters::environment::{Abilities, Environment},
    substitute,
};
use std::sync::Arc;

#[test]
fn shared_indices_preserve_distinct_callers_and_new_quantified_alternatives() -> TestResult {
    let mut store = Store::default();
    let ordinary = predicate(&mut store, ItemId(1), BYTE, &[RegionId::STATIC])?;
    let body = predicate(&mut store, ItemId(1), BYTE, &[RegionId::bound(0, 0)?])?;
    let universal = clause(&mut store, 1, body)?;
    let environment = Environment::new(&store, &[])?.with_predicates(&store, &[ordinary])?;
    let (initial, work) = Assumptions::from_environment(&store, Some(&environment))?;
    assert_eq!(work, 2);
    let (shared, work) = Assumptions::from_environment(&store, Some(&environment))?;
    assert_eq!(work, 1);
    assert!(Arc::ptr_eq(
        initial.index.as_ref().ok_or("missing initial index")?,
        shared.index.as_ref().ok_or("missing shared index")?,
    ));
    let duplicate = environment.clone().with_predicates(&store, &[ordinary])?;
    let (duplicate, work) = Assumptions::from_environment(&store, Some(&duplicate))?;
    assert_eq!(work, 1);
    assert!(Arc::ptr_eq(
        initial.index.as_ref().ok_or("missing initial index")?,
        duplicate.index.as_ref().ok_or("missing duplicate index")?,
    ));
    let extended = environment.clone().with_clauses(&store, &[universal])?;
    let (extended, work) = Assumptions::from_environment(&store, Some(&extended))?;
    assert_eq!(work, 3);
    assert!(!Arc::ptr_eq(
        initial.index.as_ref().ok_or("missing initial index")?,
        extended.index.as_ref().ok_or("missing extended index")?,
    ));
    assert!(initial.exact(&store, ordinary)?.is_some());
    assert!(extended.exact(&store, ordinary)?.is_none());
    assert!(!initial.has_quantified());
    assert!(extended.has_quantified());
    let mut context = Context::default();
    assert!(!extended.unambiguous_scoped_declaration(
        &mut store,
        &mut context,
        ordinary.into(),
        ordinary,
        20,
    )?);
    assert!(context.region_equalities.is_empty());
    assert_eq!(
        environment.trait_clauses().collect::<Vec<_>>(),
        [ordinary.into()]
    );
    Ok(())
}

#[test]
fn additional_ordinary_assumptions_invalidate_only_the_mutated_parameter_index() -> TestResult {
    let mut store = Store::default();
    let parameter = store.intern_parameter(Parameter {
        owner: ItemId(9),
        index: 0,
        kind: ParameterKind::Type { sized: true },
    })?;
    let subject = store.intern(Kind::Parameter(parameter))?;
    let original = predicate(&mut store, ItemId(1), subject, &[])?;
    let added = predicate(&mut store, ItemId(2), subject, &[])?;
    let environment = Environment::new(&store, &[(parameter, Abilities::default())])?
        .with_predicates(&store, &[original])?;
    let (initial, _) = Assumptions::from_environment(&store, Some(&environment))?;
    assert_eq!(initial.parameter_traits(parameter), [ItemId(1)]);
    let extended = environment.clone().with_predicates(&store, &[added])?;
    let (extended, work) = Assumptions::from_environment(&store, Some(&extended))?;
    assert_eq!(work, 3);
    assert_eq!(extended.parameter_traits(parameter).len(), 2);
    assert!(extended.exact(&store, added)?.is_some());
    assert!(initial.exact(&store, added)?.is_none());
    let (mut local, work) = Assumptions::from_environment(&store, Some(&environment))?;
    assert_eq!(work, 1);
    local.insert(&store, added)?;
    assert_eq!(local.parameter_traits(parameter).len(), 2);
    assert_eq!(initial.parameter_traits(parameter), [ItemId(1)]);
    assert!(local.exact(&store, added)?.is_some());
    let (unchanged, work) = Assumptions::from_environment(&store, Some(&environment))?;
    assert_eq!(work, 1);
    assert!(unchanged.exact(&store, added)?.is_none());
    let empty = Environment::new(&store, &[])?;
    let (empty, work) = Assumptions::from_environment(&store, Some(&empty))?;
    assert_eq!(work, 0);
    assert!(empty.index.is_none());
    Ok(())
}

#[test]
fn substitution_rebuilds_subject_keys_and_keeps_local_and_captured_lifetimes() -> TestResult {
    let mut store = Store::default();
    let parameter = store.intern_parameter(Parameter {
        owner: ItemId(9),
        index: 0,
        kind: ParameterKind::Type { sized: true },
    })?;
    let subject = store.intern(Kind::Parameter(parameter))?;
    let ordinary = predicate(&mut store, ItemId(1), subject, &[RegionId::bound(0, 0)?])?;
    let local = predicate(
        &mut store,
        ItemId(2),
        subject,
        &[RegionId::bound(0, 0)?, RegionId::bound(1, 0)?],
    )?;
    let universal = clause(&mut store, 1, local)?;
    let clauses = [universal, Clause::Predicate(ordinary)];
    let environment = Environment::new(&store, &[(parameter, Abilities::default())])?
        .with_clauses(&store, &clauses)?;
    let (initial, work) = Assumptions::from_environment(&store, Some(&environment))?;
    assert_eq!(work, 3);
    assert_eq!(initial.parameter_traits(parameter).len(), 2);
    let mut fold = substitute::Bound::parameters_captured(
        &store,
        &[(parameter, Argument::Type(BYTE))],
        vec![RegionId(7)],
    )?;
    let mapped = environment.substitute_regions(&mut store, &mut fold)?;
    let expected = predicate(&mut store, ItemId(1), BYTE, &[RegionId(7)])?;
    let body = predicate(
        &mut store,
        ItemId(2),
        BYTE,
        &[RegionId::bound(0, 0)?, RegionId(7)],
    )?;
    let quantified = clause(&mut store, 1, body)?;
    assert_eq!(
        mapped.trait_clauses().collect::<Vec<_>>(),
        [quantified, expected.into()]
    );
    let (mapped, work) = Assumptions::from_environment(&store, Some(&mapped))?;
    assert_eq!(work, 3);
    assert!(mapped.parameter_traits(parameter).is_empty());
    assert!(mapped.exact(&store, expected)?.is_some());
    assert!(mapped.exact(&store, ordinary)?.is_none());
    assert!(initial.exact(&store, expected)?.is_none());
    assert!(initial.exact(&store, ordinary)?.is_some());
    let Clause::ForAll(id) = quantified else {
        return Err("missing quantifier".into());
    };
    assert!(mapped.contains_quantified(id));
    assert!(!initial.contains_quantified(id));
    assert_eq!(environment.trait_clauses().collect::<Vec<_>>(), clauses);
    assert_eq!(
        Assumptions::from_environment(&store, Some(&environment))?.1,
        1
    );
    Ok(())
}

#[test]
fn mapped_duplicates_keep_one_premise_without_reusing_another_callers_keys() -> TestResult {
    use crate::sema::traits::assumptions::Selection;
    use crate::types::infer::Instantiation;
    for quantified in [false, true] {
        let mut store = Store::default();
        let region = if quantified {
            RegionId::bound(0, 0)?
        } else {
            RegionId(7)
        };
        let mut parameters = Vec::new();
        let mut replacements = Vec::new();
        let mut clauses = Vec::new();
        for index in 0..2 {
            let parameter = store.intern_parameter(Parameter {
                owner: ItemId(9),
                index,
                kind: ParameterKind::Type { sized: true },
            })?;
            let subject = store.intern(Kind::Parameter(parameter))?;
            let value = predicate(&mut store, ItemId(1), subject, &[region])?;
            clauses.push(if quantified {
                clause(&mut store, 1, value)?
            } else {
                value.into()
            });
            parameters.push((parameter, Abilities::default()));
            replacements.push((parameter, Argument::Type(BYTE)));
        }
        let environment = Environment::new(&store, &parameters)?.with_clauses(&store, &clauses)?;
        let (initial, _) = Assumptions::from_environment(&store, Some(&environment))?;
        let mut fold = substitute::Bound::parameters(&store, &replacements)?;
        let mapped = environment.substitute_regions(&mut store, &mut fold)?;
        let (mapped, _) = Assumptions::from_environment(&store, Some(&mapped))?;
        let goal = predicate(&mut store, ItemId(1), BYTE, &[RegionId(7)])?;
        let mut context = Context::default();
        let mut arguments = Instantiation::new(&mut context, &store, &[])?;
        let goal = context.instantiate_predicate(&mut store, goal, &mut arguments)?;
        assert!(matches!(
            initial.select_scoped(&mut store, &mut context, &goal, 20)?,
            Selection::Absent
        ));
        let Selection::Proven(evidence) =
            mapped.select_scoped(&mut store, &mut context, &goal, 20)?
        else {
            return Err("mapped duplicate assumptions lost their unique evidence".into());
        };
        assert_eq!(evidence.regions, 20..if quantified { 21 } else { 20 });
        assert_eq!(context.region_equalities.len(), usize::from(quantified));
        if quantified {
            assert_eq!(context.region_equalities, [(RegionId(7), RegionId(20))]);
        }
        assert_eq!(environment.trait_clauses().collect::<Vec<_>>(), clauses);
    }
    Ok(())
}
