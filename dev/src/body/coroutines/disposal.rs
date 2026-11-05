//! State-dependent cleanup owns the real frame, with no invented Drop declaration.
use super::{Constructed, Permit, permit::Identity};
use crate::{
    body::{
        ir::{
            Action, BinaryOperands, Block, BlockId, Body, Local, LocalId, LocalKind, Owner, Place,
            Projection, ProjectionId, ProjectionKind, Rvalue, ScalarId, Statement, Terminator,
            Value, ValueId,
        },
        regions::Contract,
        scopes, storage,
    },
    lex::token::Punctuation,
    sema::model::{Declarations, Scalar},
    types::{
        Kind, Primitive, RegionId, Store, TypeId,
        callable::Convention,
        future::{EntryKey, Operation},
    },
};

pub(crate) fn build(facts: &mut Declarations, future: TypeId) -> Result<Constructed, &'static str> {
    let (_, _, source) = super::creation::empty_family(facts, future)?;
    facts.charge_caller_inputs(
        512,
        size_of::<Constructed>()
            + size_of::<Option<&Permit>>()
            + size_of::<Local>()
            + 2 * size_of::<Projection>()
            + 6 * size_of::<Value>()
            + 12 * size_of::<Statement>()
            + 5 * size_of::<Block>()
            + 2 * size_of::<Scalar>()
            + size_of::<storage::Facts>()
            + size_of::<storage::Scope>()
            + 2 * size_of::<storage::Exit>(),
    )?;
    let (signature, reference, region) = signature(facts, future)?;
    let key = EntryKey {
        future,
        operation: Operation::Drop,
    };
    let mut body = Body::new(
        Owner::FutureEntry {
            key,
            signature,
            execution: signature,
        },
        source,
        TypeId::UNIT,
    );
    let mut storage = storage::Facts::new(scopes::Facts::synthetic(source.node));
    storage.exits = Vec::with_capacity(2);
    body.storage = Some(storage);
    frame(&mut body, reference, future, region);
    values(&mut body);
    flow(&mut body)?;
    let contract = Contract {
        key: body.key(),
        source: body.source,
        signature: Some(signature),
        universals: 1,
        assumptions: Vec::new(),
        parameters: Vec::new(),
        type_assumptions: Vec::new(),
        obligations: Vec::new(),
        proofs: Vec::new(),
        builtins: None,
        output: None,
        require_result_formation: false,
    };
    let permit = Permit {
        identity: Identity::Drop,
        future,
        public: signature,
        execution: signature,
        source,
        arena: facts.types.future_proof_arena(future)?,
    };
    permit.require_current(&facts.types, &body)?;
    Ok(Constructed {
        body,
        contract,
        permit,
    })
}
fn signature(
    facts: &mut Declarations,
    future: TypeId,
) -> Result<(TypeId, TypeId, RegionId), &'static str> {
    let region = RegionId::bound(0, 0)?;
    let (types, payments) = facts.caller_input_store()?;
    let pay = |work, bytes| payments.charge(work, bytes);
    let reference = types.intern_observed::<true>(
        Kind::Reference {
            region,
            mutable: true,
            referent: future,
        },
        &pay,
    )?;
    let parameters = types.intern_list_observed::<true>(&[reference], &pay)?;
    let signature = types.intern_observed::<true>(
        Kind::Function {
            parameters,
            result: TypeId::UNIT,
            unsafe_: false,
            convention: Convention::RUST,
            bound_regions: 1,
        },
        &pay,
    )?;
    Ok((signature, reference, region))
}
fn frame(body: &mut Body, reference: TypeId, future: TypeId, region: RegionId) {
    let source = body.root_site();
    body.locals = Vec::with_capacity(1);
    body.projections = Vec::with_capacity(2);
    body.locals.push(Local {
        scope: scopes::Id(0),
        storage_parent: None,
        ty: reference,
        kind: LocalKind::Parameter(0),
        definition: None,
        mutable: false,
        source,
    });
    body.projections.push(Projection {
        base: Place::Local(LocalId(0)),
        ty: future,
        kind: ProjectionKind::Dereference {
            region,
            mutable: true,
        },
    });
    body.projections.push(Projection {
        base: Place::Projection(ProjectionId(0)),
        ty: byte(),
        kind: ProjectionKind::FutureField(0),
    });
}
fn values(body: &mut Body) {
    body.scalars = Vec::with_capacity(2);
    body.values = Vec::with_capacity(6);
    body.scalars
        .extend([Scalar::Integer(0), Scalar::Integer(2)]);
    body.values.extend([
        Value {
            ty: byte(),
            kind: Rvalue::Read(Place::Projection(ProjectionId(1))),
        },
        Value {
            ty: byte(),
            kind: Rvalue::Scalar(ScalarId(0)),
        },
        Value {
            ty: boolean(),
            kind: equality(ValueId(1)),
        },
        Value {
            ty: byte(),
            kind: Rvalue::Scalar(ScalarId(1)),
        },
        Value {
            ty: boolean(),
            kind: equality(ValueId(3)),
        },
        Value {
            ty: TypeId::UNIT,
            kind: Rvalue::Unit,
        },
    ]);
}
fn flow(body: &mut Body) -> Result<(), &'static str> {
    let source = body.root_site();
    body.statements = Vec::with_capacity(12);
    body.blocks = Vec::with_capacity(5);
    for action in [
        Action::StorageLive(LocalId(0)),
        Action::Project(ProjectionId(0)),
        Action::Project(ProjectionId(1)),
        Action::Evaluate(ValueId(0)),
        Action::Evaluate(ValueId(1)),
        Action::Evaluate(ValueId(2)),
        Action::Evaluate(ValueId(3)),
        Action::Evaluate(ValueId(5)),
    ] {
        statement(body, action);
    }
    body.blocks.push(Block {
        start: 0,
        end: 8,
        source,
        terminator: Terminator::Branch {
            condition: ValueId(2),
            yes: BlockId(1),
            no: BlockId(2),
        },
    });
    statement(
        body,
        Action::Store {
            place: Place::Projection(ProjectionId(1)),
            value: ValueId(3),
        },
    );
    crate::body::build::finish_factory_block(body, 8, ValueId(5))?;
    statement(body, Action::Evaluate(ValueId(4)));
    body.blocks.push(Block {
        start: 10,
        end: 11,
        source,
        terminator: Terminator::Branch {
            condition: ValueId(4),
            yes: BlockId(3),
            no: BlockId(4),
        },
    });
    crate::body::build::finish_factory_block(body, 11, ValueId(5))?;
    body.blocks.push(Block {
        start: 12,
        end: 12,
        source,
        terminator: Terminator::Unreachable,
    });
    Ok(())
}
fn byte() -> TypeId {
    TypeId::primitive(Primitive::U8)
}
fn boolean() -> TypeId {
    TypeId::primitive(Primitive::Bool)
}
fn equality(right: ValueId) -> Rvalue {
    Rvalue::Binary {
        operands: BinaryOperands::Neither,
        operator: Punctuation::EqualEqual,
        left: ValueId(0),
        right,
    }
}
fn statement(body: &mut Body, action: Action) {
    body.statements.push(Statement {
        source: body.root_site(),
        action,
    });
}

pub(super) fn validate(types: &Store, body: &Body, permit: &Permit) -> Result<(), &'static str> {
    let Some(Kind::Future {
        identity,
        arguments,
    }) = types.kind(permit.future)
    else {
        return Err("future Drop lost its actual frame");
    };
    let storage = types.future_application(identity, arguments)?.storage;
    let Some(Kind::Function {
        parameters,
        result: TypeId::UNIT,
        unsafe_: false,
        convention: Convention::RUST,
        bound_regions: 1,
    }) = types.kind(permit.public)
    else {
        return Err("future Drop lost its genuine signature");
    };
    let region = RegionId::bound(0, 0)?;
    let [local] = body.locals.as_slice() else {
        return Err("future Drop requires its actual one frame parameter");
    };
    if types.list(storage) != Some([byte()].as_slice())
        || permit.public != permit.execution
        || types.list(parameters) != Some([local.ty].as_slice())
        || !matches!(types.kind(local.ty), Some(Kind::Reference { region: actual, mutable: true, referent }) if actual==region && referent==permit.future)
        || local.kind != LocalKind::Parameter(0)
        || local.definition.is_some()
        || local.mutable
        || local.scope != scopes::Id(0)
        || local.storage_parent.is_some()
        || local.source != body.root_site()
        || body.assembly.is_some()
        || !body.arguments.is_empty()
        || !body.temporaries.is_empty()
        || body.owned.is_some()
        || !body.guards.is_empty()
        || !body.guard_places.is_empty()
        || body.scalars != [Scalar::Integer(0), Scalar::Integer(2)]
    {
        return Err("future Drop differs from its exact empty-capture frame contract");
    }
    let [frame, state] = body.projections.as_slice() else {
        return Err("future Drop lost its frame/state coordinates");
    };
    if frame.base != Place::Local(LocalId(0))
        || frame.ty != permit.future
        || !matches!(frame.kind, ProjectionKind::Dereference { region: actual, mutable: true } if actual==region)
        || state.base != Place::Projection(ProjectionId(0))
        || state.ty != byte()
        || !matches!(state.kind, ProjectionKind::FutureField(0))
    {
        return Err("future Drop changed its actual state place");
    }
    graph(body)?;
    let storage = body
        .storage
        .as_ref()
        .ok_or("future Drop lost its storage endpoints")?;
    if !matches!(storage.scopes.as_slice(), [scope] if scope.parent.is_none() && scope.kind==scopes::Kind::Body && scope.source==body.root_site())
        || !matches!(storage.exits.as_slice(), [first,second] if first.location==storage::Location::At {block:BlockId(1),before:9} && second.location==storage::Location::At {block:BlockId(3),before:11} && first.end==storage::End::Scopes(storage::Range{inner:scopes::Id(0),outer:None}) && second.end==first.end)
    {
        return Err("future Drop changed its actual cleanup endpoints");
    }
    Ok(())
}

fn graph(body: &Body) -> Result<(), &'static str> {
    if body.values.len() != 6
        || body.statements.len() != 12
        || body.blocks.len() != 5
        || body.statements.iter().any(|s| s.source != body.root_site())
        || body.blocks.iter().any(|b| b.source != body.root_site())
    {
        return Err("future Drop changed its actual state graph");
    }
    for (index, value) in body.values.iter().enumerate() {
        let valid = match index {
            0 => {
                value.ty == byte()
                    && matches!(value.kind, Rvalue::Read(Place::Projection(ProjectionId(1))))
            }
            1 => value.ty == byte() && matches!(value.kind, Rvalue::Scalar(ScalarId(0))),
            2 | 4 => {
                value.ty == boolean()
                    && matches!(value.kind, Rvalue::Binary { operands: BinaryOperands::Neither, operator: Punctuation::EqualEqual, left: ValueId(0), right } if right==ValueId(if index==2 {1} else {3}))
            }
            3 => value.ty == byte() && matches!(value.kind, Rvalue::Scalar(ScalarId(1))),
            5 => value.ty == TypeId::UNIT && matches!(value.kind, Rvalue::Unit),
            _ => false,
        };
        if !valid {
            return Err("future Drop changed its actual state values");
        }
    }
    for (index, statement) in body.statements.iter().enumerate() {
        let valid = match index {
            0 => matches!(statement.action, Action::StorageLive(LocalId(0))),
            1 => matches!(statement.action, Action::Project(ProjectionId(0))),
            2 => matches!(statement.action, Action::Project(ProjectionId(1))),
            3..=7 => {
                matches!(statement.action, Action::Evaluate(id) if id==ValueId(if index==7 {5} else {u32::try_from(index-3).map_err(|_| "invalid future Drop statement")?}))
            }
            8 => matches!(
                statement.action,
                Action::Store {
                    place: Place::Projection(ProjectionId(1)),
                    value: ValueId(3)
                }
            ),
            9 | 11 => matches!(statement.action, Action::StorageDead(LocalId(0))),
            10 => matches!(statement.action, Action::Evaluate(ValueId(4))),
            _ => false,
        };
        if !valid {
            return Err("future Drop changed its state transfer/completion order");
        }
    }
    for (index, block) in body.blocks.iter().enumerate() {
        let (start, end) = [(0, 8), (8, 10), (10, 11), (11, 12), (12, 12)][index];
        let valid = match index {
            0 => matches!(
                block.terminator,
                Terminator::Branch {
                    condition: ValueId(2),
                    yes: BlockId(1),
                    no: BlockId(2)
                }
            ),
            1 | 3 => matches!(block.terminator, Terminator::Return(ValueId(5))),
            2 => matches!(
                block.terminator,
                Terminator::Branch {
                    condition: ValueId(4),
                    yes: BlockId(3),
                    no: BlockId(4)
                }
            ),
            4 => matches!(block.terminator, Terminator::Unreachable),
            _ => false,
        };
        if !valid || block.start != start || block.end != end {
            return Err("future Drop changed its completion/poisoned dispatch");
        }
    }
    Ok(())
}
