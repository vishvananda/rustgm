//! The first scalar family moves its checked body into a genuine required poll entry.
use super::{Constructed, Permit, permit::Identity};
use crate::{
    body::{
        instantiate::Materialized,
        ir::{
            Action, BinaryOperands, Block, BlockId, Body, Local, LocalId, LocalKind, Owner, Place,
            Projection, ProjectionId, ProjectionKind, Rvalue, ScalarId, Statement, Terminator,
            Value, ValueId, ValueList,
        },
        scopes, storage,
    },
    lex::token::Punctuation,
    sema::{
        coroutine_methods::Entry,
        members::Selection,
        model::{Declarations, Scalar},
    },
    types::{Kind, Primitive, Store, TypeId, callable::Convention},
};

pub(crate) fn build(
    facts: &mut Declarations,
    application: Materialized,
    entry: &Entry,
) -> Result<Constructed, &'static str> {
    entry.method.validate(facts)?;
    let (_, _, source) = super::creation::empty_family(facts, entry.method.key().future)?;
    let (mut body, mut contract) = application.into_empty_async_parts()?;
    require_source(&body)?;
    if body.promotions.is_some() {
        return Err("selected scalar poll must finish its actual promotion materialization");
    }
    if !matches!(body.owner, Owner::AsyncApplication { future, .. } if future==entry.method.key().future)
        || body.source != source.into()
        || contract.key != body.key()
        || contract.source != body.source
        || contract.universals != 0
        || !contract.parameters.is_empty()
        || !contract.assumptions.is_empty()
        || !contract.type_assumptions.is_empty()
        || !contract.obligations.is_empty()
        || !contract.proofs.is_empty()
        || contract.output.is_some()
    {
        return Err("scalar poll requires its actual closed checked application contract");
    }
    prepay(facts, &body)?;
    body.owner = Owner::FutureEntry {
        key: entry.method.key(),
        signature: entry.public,
        execution: entry.execution,
    };
    body.instance = Some(entry.method.instance());
    body.result = entry.result;
    frame(&facts.types, &mut body, entry)?;
    // The execution frame has new nominal applications after ordinary materialization.
    // Reuse canonical field substitution before any immutable ownership/layout view.
    for local in &body.locals {
        facts.aggregates.prepare(&mut facts.types, local.ty)?;
    }
    facts.aggregates.prepare(&mut facts.types, body.result)?;
    cleanup_free_protocol(facts, &body)?;
    values(&mut body, entry);
    flow(&mut body)?;
    contract.key = body.key();
    contract.signature = Some(entry.execution);
    contract.universals = 3;
    let permit = Permit {
        identity: Identity::Poll {
            instance: entry.method.instance(),
            pin_field: entry.pin_field,
            ready: entry.ready,
        },
        future: entry.method.key().future,
        public: entry.public,
        execution: entry.execution,
        source,
        arena: facts.types.future_proof_arena(entry.method.key().future)?,
    };
    permit.require_current(&facts.types, &body)?;
    Ok(Constructed {
        body,
        contract,
        permit,
    })
}

fn require_source(body: &Body) -> Result<(), &'static str> {
    if body.result != byte()
        || body.instance.is_some()
        || body.const_context
        || !body.locals.is_empty()
        || !body.projections.is_empty()
        || !body.arguments.is_empty()
        || !body.temporaries.is_empty()
        || body.owned.is_some()
        || !body.guards.is_empty()
        || !body.guard_places.is_empty()
        || body.assembly.is_some()
        || body.static_types.is_some()
        || body.mutable_statics.is_some()
        || body
            .promotions
            .as_ref()
            .is_some_and(|promotions| !promotions.is_empty())
        || body.closure_calls.is_some()
        || body.has_resolved_callables()
        || body.scalars.len() != 1
        || !matches!(body.values.as_slice(), [value] if value.ty==byte() && matches!(value.kind,Rvalue::Scalar(ScalarId(0))))
        || !matches!(body.statements.as_slice(), [statement] if matches!(statement.action,Action::Evaluate(ValueId(0))))
        || !matches!(body.blocks.as_slice(), [block] if block.start==0 && block.end==1 && matches!(block.terminator,Terminator::Return(ValueId(0))))
    {
        return Err(
            "future poll requires the actual scalar body profile; wider CFG remains required",
        );
    }
    lexical_endpoints(
        body,
        storage::Location::At {
            block: BlockId(0),
            before: 1,
        },
    )
}

fn frame(types: &Store, body: &mut Body, entry: &Entry) -> Result<(), &'static str> {
    empty_storage(types, entry.method.key().future)?;
    let Some(Kind::Function {
        parameters,
        result,
        unsafe_: false,
        convention: Convention::RUST,
        bound_regions: 3,
    }) = types.kind(entry.execution)
    else {
        return Err("poll lost its actual three-slot execution signature");
    };
    let [pin, context] = types.list(parameters).ok_or("poll lost its input list")? else {
        return Err("poll requires actual Pin and Context parameters");
    };
    let Some(Kind::Adt(_, arguments)) = types.kind(*pin) else {
        return Err("poll receiver lost actual Pin identity");
    };
    let mut arguments = types
        .arguments(arguments)
        .ok_or("poll receiver arguments are invalid")?;
    let Some(crate::types::arguments::Argument::Type(pointer)) = arguments.next() else {
        return Err("poll receiver requires its actual one pointer argument");
    };
    if arguments.next().is_some() {
        return Err("poll receiver has unexpected extra pointer arguments");
    }
    let Some(Kind::Reference {
        region,
        mutable: true,
        referent,
    }) = types.kind(pointer)
    else {
        return Err("poll frame lost its actual mutable Pin pointer");
    };
    if referent != entry.method.key().future || result != entry.result {
        return Err("poll frame differs from its exact future/result application");
    }
    for (ordinal, ty) in [*pin, *context].into_iter().enumerate() {
        body.locals.push(Local {
            scope: scopes::Id(0),
            storage_parent: if ordinal == 0 { None } else { Some(LocalId(0)) },
            ty,
            kind: LocalKind::Parameter(
                u32::try_from(ordinal).map_err(|_| "invalid poll parameter")?,
            ),
            definition: None,
            mutable: false,
            source: body.root_site(),
        });
    }
    body.projections.extend([
        Projection {
            base: Place::Local(LocalId(0)),
            kind: ProjectionKind::Field(Selection::Nominal(entry.pin_field)),
            ty: pointer,
        },
        Projection {
            base: Place::Projection(ProjectionId(0)),
            kind: ProjectionKind::Dereference {
                region,
                mutable: true,
            },
            ty: referent,
        },
        Projection {
            base: Place::Projection(ProjectionId(1)),
            kind: ProjectionKind::FutureField(0),
            ty: byte(),
        },
    ]);
    Ok(())
}

fn values(body: &mut Body, entry: &Entry) {
    body.scalars
        .extend([Scalar::Integer(0), Scalar::Integer(1), Scalar::Integer(2)]);
    body.arguments.push(ValueId(0));
    body.values.extend([
        Value {
            ty: byte(),
            kind: Rvalue::Read(Place::Projection(ProjectionId(2))),
        },
        Value {
            ty: byte(),
            kind: Rvalue::Scalar(ScalarId(1)),
        },
        Value {
            ty: TypeId::primitive(Primitive::Bool),
            kind: Rvalue::Binary {
                operands: BinaryOperands::Neither,
                operator: Punctuation::EqualEqual,
                left: ValueId(1),
                right: ValueId(2),
            },
        },
        Value {
            ty: byte(),
            kind: Rvalue::Scalar(ScalarId(2)),
        },
        Value {
            ty: byte(),
            kind: Rvalue::Scalar(ScalarId(3)),
        },
        Value {
            ty: entry.result,
            kind: Rvalue::Aggregate {
                variant: Some(entry.ready),
                fields: ValueList { start: 0, end: 1 },
            },
        },
    ]);
}

fn flow(body: &mut Body) -> Result<(), &'static str> {
    let original = body
        .statements
        .pop()
        .ok_or("poll lost its actual source statement")?;
    let original_block = body
        .blocks
        .pop()
        .ok_or("poll lost its actual source block")?;
    for action in [
        Action::StorageLive(LocalId(0)),
        Action::StorageLive(LocalId(1)),
        Action::Project(ProjectionId(0)),
        Action::Project(ProjectionId(1)),
        Action::Project(ProjectionId(2)),
        Action::Evaluate(ValueId(1)),
        Action::Evaluate(ValueId(2)),
        Action::Evaluate(ValueId(3)),
        Action::Evaluate(ValueId(4)),
        Action::Evaluate(ValueId(5)),
    ] {
        statement(body, action);
    }
    body.blocks.push(Block {
        start: 0,
        end: 10,
        source: body.root_site(),
        terminator: Terminator::Branch {
            condition: ValueId(3),
            yes: BlockId(1),
            no: BlockId(2),
        },
    });
    statement(
        body,
        Action::Store {
            place: Place::Projection(ProjectionId(2)),
            value: ValueId(4),
        },
    );
    let source_start =
        u32::try_from(body.statements.len()).map_err(|_| "too many poll statements")?;
    body.statements.push(original);
    statement(body, Action::Evaluate(ValueId(6)));
    statement(
        body,
        Action::Store {
            place: Place::Projection(ProjectionId(2)),
            value: ValueId(5),
        },
    );
    remap_endpoints(body, original_block.end, source_start)?;
    statement(body, Action::StorageDead(LocalId(1)));
    statement(body, Action::StorageDead(LocalId(0)));
    body.blocks.push(Block {
        start: 10,
        end: 16,
        source: original_block.source,
        terminator: Terminator::Return(ValueId(6)),
    });
    body.blocks.push(Block {
        start: 16,
        end: 16,
        source: body.root_site(),
        terminator: Terminator::Unreachable,
    });
    Ok(())
}

fn remap_endpoints(
    body: &mut Body,
    source_end: u32,
    source_start: u32,
) -> Result<(), &'static str> {
    let completion =
        u32::try_from(body.statements.len()).map_err(|_| "too many poll statements")?;
    let storage = body
        .storage
        .as_mut()
        .ok_or("poll lost its actual source storage")?;
    for exit in &mut storage.exits {
        let storage::Location::At {
            block: BlockId(0),
            before,
        } = exit.location
        else {
            return Err("scalar poll lost its actual source-block exit");
        };
        let mapped = before
            .checked_add(source_start)
            .and_then(|before| before.checked_add(2))
            .ok_or("poll scope endpoint offset overflow")?;
        if before != source_end || mapped != completion {
            return Err("poll source exit differs from its checked completion offset");
        }
        exit.location = storage::Location::At {
            block: BlockId(1),
            before: mapped,
        };
    }
    Ok(())
}

fn statement(body: &mut Body, action: Action) {
    body.statements.push(Statement {
        source: body.root_site(),
        action,
    });
}
fn byte() -> TypeId {
    TypeId::primitive(Primitive::U8)
}

fn prepay(facts: &Declarations, body: &Body) -> Result<(), &'static str> {
    use crate::support::retained::Growth;
    let mut growth = Growth::default();
    growth.include(Growth::vector::<Local>(
        body.locals.len(),
        body.locals.capacity(),
        2,
    ));
    growth.include(Growth::vector::<Projection>(
        body.projections.len(),
        body.projections.capacity(),
        3,
    ));
    growth.include(Growth::vector::<Value>(
        body.values.len(),
        body.values.capacity(),
        6,
    ));
    growth.include(Growth::vector::<Statement>(
        body.statements.len(),
        body.statements.capacity(),
        15,
    ));
    growth.include(Growth::vector::<Block>(
        body.blocks.len(),
        body.blocks.capacity(),
        2,
    ));
    growth.include(Growth::vector::<Scalar>(
        body.scalars.len(),
        body.scalars.capacity(),
        3,
    ));
    growth.include(Growth::vector::<ValueId>(
        body.arguments.len(),
        body.arguments.capacity(),
        1,
    ));
    facts.charge_caller_inputs(
        512usize.saturating_add(growth.work),
        size_of::<Constructed>().saturating_add(growth.bytes),
    )
}

pub(super) fn require_original(original: &Body, body: &Body) -> Result<(), &'static str> {
    require_source(original)?;
    if body.source != original.source
        || body.scalars.first() != original.scalars.first()
        || body.values.first().map(|value| value.ty)
            != original.values.first().map(|value| value.ty)
        || body.statements.get(11).map(|statement| statement.source)
            != original
                .statements
                .first()
                .map(|statement| statement.source)
        || body.blocks.get(1).map(|block| block.source)
            != original.blocks.first().map(|block| block.source)
    {
        return Err("generated poll changed its actual checked source scalar segment");
    }
    let before = original
        .storage
        .as_ref()
        .ok_or("checked scalar source lost its actual lexical scope")?;
    let after = body
        .storage
        .as_ref()
        .ok_or("generated scalar poll lost its actual lexical scope")?;
    if before.scope_work != after.scope_work
        || before.scopes.len() != after.scopes.len()
        || before.exits.len() != after.exits.len()
        || before
            .scopes
            .iter()
            .zip(&after.scopes)
            .any(|(before, after)| {
                before.parent != after.parent
                    || before.source != after.source
                    || before.kind != after.kind
            })
        || before
            .exits
            .iter()
            .zip(&after.exits)
            .any(|(before, after)| before.end != after.end)
    {
        return Err("generated poll changed its checked source lexical identity");
    }
    Ok(())
}

pub(super) fn validate(types: &Store, body: &Body, permit: &Permit) -> Result<(), &'static str> {
    let Identity::Poll {
        instance,
        pin_field,
        ready,
    } = permit.identity
    else {
        return Err("poll validation requires its actual private operation");
    };
    let Some(Kind::FunctionItem { signature, .. }) = types.kind(instance.ty()) else {
        return Err("poll permit lost its actual required function item");
    };
    let Some(Kind::Function {
        parameters,
        result,
        unsafe_: false,
        convention: Convention::RUST,
        bound_regions: 3,
    }) = types.kind(permit.execution)
    else {
        return Err("poll permit lost its actual execution signature");
    };
    empty_storage(types, permit.future)?;
    if signature != permit.public
        || types.contains_parameters(instance.ty()) != Some(false)
        || types.contains_projections(instance.ty()) != Some(false)
        || body.source != permit.source.into()
        || body.instance != Some(instance)
        || body.result != result
        || body.const_context
        || !matches!(body.owner,Owner::FutureEntry {key,signature,execution} if Some(key)==permit.entry() && signature==permit.public && execution==permit.execution)
        || body.locals.len() != 2
        || body.projections.len() != 3
        || !body.temporaries.is_empty()
        || body.owned.is_some()
        || !body.guards.is_empty()
        || !body.guard_places.is_empty()
        || body.assembly.is_some()
        || body.arguments != [ValueId(0)]
        || body.scalars.len() != 4
        || body.scalars[1..] != [Scalar::Integer(0), Scalar::Integer(1), Scalar::Integer(2)]
    {
        return Err("poll changed its actual family, signature or scalar frame");
    }
    frame_contract(types, body, permit, parameters, pin_field)?;
    graph(body, ready)?;
    lexical_endpoints(
        body,
        storage::Location::At {
            block: BlockId(1),
            before: 14,
        },
    )
}

fn lexical_endpoints(body: &Body, location: storage::Location) -> Result<(), &'static str> {
    let storage = body
        .storage
        .as_ref()
        .ok_or("scalar poll lost its actual source scope chain")?;
    if !matches!(storage.scopes.as_slice(),[root,block,tail] if root.parent.is_none()
        && root.kind==scopes::Kind::Body && block.parent==Some(scopes::Id(0))
        && block.kind==scopes::Kind::Block && tail.parent==Some(scopes::Id(1))
        && tail.kind==scopes::Kind::Tail)
        || storage.exits.len() != 3
    {
        return Err("scalar poll requires its actual Body/Block/Tail scope chain");
    }
    for (exit, (inner, outer)) in storage.exits.iter().zip([
        (scopes::Id(2), Some(scopes::Id(1))),
        (scopes::Id(1), Some(scopes::Id(0))),
        (scopes::Id(0), None),
    ]) {
        if exit.location != location
            || exit.end != storage::End::Scopes(storage::Range { inner, outer })
        {
            return Err("scalar poll changed an actual ordered lexical endpoint");
        }
    }
    Ok(())
}

fn frame_contract(
    types: &Store,
    body: &Body,
    permit: &Permit,
    parameters: crate::types::ListId,
    pin_field: crate::sema::members::FieldId,
) -> Result<(), &'static str> {
    let [pin, context] = types
        .list(parameters)
        .ok_or("poll parameter list is invalid")?
    else {
        return Err("poll lost its actual two parameters");
    };
    for (ordinal, (local, ty)) in body.locals.iter().zip([*pin, *context]).enumerate() {
        if local.ty != ty
            || local.scope != scopes::Id(0)
            || local.definition.is_some()
            || local.mutable
            || local.source != body.root_site()
            || local.kind
                != LocalKind::Parameter(
                    u32::try_from(ordinal).map_err(|_| "invalid poll parameter")?,
                )
            || local.storage_parent != if ordinal == 0 { None } else { Some(LocalId(0)) }
        {
            return Err("poll changed its actual public parameter storage");
        }
    }
    let [pointer, frame, state] = body.projections.as_slice() else {
        return Err("poll lost its actual Pin/frame/state coordinates");
    };
    let Some(Kind::Reference {
        region,
        mutable: true,
        referent,
    }) = types.kind(pointer.ty)
    else {
        return Err("poll lost its actual Pin pointer type");
    };
    if pointer.base != Place::Local(LocalId(0))
        || !matches!(pointer.kind,ProjectionKind::Field(Selection::Nominal(actual)) if actual==pin_field)
        || frame.base != Place::Projection(ProjectionId(0))
        || frame.ty != permit.future
        || referent != permit.future
        || !matches!(frame.kind,ProjectionKind::Dereference{region:actual,mutable:true} if actual==region)
        || state.base != Place::Projection(ProjectionId(1))
        || state.ty != byte()
        || !matches!(state.kind, ProjectionKind::FutureField(0))
    {
        return Err("poll changed its exact authenticated frame/state place");
    }
    Ok(())
}

fn empty_storage(types: &Store, future: TypeId) -> Result<(), &'static str> {
    let Some(Kind::Future {
        identity,
        arguments,
    }) = types.kind(future)
    else {
        return Err("scalar poll requires its actual full future application");
    };
    if arguments != crate::types::GenericArgs::EMPTY
        || types.list(types.future_application(identity, arguments)?.storage)
            != Some([byte()].as_slice())
    {
        return Err("scalar poll requires its genuine empty-capture state storage");
    }
    Ok(())
}

/// Here only: Pin owns a reference, Poll owns u8, and Context remains borrowed.
/// Present nominal cleanup therefore needs a wider, genuinely ordered transform.
fn cleanup_free_protocol(facts: &Declarations, body: &Body) -> Result<(), &'static str> {
    let catalog = facts
        .source_bindings
        .as_deref()
        .ok_or("poll cleanup requires its actual source inventory")?;
    catalog.require_source_destructors_prepared()?;
    catalog.require_async_destructor_registration()?;
    let Some(Kind::Adt(pin, _)) = body
        .locals
        .first()
        .and_then(|local| facts.types.kind(local.ty))
    else {
        return Err("poll cleanup lost its actual owned Pin input");
    };
    let Some(Kind::Adt(poll, arguments)) = facts.types.kind(body.result) else {
        return Err("poll cleanup lost its actual Ready result");
    };
    let mut arguments = facts
        .types
        .arguments(arguments)
        .ok_or("poll cleanup result arguments are invalid")?;
    if arguments.next() != Some(crate::types::arguments::Argument::Type(byte()))
        || arguments.next().is_some()
    {
        return Err("scalar poll requires its actual u8 output cleanup contract");
    }
    facts.charge_caller_inputs(2, 0)?;
    for item in [pin, poll] {
        if facts.destructors.item(item).is_some() || facts.destructors.async_item(item).is_some() {
            return Err("poll requires actual owned Pin/Poll cleanup before completion");
        }
    }
    Ok(())
}

fn graph(body: &Body, ready: crate::sema::members::VariantId) -> Result<(), &'static str> {
    if body.values.len() != 7 || body.statements.len() != 16 || body.blocks.len() != 3 {
        return Err("poll changed its actual state graph dimensions");
    }
    for (index, value) in body.values.iter().enumerate() {
        let valid = match index {
            0 => value.ty == byte() && matches!(value.kind, Rvalue::Scalar(ScalarId(0))),
            1 => {
                value.ty == byte()
                    && matches!(value.kind, Rvalue::Read(Place::Projection(ProjectionId(2))))
            }
            2 | 4 | 5 => {
                value.ty == byte()
                    && matches!(value.kind,Rvalue::Scalar(id) if id==ScalarId(if index==2{1}else{u32::try_from(index-2).map_err(|_| "invalid poll scalar")?}))
            }
            3 => {
                value.ty == TypeId::primitive(Primitive::Bool)
                    && matches!(
                        value.kind,
                        Rvalue::Binary {
                            operands: BinaryOperands::Neither,
                            operator: Punctuation::EqualEqual,
                            left: ValueId(1),
                            right: ValueId(2)
                        }
                    )
            }
            6 => {
                value.ty == body.result
                    && matches!(value.kind,Rvalue::Aggregate{variant:Some(actual),fields:ValueList{start:0,end:1}} if actual==ready)
            }
            _ => false,
        };
        if !valid {
            return Err("poll changed its actual scalar/state/Ready values");
        }
    }
    for (index, statement) in body.statements.iter().enumerate() {
        let valid = match index {
            0 | 1 => {
                matches!(statement.action,Action::StorageLive(id) if id==LocalId(u32::try_from(index).map_err(|_| "invalid poll local")?))
            }
            2..=4 => {
                matches!(statement.action,Action::Project(id) if id==ProjectionId(u32::try_from(index-2).map_err(|_| "invalid poll place")?))
            }
            5..=9 => {
                matches!(statement.action,Action::Evaluate(id) if id==ValueId(u32::try_from(index-4).map_err(|_| "invalid poll value")?))
            }
            10 | 13 => {
                matches!(statement.action,Action::Store{place:Place::Projection(ProjectionId(2)),value} if value==ValueId(if index==10{4}else{5}))
            }
            11 => matches!(statement.action, Action::Evaluate(ValueId(0))),
            12 => matches!(statement.action, Action::Evaluate(ValueId(6))),
            14 | 15 => {
                matches!(statement.action,Action::StorageDead(id) if id==LocalId(u32::from(index==14)))
            }
            _ => false,
        };
        if !valid || (index != 11 && statement.source != body.root_site()) {
            return Err("poll changed its poison/source/completion ordering");
        }
    }
    for (index, block) in body.blocks.iter().enumerate() {
        let (start, end) = [(0, 10), (10, 16), (16, 16)][index];
        let valid = match index {
            0 => matches!(
                block.terminator,
                Terminator::Branch {
                    condition: ValueId(3),
                    yes: BlockId(1),
                    no: BlockId(2)
                }
            ),
            1 => matches!(block.terminator, Terminator::Return(ValueId(6))),
            2 => matches!(block.terminator, Terminator::Unreachable),
            _ => false,
        };
        if !valid
            || block.start != start
            || block.end != end
            || (index != 1 && block.source != body.root_site())
        {
            return Err("poll changed its entry/completion state dispatch");
        }
    }
    Ok(())
}
