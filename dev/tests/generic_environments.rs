use rustgm::types::{
    GenericArgs, ItemId, Kind, Primitive, RegionId, Store, TypeId,
    aggregate::Definitions,
    copy::{Copies, Properties},
    parameters::{
        Parameter, ParameterId, ParameterKind,
        environment::{Abilities, Environment},
    },
};
use std::{collections::HashMap, sync::Arc};

fn parameter(store: &mut Store, owner: u32) -> Result<(ParameterId, TypeId), &'static str> {
    let id = store.intern_parameter(Parameter {
        owner: ItemId(owner),
        index: 0,
        kind: ParameterKind::Type { sized: true },
    })?;
    Ok((id, store.intern(Kind::Parameter(id))?))
}

#[test]
fn move_abilities_follow_the_fixed_environment() -> Result<(), &'static str> {
    let mut types = Store::default();
    let (id, ty) = parameter(&mut types, 0)?;
    let definitions = Definitions::default();
    let derives = HashMap::new();
    let tuple = types.intern_list(&[ty, TypeId::primitive(Primitive::U8)])?;
    let tuple = types.intern(Kind::Tuple(tuple))?;
    let array = types.intern(Kind::Array {
        element: tuple,
        length: 4,
    })?;
    let empty = types.intern(Kind::Array {
        element: ty,
        length: 0,
    })?;
    for (abilities, expected) in [
        (Abilities::default(), Properties::default()),
        (
            Abilities {
                copy: false,
                clone_: true,
            },
            Properties {
                copy: false,
                clone_: true,
            },
        ),
        (
            Abilities {
                copy: true,
                clone_: false,
            },
            Properties {
                copy: true,
                clone_: true,
            },
        ),
    ] {
        let environment = Arc::new(Environment::new(&types, &[(id, abilities)])?);
        let mut query = Copies::in_environment(Arc::clone(&environment));
        assert_eq!(
            query.demand(&types, &definitions, &derives, array)?,
            expected
        );
        let steps = query.steps;
        assert_eq!(
            query.demand(&types, &definitions, &derives, array)?,
            expected
        );
        assert_eq!(query.steps, steps);
        assert_eq!(
            query.demand(&types, &definitions, &derives, empty)?,
            expected
        );
    }
    // A closed query must still reject the same canonical type after successful
    // symbolic queries. The environment is never a global property of a TypeId.
    let mut closed = Copies::default();
    assert!(
        closed
            .demand(&types, &definitions, &derives, array)
            .is_err()
    );
    let steps = closed.steps;
    assert!(
        closed
            .demand(&types, &definitions, &derives, array)
            .is_err()
    );
    assert_eq!(closed.steps, steps);
    Ok(())
}

#[test]
fn scopes_and_failed_queries_do_not_leak_assumptions() -> Result<(), &'static str> {
    let mut types = Store::default();
    let (a, a_type) = parameter(&mut types, 0)?;
    let (b, b_type) = parameter(&mut types, 1)?;
    let definitions = Definitions::default();
    let derives = HashMap::new();
    let environment = Arc::new(Environment::new(&types, &[(a, Abilities::default())])?);
    assert_eq!(environment.abilities(a), Some(Abilities::default()));
    assert_eq!(environment.abilities(b), None);
    let mut query = Copies::in_environment(environment);
    assert_eq!(
        query.demand(&types, &definitions, &derives, b_type),
        Err("Copy parameter is outside the current environment")
    );
    let steps = query.steps;
    assert!(
        query
            .demand(&types, &definitions, &derives, b_type)
            .is_err()
    );
    assert_eq!(query.steps, steps);
    assert_eq!(
        query.demand(&types, &definitions, &derives, a_type)?,
        Properties::default()
    );
    let copy = Abilities {
        copy: true,
        clone_: false,
    };
    let environment = Arc::new(Environment::new(&types, &[(b, copy)])?);
    let mut other = Copies::in_environment(environment);
    assert!(other.demand(&types, &definitions, &derives, b_type)?.copy);
    assert!(
        other
            .demand(&types, &definitions, &derives, a_type)
            .is_err()
    );
    // Cached failures in one query do not poison independent concrete roots.
    assert!(
        query
            .demand(&types, &definitions, &derives, TypeId::UNIT)?
            .copy
    );
    Ok(())
}

#[test]
fn references_and_callable_abilities_do_not_depend_on_referents() -> Result<(), &'static str> {
    let mut types = Store::default();
    let (id, ty) = parameter(&mut types, 2)?;
    let environment = Arc::new(Environment::new(&types, &[(id, Abilities::default())])?);
    let mut query = Copies::in_environment(environment);
    let definitions = Definitions::default();
    let derives = HashMap::new();
    let parameters = types.intern_list(&[ty])?;
    let signature = types.intern(Kind::Function {
        bound_regions: 0,
        parameters,
        result: ty,
        unsafe_: false,
    })?;
    let item = types.intern(Kind::FunctionItem {
        item: ItemId(3),
        signature,
        arguments: GenericArgs::EMPTY,
    })?;
    let shared = types.intern(Kind::Reference {
        mutable: false,
        region: RegionId::STATIC,
        referent: ty,
    })?;
    let mutable = types.intern(Kind::Reference {
        mutable: true,
        region: RegionId::STATIC,
        referent: ty,
    })?;
    for root in [signature, item, shared] {
        let abilities = query.demand(&types, &definitions, &derives, root)?;
        assert!(abilities.copy && abilities.clone_);
    }
    assert_eq!(
        query.demand(&types, &definitions, &derives, mutable)?,
        Properties::default()
    );
    Ok(())
}

#[test]
fn environment_construction_validates_declarations_and_merges_bounds() -> Result<(), &'static str> {
    let mut types = Store::default();
    let (id, _) = parameter(&mut types, 0)?;
    let copy = Abilities {
        copy: true,
        clone_: false,
    };
    let clone_ = Abilities {
        copy: false,
        clone_: true,
    };
    let environment = Environment::new(&types, &[(id, clone_), (id, copy)])?;
    assert_eq!(
        environment.abilities(id),
        Some(Abilities {
            copy: true,
            clone_: true
        })
    );
    let constant = types.intern_parameter(Parameter {
        owner: ItemId(0),
        index: 1,
        kind: ParameterKind::Const {
            ty: TypeId::primitive(Primitive::Usize),
        },
    })?;
    assert!(Environment::new(&types, &[(constant, copy)]).is_err());
    assert!(Environment::new(&types, &[(ParameterId(u32::MAX), copy)]).is_err());
    let excessive = vec![(id, copy); 1_000_001];
    assert!(Environment::new(&types, &excessive).is_err());
    Ok(())
}

#[test]
fn shared_type_graph_is_queried_once_per_environment() -> Result<(), &'static str> {
    let mut types = Store::default();
    let (id, mut root) = parameter(&mut types, 0)?;
    let environment = Arc::new(Environment::new(&types, &[(id, Abilities::default())])?);
    for _ in 0..4096 {
        let list = types.intern_list(&[root, root])?;
        root = types.intern(Kind::Tuple(list))?;
    }
    let mut query = Copies::in_environment(environment);
    assert!(
        !query
            .demand(&types, &Definitions::default(), &HashMap::new(), root)?
            .copy
    );
    assert_eq!(query.steps, 4097);
    Ok(())
}
