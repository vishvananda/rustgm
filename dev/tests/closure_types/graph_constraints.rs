use super::{closure, signature};
use rustgm::types::{
    GenericArgs, ItemId, Kind, Primitive, RegionId, Store, TypeId,
    arguments::Argument,
    closure::CallKind,
    infer::{Constructor, Context, Domain, Error, arguments::Argument as Inferred},
    parameters::{Parameter, ParameterKind},
    substitute::Bound,
};

#[test]
fn signature_environment_and_unused_inputs_share_inference_and_occurs_checks()
-> Result<(), Box<dyn std::error::Error>> {
    let mut store = Store::default();
    let identity = store.declare_closure(CallKind::Shared)?;
    let mut context = Context::default();
    let hole = context.variable(Domain::Any)?;
    let signature = context.construct(
        Constructor::Function {
            bound_regions: 0,
            parameters: 1,
            unsafe_: false,
            convention: rustgm::types::callable::Convention::RUST,
        },
        &[hole, hole],
    )?;
    let environment = context.construct(Constructor::Tuple, &[hole])?;
    let value = context.closure(identity, signature, environment, &[Inferred::Type(hole)])?;
    assert_eq!(context.resolved(&mut store, value), Err(Error::Unresolved));
    let byte = context.known(&store, TypeId::primitive(Primitive::U8))?;
    context.equate(&store, hole, byte)?;
    let resolved = context.resolved(&mut store, value)?;
    let mut destination = Context::default();
    let imported = destination.import(&store, &mut context, value)?;
    assert_eq!(destination.resolved(&mut store, imported)?, resolved);
    let other = context.known(&store, TypeId::primitive(Primitive::Bool))?;
    assert_eq!(context.equate(&store, hole, other), Err(Error::Mismatch));

    for capture in [false, true] {
        let mut context = Context::default();
        let hole = context.variable(Domain::Any)?;
        let signature = super::signature(&mut store, TypeId::UNIT)?;
        let signature = context.known(&store, signature)?;
        let fields = if capture { vec![hole] } else { vec![] };
        let environment = context.construct(Constructor::Tuple, &fields)?;
        let args = if capture {
            vec![]
        } else {
            vec![Inferred::Type(hole)]
        };
        let value = context.closure(identity, signature, environment, &args)?;
        assert_eq!(context.equate(&store, hole, value), Err(Error::Occurs));
    }
    Ok(())
}

#[test]
fn substitution_keeps_callable_binders_separate_from_environment_regions()
-> Result<(), Box<dyn std::error::Error>> {
    let mut store = Store::default();
    let parameter = store.intern_parameter(Parameter {
        owner: ItemId(1),
        index: 0,
        kind: ParameterKind::Type { sized: true },
    })?;
    let ty = store.intern(Kind::Parameter(parameter))?;
    let field = store.intern(Kind::Reference {
        mutable: false,
        region: RegionId::bound(0, 0)?,
        referent: ty,
    })?;
    let input = store.intern(Kind::Reference {
        mutable: false,
        region: RegionId::bound(0, 0)?,
        referent: ty,
    })?;
    let signature = signature(&mut store, input)?;
    let Some(Kind::Function { parameters, .. }) = store.kind(signature) else {
        return Err("signature".into());
    };
    let signature = store.intern(Kind::Function {
        parameters,
        result: input,
        bound_regions: 1,
        unsafe_: false,
        convention: rustgm::types::callable::Convention::RUST,
    })?;
    let args = store.intern_arguments(&[
        Argument::Type(ty),
        Argument::Lifetime(RegionId::bound(0, 0)?),
    ])?;
    let identity = store.declare_closure(CallKind::Shared)?;
    let original = closure(&mut store, identity, signature, args, &[field])?;
    let byte = TypeId::primitive(Primitive::U8);
    let typed = Bound::parameters(&store, &[(parameter, Argument::Type(byte))])?
        .apply(&mut store, original)?;
    let result = Bound::new(vec![RegionId(7)]).apply(&mut store, typed)?;
    let Some(Kind::Closure {
        signature,
        arguments,
        environment,
        ..
    }) = store.kind(result)
    else {
        return Err("closure".into());
    };
    assert_eq!(
        store
            .arguments(arguments)
            .ok_or("arguments")?
            .collect::<Vec<_>>(),
        [Argument::Type(byte), Argument::Lifetime(RegionId(7))]
    );
    let Some(Kind::Tuple(fields)) = store.kind(environment) else {
        return Err("environment".into());
    };
    let field = store.list(fields).ok_or("fields")?[0];
    assert_eq!(
        store.kind(field),
        Some(Kind::Reference {
            mutable: false,
            region: RegionId(7),
            referent: byte
        })
    );
    let Some(Kind::Function {
        result,
        bound_regions,
        ..
    }) = store.kind(signature)
    else {
        return Err("signature".into());
    };
    assert_eq!(bound_regions, 1);
    assert_eq!(
        store.kind(result),
        Some(Kind::Reference {
            mutable: false,
            region: RegionId::bound(0, 0)?,
            referent: byte
        })
    );
    Ok(())
}

#[test]
fn distinct_closure_identities_cannot_unify_even_with_equal_signatures()
-> Result<(), Box<dyn std::error::Error>> {
    let mut store = Store::default();
    let signature = signature(&mut store, TypeId::UNIT)?;
    let a = store.declare_closure(CallKind::Shared)?;
    let b = store.declare_closure(CallKind::Shared)?;
    let a = closure(&mut store, a, signature, GenericArgs::EMPTY, &[])?;
    let b = closure(&mut store, b, signature, GenericArgs::EMPTY, &[])?;
    let mut context = Context::default();
    let a = context.known(&store, a)?;
    let b = context.known(&store, b)?;
    assert_eq!(context.equate(&store, a, b), Err(Error::Mismatch));
    Ok(())
}

#[test]
fn same_expression_relations_require_equal_capture_regions()
-> Result<(), Box<dyn std::error::Error>> {
    use rustgm::types::{relations::Constraints, variance::Variances};
    let mut store = Store::default();
    let id = store.declare_closure(CallKind::Shared)?;
    let signature = signature(&mut store, TypeId::UNIT)?;
    let a = RegionId::bound(0, 0)?;
    let b = RegionId::bound(0, 1)?;
    let mut values = Vec::new();
    for region in [a, b] {
        let field = store.intern(Kind::Reference {
            mutable: false,
            region,
            referent: TypeId::UNIT,
        })?;
        values.push(closure(
            &mut store,
            id,
            signature,
            GenericArgs::EMPTY,
            &[field],
        )?);
    }
    let variances = Variances::default();
    for equal in [false, true] {
        let mut constraints = Constraints::new(&store, &variances, 2, 1000)?;
        constraints.assume(a, b)?;
        if equal {
            constraints.assume(b, a)?;
        }
        constraints.coerce(values[0], values[1], false, 0)?;
        assert_eq!(constraints.solve().is_ok(), equal);
    }
    Ok(())
}
