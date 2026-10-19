//! Equality instantiation retains original clauses, input equations and private IDs.
#[path = "family_equalities.rs"]
mod families;
use super::{Environment, Selection, select};
use crate::types::{
    GenericArgs, ItemId, Kind, Primitive, RegionId, Store, TypeId,
    arguments::Argument,
    infer::{Context, Instantiation, projections::Application},
    lifetime_proof::Obligations,
    parameters::{
        clauses::{Clause, Quantified},
        predicates::{Bound, Predicate},
    },
    projection, substitute, traits,
};
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
const BYTE: TypeId = TypeId::primitive(Primitive::U8);

fn reference(store: &mut Store, region: RegionId) -> Result<TypeId, &'static str> {
    store.intern(Kind::Reference {
        mutable: false,
        region,
        referent: BYTE,
    })
}

fn equality(store: &mut Store, input: RegionId, output: RegionId) -> TestResult<Predicate> {
    store.declare_projection(
        ItemId(2),
        projection::Schema {
            trait_item: ItemId(1),
            sized: false,
            copies: crate::types::copy::Properties::default(),
        },
    )?;
    let arguments = store.intern_arguments(&[Argument::Lifetime(input)])?;
    let trait_application = store.intern_trait_application(traits::Application {
        item: ItemId(1),
        arguments,
    })?;
    let ty = store.intern(Kind::Projection(projection::Application {
        item: ItemId(2),
        subject: BYTE,
        trait_application,
        arguments: GenericArgs::EMPTY,
    }))?;
    Ok(Predicate {
        ty,
        bound: Bound::ProjectionEqual(reference(store, output)?),
    })
}

fn quantify(store: &mut Store, predicate: Predicate) -> TestResult<Clause> {
    Ok(Clause::ForAll(store.intern_quantified(Quantified {
        bound_regions: 1,
        predicate,
    })?))
}

fn query(store: &mut Store, region: RegionId) -> TestResult<(Context, Application)> {
    let predicate = equality(store, region, region)?;
    let Some(Kind::Projection(p)) = store.kind(predicate.ty) else {
        return Err("test projection".into());
    };
    let parent = Predicate {
        ty: p.subject,
        bound: Bound::Trait(p.trait_application),
    };
    let mut context = Context::default();
    let mut parameters = Instantiation::new(&mut context, store, &[])?;
    let predicate = context.instantiate_predicate(store, parent, &mut parameters)?;
    Ok((
        context,
        Application {
            item: p.item,
            predicate,
            arguments: Vec::new(),
        },
    ))
}

#[test]
fn separate_uses_preserve_the_selected_clause_and_own_distinct_intervals() -> TestResult {
    let mut store = Store::default();
    let local = RegionId::bound(0, 0)?;
    let predicate = equality(&mut store, local, local)?;
    let clause = quantify(&mut store, predicate)?;
    let environment = Environment::new(&store, &[])?.with_clauses(&store, &[clause])?;
    for first in [100, 200] {
        let (mut context, application) = query(&mut store, RegionId(70))?;
        let Selection::Proven(evidence) = select(
            &mut store,
            &mut context,
            &application,
            Some(&environment),
            &[],
            first,
        )?
        else {
            return Err("missing equality".into());
        };
        assert_eq!(evidence.premise.clause, clause);
        assert_eq!(evidence.premise.regions, first..first + 1);
        assert_eq!(
            store.kind(evidence.ty),
            Some(Kind::Reference {
                mutable: false,
                region: RegionId(first),
                referent: BYTE,
            })
        );
        assert!(
            context
                .region_equalities
                .iter()
                .any(|&(a, b)| a == RegionId(70) && b == RegionId(first)
                    || b == RegionId(70) && a == RegionId(first))
        );
        let mut proof = Obligations::new(first);
        evidence.append_to(&mut proof)?;
        assert_eq!(proof.regions, first..first + 1);
    }
    Ok(())
}

#[test]
fn competing_results_do_not_commit_equations_or_omit_a_quantified_candidate() -> TestResult {
    let mut store = Store::default();
    let local = RegionId::bound(0, 0)?;
    let predicate = equality(&mut store, local, local)?;
    let clause = quantify(&mut store, predicate)?;
    let ordinary = equality(&mut store, RegionId(70), RegionId::STATIC)?;
    let environment =
        Environment::new(&store, &[])?.with_clauses(&store, &[ordinary.into(), clause])?;
    let (mut context, application) = query(&mut store, RegionId(70))?;
    assert!(matches!(
        select(
            &mut store,
            &mut context,
            &application,
            Some(&environment),
            &[],
            100
        )?,
        Selection::Ambiguous
    ));
    assert!(context.region_equalities.is_empty());
    let empty = Environment::new(&store, &[])?;
    assert!(matches!(
        select(
            &mut store,
            &mut context,
            &application,
            Some(&empty),
            &[],
            100
        )?,
        Selection::Absent
    ));
    Ok(())
}

#[test]
fn mapping_captures_preserves_the_equality_binder_and_mixed_source_order() -> TestResult {
    let mut store = Store::default();
    let local = RegionId::bound(0, 0)?;
    let predicate = equality(&mut store, local, RegionId::bound(1, 0)?)?;
    let clause = quantify(&mut store, predicate)?;
    let ordinary = equality(&mut store, RegionId::STATIC, RegionId::bound(0, 1)?)?;
    let environment =
        Environment::new(&store, &[])?.with_clauses(&store, &[clause, ordinary.into(), clause])?;
    assert_eq!(
        environment.projection_clauses().collect::<Vec<_>>(),
        [clause, ordinary.into()]
    );
    let mut fold = substitute::Bound::open_regions(20)?;
    let mapped = environment.substitute_regions(&mut store, &mut fold)?;
    let mut opened: Vec<_> = fold.opened_regions().collect();
    opened.sort_unstable_by_key(|(original, _)| original.0);
    assert_eq!(
        opened,
        [
            (RegionId::bound(0, 0)?, RegionId(20)),
            (RegionId::bound(0, 1)?, RegionId(21))
        ]
    );
    let mut clauses = mapped.projection_clauses();
    let Some(Clause::ForAll(id)) = clauses.next() else {
        return Err("flattened equality".into());
    };
    let quantified = store.quantified(id).ok_or("lost equality binder")?;
    assert_eq!(quantified.bound_regions, 1);
    let expected = equality(&mut store, local, RegionId(20))?;
    assert_eq!(quantified.predicate, expected);
    let expected = equality(&mut store, RegionId::STATIC, RegionId(21))?;
    assert_eq!(clauses.next(), Some(expected.into()));
    assert!(clauses.next().is_none());
    assert_eq!(
        environment.projection_clauses().collect::<Vec<_>>(),
        [clause, ordinary.into()]
    );
    Ok(())
}
