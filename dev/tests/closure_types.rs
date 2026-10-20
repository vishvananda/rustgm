//! Closure graph invariants precede source inference; independent controls are recorded separately.
#[path = "closure_types/capture_properties.rs"]
mod capture_properties;
#[path = "closure_types/graph_constraints.rs"]
mod graph_constraints;
use rustgm::types::{
    GenericArgs, Kind, Store, TypeId,
    closure::{CallKind, ClosureId},
};

fn signature(store: &mut Store, input: TypeId) -> Result<TypeId, &'static str> {
    let parameters = store.intern_list(&[input])?;
    store.intern(Kind::Function {
        parameters,
        result: TypeId::UNIT,
        bound_regions: 0,
        unsafe_: false,
    })
}
fn closure(
    store: &mut Store,
    identity: ClosureId,
    signature: TypeId,
    arguments: GenericArgs,
    fields: &[TypeId],
) -> Result<TypeId, &'static str> {
    let fields = store.intern_list(fields)?;
    let environment = store.intern(Kind::Tuple(fields))?;
    store.intern(Kind::Closure {
        identity,
        arguments,
        signature,
        environment,
    })
}

#[test]
fn expression_identity_and_complete_parent_arguments_are_preserved() -> Result<(), &'static str> {
    let mut store = Store::default();
    let first = store.declare_closure(CallKind::Shared)?;
    let second = store.declare_closure(CallKind::Shared)?;
    let signature = signature(&mut store, TypeId::UNIT)?;
    let args = GenericArgs::EMPTY;
    let a = closure(&mut store, first, signature, args, &[])?;
    let b = closure(&mut store, second, signature, args, &[])?;
    let unused =
        store.intern_arguments(&[rustgm::types::arguments::Argument::Type(TypeId::UNIT)])?;
    let c = closure(&mut store, first, signature, unused, &[])?;
    assert_ne!(a, b);
    assert_ne!(a, c);
    assert_eq!(a, closure(&mut store, first, signature, args, &[])?);
    for (actual, expected) in [
        (CallKind::Shared, [true, true, true]),
        (CallKind::Mutable, [false, true, true]),
        (CallKind::Once, [false, false, true]),
    ] {
        for (required, accepted) in [CallKind::Shared, CallKind::Mutable, CallKind::Once]
            .into_iter()
            .zip(expected)
        {
            assert_eq!(actual.implements(required), accepted);
        }
    }
    // Compact constructor storage remains the same size as the prior type vocabulary.
    assert_eq!(std::mem::size_of::<Kind>(), 24);
    Ok(())
}

#[test]
fn malformed_environments_and_unpublished_identities_are_rejected() -> Result<(), &'static str> {
    let mut store = Store::default();
    let signature = signature(&mut store, TypeId::UNIT)?;
    let id = store.declare_closure(CallKind::Once)?;
    let valid = Kind::Closure {
        identity: id,
        arguments: GenericArgs::EMPTY,
        signature,
        environment: TypeId::UNIT,
    };
    for (identity, signature, environment) in [
        (ClosureId(8), signature, TypeId::UNIT),
        (id, TypeId::UNIT, TypeId::UNIT),
        (id, signature, signature),
    ] {
        assert!(
            store
                .intern(Kind::Closure {
                    identity,
                    signature,
                    environment,
                    arguments: GenericArgs::EMPTY,
                })
                .is_err()
        );
    }
    store.intern(valid)?;
    Ok(())
}
