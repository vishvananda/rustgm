//! The real `FnPtr` address shim is one typed read and cast, never a pointer call.
use super::{
    ir::{
        Action, Body, BodyKey, Local, LocalId, LocalKind, Owner, Place, Rvalue, Statement,
        Terminator, Value, ValueId,
    },
    regions::Contract,
};
use crate::{
    resolve::{expansion::Report, package::Error},
    sema::{
        compiler_items::pointers::{Address, address},
        model::{Cast, Declarations},
    },
    types::{Kind, TypeId, instance::Instance, substitute::Bound},
};

#[cfg(test)]
#[path = "../../tests/invariants/shim_contracts.rs"]
mod tests;

#[derive(Clone, Copy)]
pub(crate) struct Permit {
    instance: Instance,
}
impl Permit {
    pub(crate) const fn instance(self) -> Instance {
        self.instance
    }
}

fn contract(
    facts: &mut Declarations,
    selected: Address,
) -> Result<(Contract, TypeId), &'static str> {
    let mut regions = Bound::universals(0);
    let receiver = regions.apply(&mut facts.types, selected.receiver)?;
    let count = u32::try_from(regions.universal_count())
        .map_err(|_| "too many function-pointer shim lifetimes")?;
    let parameters = facts.types.intern_list(&[receiver])?;
    let signature = facts.types.intern(Kind::Function {
        bound_regions: count,
        parameters,
        result: selected.result,
        unsafe_: false,
        convention: crate::types::callable::Convention::RUST,
    })?;
    let mut input = super::regions::callable(
        facts,
        selected.definition,
        selected.definition,
        selected.source,
        signature,
    )?;
    input.parameters.clear();
    Ok((input, receiver))
}

pub(crate) fn build(
    report: &Report,
    facts: &mut Declarations,
    selected: Address,
) -> Result<(Body, Contract), Error> {
    let error = |message| report.body_error(selected.source, message);
    let (input, receiver) = contract(facts, selected).map_err(error)?;
    let mut body = Body::new(
        Owner::Function {
            definition: selected.definition,
            signature: selected.signature,
            execution: input
                .signature
                .filter(|&signature| signature != selected.signature),
        },
        selected.source,
        selected.result,
    );
    body.instance = Some(selected.instance);
    body.storage = Some(super::storage::Facts::new(super::scopes::Facts::synthetic(
        selected.source.node,
    )));
    body.locals.push(Local {
        scope: super::scopes::Id(0),
        storage_parent: None,
        ty: receiver,
        kind: LocalKind::Parameter(0),
        definition: None,
        mutable: false,
        source: selected.source.node.into(),
    });
    body.values.push(Value {
        ty: receiver,
        kind: Rvalue::Read(Place::Local(LocalId(0))),
    });
    body.values.push(Value {
        ty: selected.result,
        kind: Rvalue::Cast {
            direct_variant: false,
            kind: Cast::Pointer,
            operand: ValueId(0),
        },
    });
    for action in [
        Action::StorageLive(LocalId(0)),
        Action::Evaluate(ValueId(0)),
        Action::Evaluate(ValueId(1)),
    ] {
        body.statements.push(Statement {
            source: body.root_site(),
            action,
        });
    }
    super::build::finish_factory(&mut body, ValueId(1)).map_err(error)?;
    validate(facts, selected, &body, &input).map_err(error)?;
    Ok((body, input))
}

pub(crate) fn validate(
    facts: &mut Declarations,
    selected: Address,
    body: &Body,
    input: &Contract,
) -> Result<Permit, &'static str> {
    if address(facts, selected.instance)? != Some(selected) {
        return Err("function-pointer shim differs from its actual canonical source method");
    }
    let (expected, receiver) = contract(facts, selected)?;
    let execution = expected
        .signature
        .filter(|&signature| signature != selected.signature);
    if body.instance != Some(selected.instance)
        || body.key() != BodyKey::Declaration(selected.definition)
        || body.source != selected.source.into()
        || body.result != selected.result
        || !matches!(body.owner, Owner::Function {definition,signature,execution:actual} if definition==selected.definition && signature==selected.signature && actual==execution)
    {
        return Err("function-pointer shim differs from its actual canonical source method");
    }
    if input.key != expected.key
        || input.source != expected.source
        || input.signature != expected.signature
        || input.universals != expected.universals
        || !input.parameters.is_empty()
        || !input.assumptions.is_empty()
        || !input.type_assumptions.is_empty()
        || !input.obligations.is_empty()
        || !input.proofs.is_empty()
        || input.builtins.is_some()
        || input.output.is_some()
        || input.require_result_formation
    {
        return Err("function-pointer shim lost its independent lifetime contract");
    }
    let [local] = body.locals.as_slice() else {
        return Err("function-pointer shim requires exactly one parameter");
    };
    if local.ty != receiver
        || local.kind != LocalKind::Parameter(0)
        || local.mutable
        || local.definition.is_some()
        || local.storage_parent.is_some()
        || local.scope != super::scopes::Id(0)
        || body.values.len() != 2
        || body.statements.len() != 4
        || body.blocks.len() != 1
        || !body.projections.is_empty()
        || !body.arguments.is_empty()
        || !body.scalars.is_empty()
        || !body.temporaries.is_empty()
        || !body.guards.is_empty()
        || !body.guard_places.is_empty()
        || body.assembly.is_some()
        || body.closure_calls.is_some()
        || body.const_context
    {
        return Err("function-pointer shim changed its exact read/cast body");
    }
    if !matches!(body.values[0], Value {ty,kind:Rvalue::Read(Place::Local(LocalId(0)))} if ty==receiver)
        || !matches!(body.values[1], Value {ty,kind:Rvalue::Cast {direct_variant:false,kind:Cast::Pointer,operand:ValueId(0)}} if ty==selected.result)
        || !matches!(body.statements[0].action, Action::StorageLive(LocalId(0)))
        || !matches!(body.statements[1].action, Action::Evaluate(ValueId(0)))
        || !matches!(body.statements[2].action, Action::Evaluate(ValueId(1)))
        || !matches!(body.statements[3].action, Action::StorageDead(LocalId(0)))
        || body.blocks[0].start != 0
        || body.blocks[0].end != 4
        || !matches!(body.blocks[0].terminator, Terminator::Return(ValueId(1)))
    {
        return Err("function-pointer shim must return the real pointer without invoking it");
    }
    Ok(Permit {
        instance: selected.instance,
    })
}
