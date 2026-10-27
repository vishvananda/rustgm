//! A private U scope shares declared rows and never turns missing membership into ability.
use super::*;
use crate::types::{
    ItemId, Kind,
    parameters::{Parameter, ParameterKind},
};
use std::cell::Cell;

pub(crate) fn controls() -> Result<(), Box<dyn std::error::Error>> {
    let mut types = Store::default();
    let mut parameters = Vec::new();
    for index in 0..128 {
        let id = types.intern_parameter(Parameter {
            owner: ItemId(1),
            index,
            kind: ParameterKind::Type { sized: true },
        })?;
        parameters.push((id, Abilities::default()));
    }
    let target = types.intern_parameter(Parameter {
        owner: ItemId(2),
        index: 0,
        kind: ParameterKind::Type { sized: false },
    })?;
    let base = Arc::new(Environment::new(&types, &parameters)?);
    let arena = types.arena_identity();
    let work = Cell::new(0_usize);
    let bytes = Cell::new(0_usize);
    let limit = Cell::new(0_usize);
    let pay = |steps, retained| {
        work.set(work.get().saturating_add(steps));
        bytes.set(bytes.get().saturating_add(retained));
        if bytes.get() > limit.get() {
            Err("test source retention exhausted")
        } else {
            Ok(())
        }
    };
    assert!(
        base.with_rigid_query_parameter(&types, &arena, target, &pay)
            .is_err()
    );
    let failed = (work.get(), bytes.get());
    assert!(failed.0 > 0 && failed.1 > 0);
    limit.set(1_000_000);
    let query = base.with_rigid_query_parameter(&types, &arena, target, &pay)?;
    assert!(work.get() > failed.0 && bytes.get() > failed.1);
    assert!(Arc::ptr_eq(
        query.parameters.prefix.as_ref().ok_or("shared prefix")?,
        &base.parameters
    ));
    assert_eq!(query.parameters.own.len(), 1);
    assert_eq!(base.parameters.own.len(), 128);
    assert_eq!(query.abilities(target), Some(Abilities::default()));
    for &(id, ability) in &parameters {
        assert_eq!(query.abilities(id), Some(ability));
    }
    assert!(
        base.with_rigid_query_parameter(&types, &arena, parameters[0].0, &pay)
            .is_err()
    );
    assert!(
        query
            .with_rigid_query_parameter(&types, &arena, target, &pay)
            .is_err()
    );
    let mutation = (*query).clone().with_predicates(&types, &[]);
    assert!(mutation.is_err());
    let foreign = Store::default();
    assert!(
        base.with_rigid_query_parameter(&foreign, &arena, target, &pay)
            .is_err()
    );
    let mut ordinary = crate::types::parameters::uses::Uses::default();
    let mut counted = crate::types::parameters::uses::Uses::default();
    let mut roots = Vec::new();
    for &(id, _) in &parameters {
        roots.push(types.intern(Kind::Parameter(id))?);
    }
    let leaves = types.intern_list(&roots)?;
    let diamond = types.intern(Kind::Tuple(leaves))?;
    let shared = types.intern_list(&[diamond, diamond, diamond])?;
    let root = types.intern(Kind::Tuple(shared))?;
    ordinary.collect(&types, root)?;
    counted.collect_counted(&types, root, &|growth| pay(growth.work, growth.bytes))?;
    assert_eq!(ordinary.parameters, counted.parameters);
    assert_eq!(ordinary.steps, counted.steps);
    assert_eq!(counted.parameters.len(), 128);
    println!(
        "rigid-scope declared={} capacity={} own={} capacity={} work={} bytes={} Uses={} Environment={}",
        base.parameters.own.len(),
        base.parameters.own.capacity(),
        query.parameters.own.len(),
        query.parameters.own.capacity(),
        work.get(),
        bytes.get(),
        size_of_val(&counted),
        size_of::<Environment>()
    );
    Ok(())
}
