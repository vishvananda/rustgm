//! Query opening shares actual body aliases without identifying independent inputs.
use super::*;
use crate::types::{
    ItemId, Kind, Primitive, Store, TypeId,
    arguments::Argument,
    callable::Convention,
    copy::Properties,
    parameters::predicates::{Bound as TraitBound, Predicate},
    projection,
    relations::RegionEqualities,
    traits::Application,
};
type Result<T = ()> = std::result::Result<T, &'static str>;

fn environment(store: &Store) -> Result<Environment> {
    Environment::new(store, &[])?.with_body_aliases(vec![RegionId(7), RegionId(9)])
}

#[test]
fn body_opening_maps_only_the_actual_slots_and_preserves_nested_binders() -> Result {
    let store = Store::default();
    let environment = environment(&store)?;
    let mut fold = environment.open_body_scope(vec![RegionId(100), RegionId(101)])?;
    for (slot, alias) in [(0, RegionId(7)), (1, RegionId(9))] {
        let opened = RegionId(100 + u32::from(slot));
        assert_eq!(fold.region(RegionId::bound(0, slot)?, 0)?, opened);
        assert_eq!(fold.region(alias, 0)?, opened);
        assert_eq!(fold.region(RegionId::bound(1, slot)?, 1)?, opened);
    }
    let local = RegionId::bound(0, 0)?;
    assert_eq!(fold.region(local, 1)?, local);
    assert_eq!(
        fold.region(RegionId::bound(2, 0)?, 1)?,
        RegionId::bound(1, 0)?
    );
    assert_eq!(fold.region(RegionId(8), 0)?, RegionId(8));
    assert_eq!(fold.region(RegionId::STATIC, 0)?, RegionId::STATIC);
    assert_ne!(fold.region(RegionId(7), 0)?, fold.region(RegionId(9), 0)?);
    Ok(())
}

fn projected(
    store: &mut Store,
    region: RegionId,
    parent: TypeId,
    family: TypeId,
) -> Result<TypeId> {
    store.declare_projection(
        ItemId(12),
        projection::Schema {
            trait_item: ItemId(11),
            sized: false,
            copies: Properties::default(),
        },
    )?;
    let arguments =
        store.intern_arguments(&[Argument::Lifetime(region), Argument::Type(parent)])?;
    let trait_application = store.intern_trait_application(Application {
        item: ItemId(11),
        arguments,
    })?;
    let arguments = store.intern_arguments(&[Argument::Type(family)])?;
    store.intern(Kind::Projection(projection::Application {
        item: ItemId(12),
        subject: TypeId::UNIT,
        trait_application,
        arguments,
    }))
}

#[test]
fn body_opening_keeps_full_trait_and_family_inputs_distinct() -> Result {
    let mut store = Store::default();
    let environment = environment(&store)?;
    let byte = TypeId::primitive(Primitive::U8);
    let boolean = TypeId::primitive(Primitive::Bool);
    let declared = projected(&mut store, RegionId::bound(0, 0)?, byte, byte)?;
    let alias = projected(&mut store, RegionId(7), byte, byte)?;
    let other_lifetime = projected(&mut store, RegionId(9), byte, byte)?;
    let other_parent = projected(&mut store, RegionId(7), boolean, byte)?;
    let other_family = projected(&mut store, RegionId(7), byte, boolean)?;
    let mut fold = environment.open_body_scope(vec![RegionId(100), RegionId(101)])?;
    let expected = fold.apply(&mut store, declared)?;
    assert_eq!(expected, fold.apply(&mut store, alias)?);
    for other in [other_lifetime, other_parent, other_family] {
        assert_ne!(expected, fold.apply(&mut store, other)?);
    }
    let local = store.intern(Kind::Reference {
        mutable: false,
        region: RegionId::bound(0, 0)?,
        referent: byte,
    })?;
    let captured = store.intern(Kind::Reference {
        mutable: false,
        region: RegionId::bound(1, 0)?,
        referent: byte,
    })?;
    let parameters = store.intern_list(&[local, captured])?;
    let callable = store.intern(Kind::Function {
        bound_regions: 1,
        parameters,
        result: byte,
        unsafe_: false,
        convention: Convention::RUST,
    })?;
    let opened = fold.apply(&mut store, callable)?;
    let Some(Kind::Function { parameters, .. }) = store.kind(opened) else {
        return Err("opened function");
    };
    let parameters = store.list(parameters).ok_or("opened parameters")?;
    assert_eq!(parameters[0], local);
    assert!(matches!(
        store.kind(parameters[1]),
        Some(Kind::Reference {
            region: RegionId(100),
            ..
        })
    ));
    Ok(())
}

#[test]
fn semantic_equalities_do_not_mint_alias_metadata_and_proof_views_preserve_it() -> Result {
    let store = Store::default();
    let semantic = RegionEqualities::default()
        .with_aliases(&[RegionId(7)])
        .map_err(|_| "invalid semantic alias fixture")?;
    let ordinary = Environment::new(&store, &[])?.with_region_equalities(semantic);
    assert!(ordinary.body_aliases().is_none());
    let mut fold = ordinary.open_body_scope(vec![RegionId(100)])?;
    assert_eq!(fold.region(RegionId(7), 0)?, RegionId(7));
    let environment = environment(&store)?;
    let original = Arc::clone(environment.body_aliases().ok_or("body aliases")?);
    let proved = environment.with_region_equalities(RegionEqualities::default());
    assert!(Arc::ptr_eq(
        &original,
        proved.body_aliases().ok_or("retained aliases")?
    ));
    let mut fold = proved.open_body_scope(vec![RegionId(100), RegionId(101)])?;
    assert_eq!(fold.region(RegionId(7), 0)?, RegionId(100));
    Ok(())
}

#[test]
fn body_opening_rejects_incomplete_overlapping_and_malformed_alias_inputs() -> Result {
    let store = Store::default();
    let environment = environment(&store)?;
    assert!(environment.open_body_scope(vec![RegionId(100)]).is_err());
    assert!(
        environment
            .open_body_scope(vec![RegionId(7), RegionId(101)])
            .is_err()
    );
    for aliases in [
        vec![RegionId::STATIC],
        vec![RegionId::bound(0, 0)?],
        vec![RegionId(7), RegionId(7)],
    ] {
        assert!(
            Environment::new(&store, &[])?
                .with_body_aliases(aliases)
                .is_err()
        );
    }
    let mut fold = environment.open_body_scope(vec![RegionId(100), RegionId(101)])?;
    fold.steps = 1_000_000;
    assert!(fold.region(RegionId(7), 0).is_err());
    assert!(fold.steps > 1_000_000);
    let ordinary = Environment::new(&store, &[])?;
    assert!(ordinary.regions.is_none());
    let _fold = ordinary.open_body_scope(Vec::new())?;
    assert!(ordinary.regions.is_none());
    Ok(())
}

#[test]
fn query_environment_opens_both_namespaces_without_rewriting_the_owner() -> Result {
    let mut store = Store::default();
    let mut application = |region| -> Result<_> {
        let arguments = store.intern_arguments(&[Argument::Lifetime(region)])?;
        store.intern_trait_application(Application {
            item: ItemId(11),
            arguments,
        })
    };
    let declared = application(RegionId::bound(0, 0)?)?;
    let aliased = application(RegionId(7))?;
    let clauses = [
        Predicate {
            ty: TypeId::UNIT,
            bound: TraitBound::Trait(declared),
        }
        .into(),
        Predicate {
            ty: TypeId::UNIT,
            bound: TraitBound::Trait(aliased),
        }
        .into(),
        Predicate {
            ty: TypeId::UNIT,
            bound: TraitBound::MaybeConstTrait(aliased),
        }
        .into(),
    ];
    let environment = environment(&store)?.with_clauses(&store, &clauses)?;
    let mut fold = environment.open_body_scope(vec![RegionId(100), RegionId(101)])?;
    let query = environment.substitute_regions(&mut store, &mut fold)?;
    assert_eq!(environment.trait_clauses()?.collect::<Vec<_>>(), clauses);
    let opened = query.trait_clauses()?.collect::<Vec<_>>();
    assert_eq!(opened.len(), 3);
    assert_eq!(opened[0], opened[1]);
    let [
        crate::types::parameters::clauses::Clause::Predicate(ordinary),
        _,
        crate::types::parameters::clauses::Clause::Predicate(conditional),
    ] = opened.as_slice()
    else {
        return Err("whole copied clauses");
    };
    assert_eq!(
        ordinary.bound.trait_parts().map(|(id, _)| id),
        conditional.bound.trait_parts().map(|(id, _)| id)
    );
    assert_ne!(ordinary.bound, conditional.bound);
    assert!(query.body_aliases().is_none());
    assert!(environment.body_aliases().is_some());
    Ok(())
}

fn opening_work(width: u32, noise: u64) -> Result<(usize, usize, usize)> {
    let mut store = Store::default();
    for length in 0..noise {
        store.intern(Kind::Array {
            element: TypeId::UNIT,
            length,
        })?;
    }
    let environment = environment(&store)?;
    let byte = TypeId::primitive(Primitive::U8);
    let mut roots = Vec::new();
    for length in 0..u64::from(width) {
        let parent = store.intern(Kind::Array {
            element: byte,
            length,
        })?;
        roots.push(projected(&mut store, RegionId(7), parent, byte)?);
    }
    let mut fold = environment.open_body_scope(vec![RegionId(100), RegionId(101)])?;
    let mut expected = Vec::new();
    for &root in &roots {
        expected.push(fold.apply(&mut store, root)?);
    }
    let cold = fold.steps;
    let bytes = fold.retained_allocation().0;
    for (&root, expected) in roots.iter().zip(expected) {
        assert_eq!(fold.apply(&mut store, root)?, expected);
    }
    assert_eq!(fold.retained_allocation().0, bytes);
    let warm = fold.steps - cold;
    println!("body_opening width={width} noise={noise} cold={cold} warm={warm} retained={bytes}");
    Ok((cold, warm, bytes))
}

#[test]
fn body_opening_shares_repeated_roots_and_bounds_distinct_input_work() -> Result {
    let small = opening_work(8, 0)?;
    let middle = opening_work(32, 0)?;
    let large = opening_work(128, 0)?;
    assert_eq!(small, opening_work(8, 8192)?);
    assert_eq!(small.1, 0);
    assert_eq!(middle.1, 0);
    assert_eq!(large.1, 0);
    assert!(middle.0 <= small.0 * 4);
    assert!(large.0 <= middle.0 * 4);
    assert!(middle.2 <= small.2 * 4);
    assert!(large.2 <= middle.2 * 4);
    Ok(())
}
