//! Opening one captured clause shares independent closed partitions and their indices.
use super::{Abilities, Environment};
use crate::types::{
    GenericArgs, ItemId, Kind, RegionId, Store, TypeId,
    arguments::Argument,
    copy::Properties,
    parameters::{
        Parameter, ParameterKind,
        clauses::{Clause, Quantified},
        predicates::{Bound, Predicate},
    },
    projection, substitute, traits,
};
use std::sync::Arc;

fn quantified_environment(store: &mut Store, width: u32) -> Result<Environment, &'static str> {
    let mut parameters = Vec::new();
    let mut clauses = Vec::new();
    for index in 0..width {
        let parameter = store.intern_parameter(Parameter {
            owner: ItemId(0),
            index,
            kind: ParameterKind::Type { sized: true },
        })?;
        parameters.push((parameter, Abilities::default()));
        let subject = store.intern(Kind::Parameter(parameter))?;
        let owner = ItemId(index * 2 + 1);
        let member = ItemId(index * 2 + 2);
        store.declare_projection(
            member,
            projection::Schema {
                trait_item: owner,
                sized: true,
                copies: Properties::default(),
            },
        )?;
        let region = RegionId::bound(0, 0)?;
        let arguments = store.intern_arguments(&[Argument::Lifetime(region)])?;
        let trait_application = store.intern_trait_application(traits::Application {
            item: owner,
            arguments,
        })?;
        let projected = store.intern(Kind::Projection(projection::Application {
            item: member,
            subject,
            trait_application,
            arguments: GenericArgs::EMPTY,
        }))?;
        let reference = store.intern(Kind::Reference {
            referent: subject,
            mutable: false,
            region,
        })?;
        for predicate in [
            Predicate {
                ty: subject,
                bound: Bound::Trait(trait_application),
            },
            Predicate {
                ty: projected,
                bound: Bound::ProjectionEqual(reference),
            },
            Predicate {
                ty: reference,
                bound: Bound::Copy,
            },
        ] {
            clauses.push(Clause::ForAll(store.intern_quantified(Quantified {
                bound_regions: 1,
                predicate,
            })?));
        }
        clauses.push(Clause::Predicate(Predicate {
            ty: subject,
            bound: Bound::Sized,
        }));
    }
    Environment::new(store, &parameters)?.with_clauses(store, &clauses)
}

fn shared<T>(before: Option<&Arc<T>>, after: Option<&Arc<T>>) -> Result<(), &'static str> {
    assert!(Arc::ptr_eq(
        before.ok_or("missing original partition")?,
        after.ok_or("missing mapped partition")?
    ));
    Ok(())
}

#[test]
fn captured_region_mapping_shares_closed_partitions_and_bounds_work_independently_of_width()
-> Result<(), Box<dyn std::error::Error>> {
    let mut mapping_work = None;
    for width in [16, 64, 256, 1024] {
        let mut store = Store::default();
        let environment = quantified_environment(&mut store, width)?;
        assert!(!environment.requires_region_mapping());
        let (_, construction) = environment.trait_index(&store)?;
        assert_eq!(construction, width as usize + 1);
        let aliases =
            crate::types::relations::RegionEqualities::default().with_aliases(&[RegionId(7)])?;
        let environment = environment.with_region_equalities(aliases);
        assert!(environment.requires_region_mapping());
        for first in [100, 200, 300] {
            let mut fold = substitute::Bound::open_regions(first)?;
            let mapped = environment.substitute_regions(&mut store, &mut fold)?;
            assert!(Arc::ptr_eq(&environment.parameters, &mapped.parameters));
            shared(
                environment.assumptions.as_ref(),
                mapped.assumptions.as_ref(),
            )?;
            shared(environment.traits.as_ref(), mapped.traits.as_ref())?;
            shared(environment.builtins.as_ref(), mapped.builtins.as_ref())?;
            shared(environment.equalities.as_ref(), mapped.equalities.as_ref())?;
            let (a, _) = environment.trait_index(&store)?;
            let (b, work) = mapped.trait_index(&store)?;
            shared(a.as_ref(), b.as_ref())?;
            assert_eq!(work, 1);
            assert_eq!(*mapping_work.get_or_insert(fold.steps), fold.steps);
            assert_eq!(fold.opened_regions().count(), 2);
            let original = RegionId::bound(0, 0)?;
            assert!(
                fold.opened_regions()
                    .any(|pair| pair == (original, RegionId(first)))
            );
            assert_eq!(fold.next_region(), Some(first + 1));
            assert_eq!(mapped.projection_clauses().count(), width as usize);
        }
    }
    Ok(())
}

#[test]
fn parameter_replacement_still_visits_closed_partitions_and_updates_region_summaries()
-> Result<(), &'static str> {
    let mut store = Store::default();
    let environment = quantified_environment(&mut store, 1)?;
    let parameter = *environment
        .parameters
        .keys()
        .next()
        .ok_or("missing parameter")?;
    let reference = store.intern(Kind::Reference {
        referent: TypeId::UNIT,
        mutable: false,
        region: RegionId::bound(0, 2)?,
    })?;
    let mut fold =
        substitute::Bound::parameters(&store, &[(parameter, Argument::Type(reference))])?;
    let mapped = environment.substitute_regions(&mut store, &mut fold)?;
    assert!(mapped.requires_region_mapping());
    assert!(!environment.requires_region_mapping());
    assert!(mapped.assumes_sized(reference));
    for clause in mapped.trait_clauses()?.chain(mapped.projection_clauses()) {
        assert!(clause.has_free_regions(&store)?);
    }
    let original = environment
        .traits
        .as_ref()
        .ok_or("missing original traits")?;
    let mapped = mapped.traits.as_ref().ok_or("missing mapped traits")?;
    assert!(!Arc::ptr_eq(original, mapped));
    Ok(())
}
