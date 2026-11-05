//! Construction initializes owned input storage; authored statements stay deferred.
use super::Permit;
use crate::{
    body::{
        ir::{Action, Body, Owner, Rvalue, ScalarId, Statement, Value, ValueId, ValueList},
        regions::Contract,
    },
    resolve::{model::DefId, unit::NodeKey},
    sema::model::{Declarations, Scalar},
    types::{GenericArgs, Kind, Primitive, TypeId, instance::Instance},
};

pub(crate) struct Constructed {
    pub body: Body,
    pub contract: Contract,
    pub permit: Permit,
}

pub(crate) fn constructor(
    facts: &mut Declarations,
    instance: Instance,
    future: TypeId,
) -> Result<Constructed, &'static str> {
    let (definition, public, source) = inputs(facts, instance, future)?;
    let Some(Kind::Function {
        parameters,
        unsafe_,
        convention,
        bound_regions,
        ..
    }) = facts.types.kind(public)
    else {
        return Err("future construction lost its actual public signature");
    };
    let (types, payments) = facts.caller_input_store()?;
    let execution = types.intern_observed::<true>(
        Kind::Function {
            parameters,
            result: future,
            unsafe_,
            convention,
            bound_regions,
        },
        &|work, bytes| payments.charge(work, bytes),
    )?;
    facts.charge_caller_inputs(
        8,
        size_of::<Constructed>()
            + size_of::<crate::body::storage::Facts>()
            + size_of::<crate::body::storage::Scope>()
            + size_of::<crate::body::storage::Exit>()
            + 4 * (size_of::<Value>()
                + size_of::<Statement>()
                + size_of::<Scalar>()
                + size_of::<ValueId>()
                + size_of::<crate::body::ir::Block>()),
    )?;
    let contract =
        crate::body::regions::callable(facts, definition, definition, source, execution)?;
    let mut body = Body::new(
        Owner::Function {
            definition,
            signature: public,
            execution: Some(execution),
        },
        source,
        future,
    );
    body.instance = Some(instance);
    body.storage = Some(crate::body::storage::Facts::new(
        crate::body::scopes::Facts::synthetic(source.node),
    ));
    body.scalars.push(Scalar::Integer(0));
    body.values.push(Value {
        ty: TypeId::primitive(Primitive::U8),
        kind: Rvalue::Scalar(ScalarId(0)),
    });
    body.arguments.push(ValueId(0));
    body.values.push(Value {
        ty: future,
        kind: Rvalue::Aggregate {
            variant: None,
            fields: ValueList { start: 0, end: 1 },
        },
    });
    for value in [ValueId(0), ValueId(1)] {
        body.statements.push(Statement {
            source: source.node.into(),
            action: Action::Evaluate(value),
        });
    }
    crate::body::build::finish_factory(&mut body, ValueId(1))?;
    owned_return(facts, &mut body)?;
    let permit = Permit {
        identity: super::permit::Identity::Constructor(instance),
        future,
        public,
        execution,
        source,
        arena: facts.types.future_proof_arena(future)?,
    };
    validate(facts, &permit, &body, &contract)?;
    Ok(Constructed {
        body,
        contract,
        permit,
    })
}

fn owned_return(facts: &Declarations, body: &mut Body) -> Result<(), &'static str> {
    facts.charge_caller_inputs(
        4,
        size_of::<crate::body::drops::values::Facts>()
            + 4 * (size_of::<crate::body::ir::Temporary>()
                + size_of::<crate::body::drops::values::Release>())
            + crate::support::retained::new_map::<ValueId, usize>(1),
    )?;
    let mut needed = crate::types::destruction::NeedsDrop::new(
        facts.destructors.implementations().iter().map(|d| d.item),
    );
    crate::body::build::factory_owned_values(facts, body, &mut needed)?;
    Ok(())
}

fn inputs(
    facts: &Declarations,
    instance: Instance,
    future: TypeId,
) -> Result<(DefId, TypeId, NodeKey), &'static str> {
    let (definition, public, source) = empty_family(facts, future)?;
    if !matches!(facts.types.kind(instance.ty()), Some(Kind::FunctionItem { item, signature, arguments: GenericArgs::EMPTY }) if item.0 == definition.0 && signature == public)
    {
        return Err("future constructor differs from its actual full public item");
    }
    Ok((definition, public, source))
}

pub(super) fn empty_family(
    facts: &Declarations,
    future: TypeId,
) -> Result<(DefId, TypeId, NodeKey), &'static str> {
    let definition = facts.checked_future_definition(future)?;
    let function = facts
        .function_indices
        .get(&definition)
        .and_then(|&index| facts.functions.get(index))
        .ok_or("future construction lost its actual function ordinal")?;
    let signature = function.signature;
    let Some(Kind::Future {
        arguments: GenericArgs::EMPTY,
        ..
    }) = facts.types.kind(future)
    else {
        return Err("future constructor generic capture formation is not implemented");
    };
    let Some(Kind::Function {
        parameters,
        bound_regions: 0,
        ..
    }) = facts.types.kind(signature)
    else {
        return Err("future constructor binder transport is not implemented");
    };
    if facts.types.list(parameters) != Some([].as_slice()) {
        return Err(
            "future constructor input transfer requires its generated capture implementation",
        );
    }
    let body = facts
        .authored_async_body(function)?
        .ok_or("future construction has no actual authored source contract")?;
    if body.future != future {
        return Err("future construction differs from its actual primary application");
    }
    Ok((
        definition,
        signature,
        function
            .body
            .ok_or("future construction has no actual source body")?,
    ))
}

pub(super) fn validate(
    facts: &Declarations,
    permit: &Permit,
    body: &Body,
    contract: &Contract,
) -> Result<(), &'static str> {
    permit.require_current(&facts.types, body)?;
    let (_, public, source) = inputs(
        facts,
        permit
            .instance()
            .ok_or("future constructor lost its actual function item")?,
        permit.future,
    )?;
    if public != permit.public
        || source != permit.source
        || contract.key != body.key()
        || contract.source != body.source
        || contract.signature != Some(permit.execution)
        || contract.universals != 0
        || !contract.parameters.is_empty()
        || !body.locals.is_empty()
        || !body.projections.is_empty()
        || body.blocks.len() != 1
        || body.values.len() != 2
        || body.scalars != [Scalar::Integer(0)]
        || body.arguments != [ValueId(0)]
        || !matches!(body.values[0],Value {ty,kind:Rvalue::Scalar(ScalarId(0))} if ty==TypeId::primitive(Primitive::U8))
        || !matches!(body.values[1],Value {ty,kind:Rvalue::Aggregate {variant:None,fields:ValueList {start:0,end:1}}} if ty==permit.future)
        || !matches!(
            body.blocks[0].terminator,
            crate::body::ir::Terminator::Return(ValueId(1))
        )
    {
        return Err("future constructor differs from its actual dormant construction contract");
    }
    Ok(())
}
