//! Exact lookup consumes whole caller facts and keeps competing clauses visible.
use super::{Environment, exact_equality};
use crate::types::{
    GenericArgs, ItemId, Kind, RegionId, Store, TypeId,
    arguments::Argument,
    copy::Properties,
    parameters::{
        clauses::{Clause, Quantified},
        predicates::{Bound, Predicate},
    },
    projection, traits,
};
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

fn equation(
    store: &mut Store,
    index: u32,
    region: RegionId,
) -> TestResult<(projection::Application, Predicate, Predicate)> {
    let member = ItemId(index * 2 + 2);
    let owner = ItemId(index * 2 + 1);
    store.declare_projection(
        member,
        projection::Schema {
            trait_item: owner,
            sized: true,
            copies: Properties::default(),
        },
    )?;
    let arguments = store.intern_arguments(&[Argument::Lifetime(region)])?;
    let trait_application = store.intern_trait_application(traits::Application {
        item: owner,
        arguments,
    })?;
    let application = projection::Application {
        item: member,
        subject: TypeId::UNIT,
        trait_application,
        arguments: GenericArgs::EMPTY,
    };
    let result = store.intern(Kind::Reference {
        referent: TypeId::UNIT,
        mutable: false,
        region,
    })?;
    Ok((
        application,
        Predicate {
            ty: TypeId::UNIT,
            bound: Bound::Trait(trait_application),
        },
        Predicate {
            ty: store.intern(Kind::Projection(application))?,
            bound: Bound::ProjectionEqual(result),
        },
    ))
}

#[test]
fn wide_captured_callers_build_one_index_and_use_constant_work_per_exact_result() -> TestResult {
    for width in [16, 64, 256, 1024] {
        let mut store = Store::default();
        let mut clauses = Vec::new();
        let mut requests = Vec::new();
        for index in 0..width {
            // Every entry uses a distinct captured lifetime as well as a member.
            let slot = u16::try_from(index)?;
            let (application, parent, equality) =
                equation(&mut store, index, RegionId::bound(0, slot)?)?;
            clauses.extend([Clause::Predicate(parent), Clause::Predicate(equality)]);
            requests.push((application, parent, equality.bound));
        }
        let environment = Environment::new(&store, &[])?.with_clauses(&store, &clauses)?;
        assert!(environment.requires_region_mapping());
        let mut work = 0;
        for (index, &(application, parent, expected)) in requests.iter().enumerate() {
            let (actual, result, steps) =
                exact_equality(&store, &environment, application)?.ok_or("missing exact result")?;
            assert_eq!(actual, parent);
            assert_eq!(Bound::ProjectionEqual(result), expected);
            assert_eq!(steps, if index == 0 { width as usize + 1 } else { 1 });
            work += steps;
        }
        assert_eq!(work, width as usize * 2);
        assert_eq!(environment.projection_clauses().count(), width as usize);
    }
    Ok(())
}

#[test]
fn complete_application_identity_and_parent_permission_are_both_required() -> TestResult {
    let mut store = Store::default();
    let (application, parent, equality) = equation(&mut store, 0, RegionId::bound(0, 0)?)?;
    let environment =
        Environment::new(&store, &[])?.with_predicates(&store, &[parent, equality])?;
    assert!(exact_equality(&store, &environment, application)?.is_some());
    for region in [
        RegionId::bound(0, 1)?,
        RegionId::bound(1, 0)?,
        RegionId::STATIC,
        RegionId(7),
    ] {
        let (other, _, _) = equation(&mut store, 0, region)?;
        assert!(exact_equality(&store, &environment, other)?.is_none());
    }
    let own = store.intern_arguments(&[Argument::Type(TypeId::NEVER)])?;
    for other in [
        projection::Application {
            subject: TypeId::NEVER,
            ..application
        },
        projection::Application {
            arguments: own,
            ..application
        },
        equation(&mut store, 1, RegionId::bound(0, 0)?)?.0,
    ] {
        assert!(exact_equality(&store, &environment, other)?.is_none());
    }
    for predicates in [vec![], vec![parent], vec![equality]] {
        let foreign = Environment::new(&store, &[])?.with_predicates(&store, &predicates)?;
        assert!(exact_equality(&store, &foreign, application)?.is_none());
    }
    // Extending a cloned caller cannot mutate the completed original index.
    let (_, other, _) = equation(&mut store, 0, RegionId::bound(0, 1)?)?;
    let competing = environment.clone().with_predicates(&store, &[other])?;
    assert!(exact_equality(&store, &competing, application)?.is_none());
    assert!(exact_equality(&store, &environment, application)?.is_some());
    Ok(())
}

#[test]
fn ordinary_and_quantified_competitors_never_disappear_from_exact_lookup() -> TestResult {
    let mut store = Store::default();
    let (application, parent, equality) = equation(&mut store, 0, RegionId::bound(0, 0)?)?;
    let (_, other_parent, other_equality) = equation(&mut store, 0, RegionId::bound(0, 1)?)?;
    let base = Environment::new(&store, &[])?.with_predicates(&store, &[parent, equality])?;
    let quantified_parent = Clause::ForAll(store.intern_quantified(Quantified {
        bound_regions: 1,
        predicate: parent,
    })?);
    let quantified_equality = Clause::ForAll(store.intern_quantified(Quantified {
        bound_regions: 1,
        predicate: equality,
    })?);
    for competitor in [
        Clause::Predicate(other_parent),
        Clause::Predicate(other_equality),
        Clause::Predicate(Predicate {
            bound: Bound::ProjectionEqual(TypeId::NEVER),
            ..equality
        }),
        quantified_parent,
        quantified_equality,
    ] {
        let environment = base.clone().with_clauses(&store, &[competitor])?;
        assert!(exact_equality(&store, &environment, application)?.is_none());
    }
    assert!(exact_equality(&store, &base, application)?.is_some());
    Ok(())
}
