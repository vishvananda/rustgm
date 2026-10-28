//! Complete batches preserve feature order and fail before publishing unpaid scopes.
use super::*;
use crate::types::parameters::{clauses::Clause, predicates};
use std::cell::Cell;
type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn mixed_clause_batches_pay_feature_growth_and_keep_the_shared_prefix_immutable() -> TestResult {
    let mut types = Store::default();
    let first = types.intern_feature_charged("first", &|_, _| Ok(()))?;
    let second = types.intern_feature_charged("second", &|_, _| Ok(()))?;
    let application = types.intern_trait_application(crate::types::traits::Application {
        item: ItemId(7),
        arguments: GenericArgs::EMPTY,
    })?;
    let clause = Clause::Predicate(predicates::Predicate {
        ty: TypeId::UNIT,
        bound: predicates::Bound::Trait(application),
    });
    let expected = [
        Clause::UnstableFeature(first),
        clause,
        Clause::UnstableFeature(second),
    ];
    let bytes = Cell::new(0_usize);
    let pay = |_, allocation| {
        bytes.set(bytes.get().saturating_add(allocation));
        Ok(())
    };
    let environment =
        Arc::new(Environment::new(&types, &[])?.with_clauses_charged(&types, &expected, &pay)?);
    assert_eq!(environment.trait_clauses()?.collect::<Vec<_>>(), expected);
    assert!(bytes.get() > 0);
    let third = types.intern_feature_charged("third", &|_, _| Ok(()))?;
    let rejected = Cell::new(false);
    assert!(
        environment
            .as_ref()
            .clone()
            .with_clauses_charged(
                &types,
                &[Clause::UnstableFeature(third)],
                &|_, allocation| {
                    if allocation > 0 {
                        rejected.set(true);
                        Err("no new allocation")
                    } else {
                        Ok(())
                    }
                },
            )
            .is_err()
    );
    assert!(rejected.get());
    assert_eq!(environment.trait_clauses()?.collect::<Vec<_>>(), expected);
    assert!(!environment.assumes_feature(&types, third, &|_, _| Ok(()))?);
    Ok(())
}

#[test]
fn feature_only_declarations_require_payment_and_reuse_completed_parent_scopes() -> TestResult {
    let mut types = Store::default();
    let feature = types.intern_feature_charged("gate", &|_, _| Ok(()))?;
    let owner = DefId(1);
    let child = DefId(2);
    let mut generics = Generics::default();
    generics.predicates.insert(
        owner,
        vec![Predicate {
            source: crate::resolve::unit::NodeKey {
                file: crate::resolve::unit::FileId(0),
                node: crate::syntax::ast::NodeId::new(0),
            },
            value: Clause::UnstableFeature(feature),
        }],
    );
    assert!(generics.parameter_environment(&types, owner).is_err());
    assert!(
        generics
            .complete_environment_charged(&mut types, owner, &|_, bytes| {
                if bytes > 0 { Err("storage") } else { Ok(()) }
            })
            .is_err()
    );
    assert!(!generics.environments.contains_key(&owner));
    let bytes = Cell::new(0_usize);
    let pay = |_, allocation| {
        bytes.set(bytes.get().saturating_add(allocation));
        Ok(())
    };
    generics.complete_environment_charged(&mut types, owner, &pay)?;
    let parent = Arc::clone(generics.environments.get(&owner).ok_or("completed scope")?);
    assert!(parent.assumes_feature(&types, feature, &|_, _| Ok(()))?);
    generics.parents.insert(child, owner);
    generics.complete_environment_charged(&mut types, child, &pay)?;
    assert!(Arc::ptr_eq(
        &parent,
        generics.environments.get(&child).ok_or("shared child")?
    ));
    assert!(bytes.get() > 0);
    assert!(
        generics
            .complete_environment_charged(&mut types, owner, &pay)
            .is_err()
    );
    Ok(())
}
