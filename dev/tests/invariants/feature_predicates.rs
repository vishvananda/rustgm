//! Non-type requirements have real identities, independent premises and paid queries.
use super::*;
use crate::types::{
    parameters::{
        clauses::{Clause, Quantifier},
        environment::Environment,
    },
    substitute,
};
use std::{cell::Cell, sync::Arc};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
#[derive(Default)]
struct Paid {
    steps: Cell<usize>,
    bytes: Cell<usize>,
}
impl Paid {
    fn pay(&self, steps: usize, bytes: usize) -> std::result::Result<(), &'static str> {
        self.steps.set(self.steps.get().saturating_add(steps));
        self.bytes.set(self.bytes.get().saturating_add(bytes));
        if self.steps.get() > 1_000_000 || self.bytes.get() > 256 * 1024 * 1024 {
            return Err("feature invariant measurement budget exhausted");
        }
        Ok(())
    }
}
fn original_clause_layout() -> usize {
    use crate::types::parameters::{
        clauses::QuantifiedId,
        predicates::{Bound, Predicate},
    };
    // Exact pre-change sum type, sharing the unchanged canonical payload types.
    enum Original {
        Predicate(Predicate),
        ForAll(QuantifiedId),
    }
    for value in [
        Original::Predicate(Predicate {
            ty: crate::types::TypeId::UNIT,
            bound: Bound::Sized,
        }),
        Original::ForAll(QuantifiedId(0)),
    ] {
        match value {
            Original::Predicate(value) => assert_eq!(value.ty, crate::types::TypeId::UNIT),
            Original::ForAll(value) => assert_eq!(value.0, 0),
        }
    }
    size_of::<Original>()
}
#[test]
fn canonical_feature_names_reject_foreign_stores_and_failed_publication() -> Result {
    let paid = Paid::default();
    let mut types = Store::default();
    assert!(
        types
            .intern_feature_charged("not paid", &|_, _| Err("payment"))
            .is_err()
    );
    let feature = types.intern_feature_charged("transmutability", &|w, b| paid.pay(w, b))?;
    assert_eq!(types.feature_name(feature), Some("transmutability"));
    let before = paid.bytes.get();
    assert_eq!(
        types.intern_feature_charged("transmutability", &|w, b| paid.pay(w, b))?,
        feature
    );
    assert_eq!(paid.bytes.get(), before);
    let mut other = Store::default();
    let other_id = other.intern_feature_charged("transmutability", &|w, b| paid.pay(w, b))?;
    assert_ne!(feature, other_id);
    assert!(types.feature_name(other_id).is_none());
    assert!(other.feature_name(feature).is_none());
    for width in [1_usize, 8, 32, 128] {
        let paid = Paid::default();
        let mut sized = Store::default();
        for index in 0..width {
            sized.intern_feature_charged(&format!("feature_{index}"), &|w, b| paid.pay(w, b))?;
        }
        println!(
            "feature names width={width} steps={} bytes={}",
            paid.steps.get(),
            paid.bytes.get()
        );
        assert!(paid.steps.get() <= 512 + width * 600);
        assert!(paid.bytes.get() <= 2048 + width * 4096);
    }
    assert_eq!(size_of::<FeatureId>(), 8);
    assert_eq!(original_clause_layout(), 12);
    assert_eq!(size_of::<Clause>(), original_clause_layout());
    assert_eq!(size_of::<Store>(), 472);
    assert_eq!(size_of::<Environment>(), 64);
    assert_eq!(size_of::<crate::sema::model::NodeFacts>(), 80);
    Ok(())
}
#[test]
fn non_type_clause_laws_preserve_identity_without_type_subjects() -> Result {
    let mut types = Store::default();
    let id = types.intern_feature_charged("gate", &|_, _| Ok(()))?;
    let clause = Clause::UnstableFeature(id);
    assert!(!clause.requires_definition_proof(&types)?);
    assert!(!clause.checks_parameter_default(&types)?);
    assert!(!clause.contains_parameters(&types)?);
    assert!(!clause.has_free_regions(&types)?);
    assert!(!clause.is_outlives(&types)?);
    assert_eq!(clause.caller_outlives(&types)?, None);
    assert_eq!(clause.escaping_binders(&types)?, 0);
    assert!(clause.ordinary().is_err());
    assert_eq!(
        substitute::Bound::new(Vec::new()).clause(&mut types, clause)?,
        clause
    );
    assert_eq!(Quantifier::new(3)?.clause(&mut types, clause)?, clause);
    assert_eq!(
        crate::sema::traits::effects::in_context(
            &mut types,
            clause,
            crate::support::constness::Constness::Const
        )?,
        clause
    );
    Ok(())
}
#[test]
fn feature_only_premises_remain_non_neutral_and_share_warm_indexes() -> Result {
    let paid = Paid::default();
    let pay = |w, b| paid.pay(w, b);
    let mut types = Store::default();
    let id = types.intern_feature_charged("gate", &pay)?;
    let missing = types.intern_feature_charged("other", &pay)?;
    let env = Arc::new(
        Environment::new(&types, &[])?.with_feature_assumptions_charged(
            &types,
            &[id],
            &|_, _| Ok(()),
        )?,
    );
    assert!(!env.is_neutral());
    assert!(!env.has_only_parameters());
    assert!(env.assumes_feature(&types, id, &pay)?);
    assert!(!env.assumes_feature(&types, missing, &pay)?);
    let failed_before = (paid.steps.get(), paid.bytes.get());
    assert!(
        env.trait_index_charged(&types, &|w, b| {
            paid.pay(w, b)?;
            if b == 0 { Ok(()) } else { Err("cold failure") }
        })
        .is_err()
    );
    assert!(paid.steps.get() > failed_before.0 && paid.bytes.get() > failed_before.1);
    let index = env
        .trait_index_charged(&types, &pay)?
        .ok_or("actual feature index")?;
    let mut foreign = Store::default();
    let foreign_id = foreign.intern_feature_charged("gate", &pay)?;
    assert!(index.contains_feature(&foreign, foreign_id, &pay).is_err());
    assert!(env.assumes_feature(&foreign, foreign_id, &pay).is_err());
    let before = paid.bytes.get();
    for _ in 0..16 {
        assert!(index.contains_feature(&types, id, &pay)?);
    }
    assert_eq!(paid.bytes.get(), before);
    let extended = crate::types::parameters::premise_index::ClauseIndex::extended(
        Some(Arc::clone(&index)),
        &types,
        &[],
        &pay,
    )?;
    assert!(extended.contains_feature(&types, id, &pay)?);
    let before_failure = paid.steps.get();
    assert!(
        index
            .contains_feature(&types, id, &|w, b| {
                paid.pay(w, b)?;
                Err("failure")
            })
            .is_err()
    );
    assert!(paid.steps.get() > before_failure);
    Ok(())
}
#[test]
fn immutable_policy_separates_caller_premises_from_explicit_global_enabling() -> Result {
    let mut types = Store::default();
    let id = types.intern_feature_charged("gate", &|_, _| Ok(()))?;
    let other = types.intern_feature_charged("other", &|_, _| Ok(()))?;
    let env = Environment::new(&types, &[])?.with_feature_assumptions_charged(
        &types,
        &[id],
        &|_, _| Ok(()),
    )?;
    let caller_only = Policy::new(&mut types, &[], &|_, _| Ok(()))?;
    assert!(!caller_only.prove(&types, id, None, &|_, _| Ok(()))?);
    assert!(caller_only.prove(&types, id, Some(&env), &|_, _| Ok(()))?);
    assert!(!caller_only.prove(&types, other, Some(&env), &|_, _| Ok(()))?);
    let global = Policy::new(&mut types, &[id], &|_, _| Ok(()))?;
    assert!(global.prove(&types, id, None, &|_, _| Ok(()))?);
    assert!(!global.prove(&types, other, None, &|_, _| Ok(()))?);
    let mut foreign = Store::default();
    let foreign_id = foreign.intern_feature_charged("gate", &|_, _| Ok(()))?;
    assert!(
        global
            .prove(&foreign, foreign_id, None, &|_, _| Ok(()))
            .is_err()
    );
    assert!(Policy::new(&mut types, &[foreign_id], &|_, _| Ok(())).is_err());
    Ok(())
}
#[test]
fn actual_impl_candidates_and_nominal_queries_require_non_type_conditions() -> Result {
    use crate::types::{
        ItemId, Primitive, TypeId,
        parameters::predicates::{Bound, Predicate},
    };
    let report =
        crate::test_source::resolve("trait Base{}trait Target{}impl<T:Base>Target for T{}", true)?;
    let mut facts = crate::sema::declarations::analyze(&report)?;
    let id = facts.types.intern_feature_charged("gate", &|_, _| Ok(()))?;
    let header = facts.generics.implementations().next().ok_or("real impl")?;
    let owner = header.owner;
    let Bound::Trait(application) = header.predicate.bound else {
        return Err("real trait".into());
    };
    let target = facts
        .types
        .trait_application(application)
        .ok_or("real application")?
        .item;
    facts
        .generics
        .predicates
        .get_mut(&owner)
        .ok_or("actual written impl predicate")?[0]
        .value = Clause::UnstableFeature(id);
    assert!(facts.prove_feature(id, None, &|_, _| Ok(())).is_err());
    facts.publish_feature_policy(&[], &|_, _| Ok(()))?;
    assert!(facts.publish_feature_policy(&[id], &|_, _| Ok(())).is_err());
    let application = facts
        .types
        .intern_trait_application(crate::types::traits::Application {
            item: ItemId(target.0),
            arguments: crate::types::GenericArgs::EMPTY,
        })?;
    let requirement = Predicate {
        ty: TypeId::primitive(Primitive::U8),
        bound: Bound::Trait(application),
    };
    let env = Arc::new(
        Environment::new(&facts.types, &[])?.with_feature_assumptions_charged(
            &facts.types,
            &[id],
            &|_, _| Ok(()),
        )?,
    );
    let mut permitted = crate::sema::nominals::traits::State::body(&mut facts, Some(env), 0, 1)?;
    assert!(permitted.prove_clause(&mut facts, Clause::UnstableFeature(id), false)?);
    assert!(permitted.prove(&mut facts, requirement, false)?);
    let mut missing = crate::sema::nominals::traits::State::body(&mut facts, None, 0, 1)?;
    assert!(!missing.prove_clause(&mut facts, Clause::UnstableFeature(id), false)?);
    assert!(!missing.prove(&mut facts, requirement, false)?);
    Ok(())
}
