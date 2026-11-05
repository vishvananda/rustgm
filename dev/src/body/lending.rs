//! Async callable wrappers unpack the actual tuple and invoke the actual receiver.
use super::{
    ir::{
        Action, Body, BodyKey, Local, LocalId, LocalKind, Owner, Place, Projection, ProjectionId,
        ProjectionKind, Rvalue, Statement, Terminator, Value, ValueId, ValueList,
    },
    regions::Contract,
};
use crate::{
    resolve::{expansion::Report, package::Error},
    sema::{
        lending_methods::{Method, Validated},
        members::Selection,
        model::Declarations,
    },
    support::call_traits::CallTrait,
    types::{
        Kind, RegionId, TypeId, arguments::Argument, callable::Convention, instance::Instance,
        substitute::Bound,
    },
};

#[cfg(test)]
#[path = "../../tests/invariants/entry_wrappers.rs"]
mod tests;

pub(crate) struct Permit {
    instance: Instance,
}
impl Permit {
    pub(crate) const fn instance(&self) -> Instance {
        self.instance
    }
}

struct Interface {
    input: Contract,
    parameters: [TypeId; 2],
    result: TypeId,
    invocation: TypeId,
}

fn interface(facts: &mut Declarations, source: &Validated) -> Result<Interface, &'static str> {
    let (types, payments) = facts.caller_input_store()?;
    let pay = |work, bytes| payments.charge(work, bytes);
    // Map Self, tuple and future together before appending the method's own
    // receiver lifetime. Nested function binders and static remain distinct.
    let mut regions = Bound::universals_charged(0, &pay)?;
    let mut roots = [source.subject, source.tuple, source.future];
    for ty in &mut roots {
        let Argument::Type(mapped) =
            regions.apply_argument_charged(types, Argument::Type(*ty), &pay)?
        else {
            return Err("async wrapper lifetime import changed an argument kind");
        };
        *ty = mapped;
    }
    let [subject, tuple, result] = roots;
    let mut universals =
        u32::try_from(regions.universal_count()).map_err(|_| "too many async wrapper lifetimes")?;
    let receiver = if source.kind == CallTrait::FnOnce {
        subject
    } else {
        let region = RegionId::bound(
            0,
            u16::try_from(universals).map_err(|_| "too many async receiver lifetimes")?,
        )?;
        universals = universals
            .checked_add(1)
            .ok_or("async wrapper binder overflow")?;
        types.intern_observed::<true>(
            Kind::Reference {
                region,
                mutable: source.kind == CallTrait::FnMut,
                referent: subject,
            },
            &pay,
        )?
    };
    let parameters = types.intern_list_observed::<true>(&[receiver, tuple], &pay)?;
    let signature = types.intern_observed::<true>(
        Kind::Function {
            parameters,
            result,
            bound_regions: universals,
            unsafe_: false,
            convention: Convention::RUST,
        },
        &pay,
    )?;
    let Some(Kind::Tuple(fields)) = types.kind(tuple) else {
        return Err("async wrapper lost its tuple input");
    };
    let invocation = types.intern_observed::<true>(
        Kind::Function {
            parameters: fields,
            result,
            bound_regions: 0,
            unsafe_: false,
            convention: Convention::RUST,
        },
        &pay,
    )?;
    let input = Contract {
        key: BodyKey::Declaration(source.definition),
        source: source.source.into(),
        signature: Some(signature),
        universals,
        parameters: Vec::new(),
        assumptions: Vec::new(),
        type_assumptions: Vec::new(),
        obligations: Vec::new(),
        proofs: Vec::new(),
        builtins: None,
        output: None,
        require_result_formation: false,
    };
    Ok(Interface {
        input,
        parameters: [receiver, tuple],
        result,
        invocation,
    })
}

pub(crate) fn build(
    report: &Report,
    facts: &mut Declarations,
    selected: Method,
) -> Result<(Body, Contract), Error> {
    let source = selected
        .validate(facts)
        .map_err(|message| Error::at(crate::resolve::unit::FileId(0), 0, message))?;
    let error = |message| report.body_error(source.source, message);
    let interface = interface(facts, &source).map_err(error)?;
    for ty in [
        source.subject,
        source.tuple,
        source.future,
        interface.parameters[0],
        interface.parameters[1],
        interface.result,
    ] {
        facts
            .aggregates
            .prepare(&mut facts.types, ty)
            .map_err(error)?;
    }
    let mut body = Body::new(
        Owner::Function {
            definition: source.definition,
            signature: source.signature,
            execution: interface
                .input
                .signature
                .filter(|&ty| ty != source.signature),
        },
        source.source,
        interface.result,
    );
    body.instance = Some(selected.instance());
    let Some(Kind::Function {
        parameters: fields, ..
    }) = facts.types.kind(interface.invocation)
    else {
        return Err(error(
            "async wrapper lost its ordinary invocation signature",
        ));
    };
    let fields = facts
        .types
        .list(fields)
        .ok_or_else(|| error("async wrapper has invalid tuple fields"))?;
    reserve(facts, &mut body, fields.len()).map_err(error)?;
    body.storage = Some(super::storage::Facts::new(super::scopes::Facts::synthetic(
        source.source.node,
    )));
    parameters(&mut body, interface.parameters);
    let result = operation(&mut body, fields, &interface, source.kind).map_err(error)?;
    super::build::finish_factory(&mut body, result).map_err(error)?;
    Ok((body, interface.input))
}

fn parameters(body: &mut Body, parameters: [TypeId; 2]) {
    for (ordinal, ty) in [0_u32, 1].into_iter().zip(parameters) {
        body.locals.push(Local {
            scope: super::scopes::Id(0),
            storage_parent: ordinal.checked_sub(1).map(LocalId),
            ty,
            kind: LocalKind::Parameter(ordinal),
            definition: None,
            mutable: false,
            source: body.root_site(),
        });
        statement(body, Action::StorageLive(LocalId(ordinal)));
    }
}

fn operation(
    body: &mut Body,
    fields: &[TypeId],
    interface: &Interface,
    kind: CallTrait,
) -> Result<ValueId, &'static str> {
    evaluate(
        body,
        interface.parameters[0],
        Rvalue::Read(Place::Local(LocalId(0))),
    )?;
    body.arguments.push(ValueId(0));
    for (index, &ty) in fields.iter().enumerate() {
        let ordinal = u32::try_from(index).map_err(|_| "async tuple ordinal overflow")?;
        body.projections.push(Projection {
            base: Place::Local(LocalId(1)),
            kind: ProjectionKind::Field(Selection::Tuple(ordinal)),
            ty,
        });
        statement(body, Action::Project(ProjectionId(ordinal)));
        let value = evaluate(
            body,
            ty,
            Rvalue::Read(Place::Projection(ProjectionId(ordinal))),
        )?;
        body.arguments.push(value);
    }
    let count =
        u32::try_from(body.arguments.len()).map_err(|_| "too many async invocation inputs")?;
    evaluate(
        body,
        interface.result,
        Rvalue::Invoke {
            kind,
            signature: interface.invocation,
            arguments: ValueList {
                start: 0,
                end: count,
            },
        },
    )
}

fn reserve(facts: &Declarations, body: &mut Body, fields: usize) -> Result<(), &'static str> {
    if fields > 100_000 {
        return Err("async wrapper tuple exceeds its work budget");
    }
    let values = fields.saturating_add(2);
    let statements = fields.saturating_mul(2).saturating_add(5);
    let bytes = size_of::<Body>()
        + size_of::<Contract>()
        + 2 * size_of::<Local>()
        + fields * size_of::<Projection>()
        + values * size_of::<Value>()
        + statements * size_of::<Statement>()
        + (fields + 1) * size_of::<ValueId>()
        + size_of::<super::ir::Block>()
        + size_of::<super::storage::Scope>()
        + size_of::<super::storage::Exit>();
    facts.charge_caller_inputs(statements, bytes)?;
    body.locals.reserve_exact(2);
    body.projections.reserve_exact(fields);
    body.values.reserve_exact(values);
    body.statements.reserve_exact(statements);
    body.arguments.reserve_exact(fields + 1);
    body.blocks.reserve_exact(1);
    Ok(())
}

fn statement(body: &mut Body, action: Action) {
    body.statements.push(Statement {
        source: body.root_site(),
        action,
    });
}
fn evaluate(body: &mut Body, ty: TypeId, kind: Rvalue) -> Result<ValueId, &'static str> {
    // The complete body was bounded before allocating any graph records.
    let value =
        ValueId(u32::try_from(body.values.len()).map_err(|_| "too many async wrapper values")?);
    body.values.push(Value { ty, kind });
    statement(body, Action::Evaluate(value));
    Ok(value)
}

pub(crate) fn validate(
    facts: &mut Declarations,
    selected: Method,
    body: &Body,
    input: &Contract,
) -> Result<Permit, &'static str> {
    let source = selected.validate(facts)?;
    let expected = interface(facts, &source)?;
    if body.instance != Some(selected.instance())
        || body.source != source.source.into()
        || body.key() != BodyKey::Declaration(source.definition)
        || body.result != expected.result
        || !matches!(body.owner, Owner::Function { definition, signature, execution }
            if definition == source.definition && signature == source.signature
                && execution == expected.input.signature.filter(|&ty| ty != source.signature))
        || input.key != expected.input.key
        || input.source != expected.input.source
        || input.signature != expected.input.signature
        || input.universals != expected.input.universals
        || !input.parameters.is_empty()
        || !input.assumptions.is_empty()
        || !input.type_assumptions.is_empty()
        || !input.obligations.is_empty()
        || !input.proofs.is_empty()
        || input.builtins.is_some()
        || input.output.is_some()
        || input.require_result_formation
    {
        return Err("async wrapper differs from its actual independent method contract");
    }
    validate_graph(facts, body, &expected, source.kind)?;
    Ok(Permit {
        instance: selected.instance(),
    })
}

fn validate_graph(
    facts: &Declarations,
    body: &Body,
    expected: &Interface,
    kind: CallTrait,
) -> Result<(), &'static str> {
    let Some(Kind::Function { parameters, .. }) = facts.types.kind(expected.invocation) else {
        return Err("async wrapper lost its invocation signature");
    };
    let fields = facts
        .types
        .list(parameters)
        .ok_or("invalid async wrapper field list")?;
    let count = fields.len();
    if body.locals.len() != 2
        || body.projections.len() != count
        || body.values.len() != count + 2
        || body.arguments.len() != count + 1
        || body.statements.len() != 2 * count + 5
        || body.blocks.len() != 1
        || !body.scalars.is_empty()
        || !body.temporaries.is_empty()
        || !body.guards.is_empty()
        || !body.guard_places.is_empty()
        || body.assembly.is_some()
        || body.static_types.is_some()
        || body.mutable_statics.is_some()
        || body.const_context
        || body.owned.is_some()
        || body.promotions.is_some()
        || body.resolved_callables.is_some()
        || body.closure_calls.is_some()
    {
        return Err("async wrapper changed its exact tuple invocation graph");
    }
    validate_parameters(body, expected)?;
    validate_fields(body, fields)?;
    validate_result(body, expected, kind, count)?;
    if body
        .statements
        .iter()
        .any(|statement| statement.source != body.root_site())
    {
        return Err("async wrapper changed its actual operation source");
    }
    validate_storage(body, count)
}

fn validate_parameters(body: &Body, expected: &Interface) -> Result<(), &'static str> {
    for (index, local) in body.locals.iter().enumerate() {
        let ordinal = u32::try_from(index).map_err(|_| "async local ordinal overflow")?;
        if local.ty != expected.parameters[index]
            || local.kind != LocalKind::Parameter(ordinal)
            || local.scope != super::scopes::Id(0)
            || local.storage_parent != ordinal.checked_sub(1).map(LocalId)
            || local.definition.is_some()
            || local.mutable
            || local.source != body.root_site()
        {
            return Err("async wrapper changed its parameter storage");
        }
    }
    if !matches!(body.values[0], Value { ty, kind: Rvalue::Read(Place::Local(LocalId(0))) } if ty == expected.parameters[0])
        || !matches!(body.statements[0].action, Action::StorageLive(LocalId(0)))
        || !matches!(body.statements[1].action, Action::StorageLive(LocalId(1)))
        || !matches!(body.statements[2].action, Action::Evaluate(ValueId(0)))
    {
        return Err("async wrapper must evaluate its real receiver exactly once");
    }
    Ok(())
}

fn validate_fields(body: &Body, fields: &[TypeId]) -> Result<(), &'static str> {
    for (index, &ty) in fields.iter().enumerate() {
        let ordinal = u32::try_from(index).map_err(|_| "async field ordinal overflow")?;
        let projection = body.projections[index];
        if projection.base != Place::Local(LocalId(1))
            || projection.ty != ty
            || !matches!(projection.kind, ProjectionKind::Field(Selection::Tuple(field)) if field == ordinal)
            || !matches!(body.values[index + 1], Value { ty: actual, kind: Rvalue::Read(Place::Projection(field)) } if actual == ty && field == ProjectionId(ordinal))
            || !matches!(body.statements[3 + 2 * index].action, Action::Project(field) if field == ProjectionId(ordinal))
            || !matches!(body.statements[4 + 2 * index].action, Action::Evaluate(value) if value == ValueId(ordinal + 1))
        {
            return Err("async wrapper changed a projected argument or its evaluation order");
        }
    }
    for (index, &value) in body.arguments.iter().enumerate() {
        if usize::try_from(value.0).map_err(|_| "async argument ordinal overflow")? != index {
            return Err("async wrapper changed its invocation arguments");
        }
    }
    Ok(())
}

fn validate_result(
    body: &Body,
    expected: &Interface,
    kind: CallTrait,
    count: usize,
) -> Result<(), &'static str> {
    let result = u32::try_from(count + 1).map_err(|_| "async result ordinal overflow")?;
    if !matches!(body.values[count + 1], Value { ty, kind: Rvalue::Invoke { kind: actual, signature, arguments } }
        if ty == expected.result && actual == kind && signature == expected.invocation && arguments.start == 0 && arguments.end == result)
        || !matches!(body.statements[3 + 2 * count].action, Action::Evaluate(value) if value == ValueId(result))
        || !matches!(
            body.statements[4 + 2 * count].action,
            Action::StorageEnd {
                first: LocalId(0),
                last: LocalId(1)
            }
        )
        || body.blocks[0].start != 0
        || usize::try_from(body.blocks[0].end).ok() != Some(body.statements.len())
        || body.blocks[0].source != body.root_site()
        || !matches!(body.blocks[0].terminator, Terminator::Return(value) if value == ValueId(result))
    {
        return Err("async wrapper must return its real invocation result");
    }
    Ok(())
}

fn validate_storage(body: &Body, fields: usize) -> Result<(), &'static str> {
    let storage = body
        .storage
        .as_ref()
        .ok_or("async wrapper lost its parameter storage")?;
    let [scope] = storage.scopes.as_slice() else {
        return Err("async wrapper changed its storage scopes");
    };
    let [exit] = storage.exits.as_slice() else {
        return Err("async wrapper changed its storage exits");
    };
    let before = u32::try_from(4 + 2 * fields).map_err(|_| "async storage location overflow")?;
    if scope.parent.is_some()
        || scope.source != body.root_site()
        || scope.kind != super::scopes::Kind::Body
        || storage.scope_work != 1
        || !matches!(exit.location, super::storage::Location::At { block: super::ir::BlockId(0), before: actual } if actual == before)
        || exit.end
            != super::storage::End::Scopes(super::storage::Range {
                inner: super::scopes::Id(0),
                outer: None,
            })
    {
        return Err("async wrapper must end exactly its actual parameter scope");
    }
    Ok(())
}
