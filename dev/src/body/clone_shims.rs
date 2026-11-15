//! The language's `Clone` shim clones every stored member in order and copies a function.
use super::{
    collection_factories::{local, projection},
    ir::{
        Body, BodyKey, BorrowKind, LocalId, LocalKind, Owner, Place, Projection, ProjectionId,
        ProjectionKind, Rvalue, Terminator, Value, ValueId, ValueList,
    },
    provided::value,
    regions::Contract,
};
use crate::{
    resolve::{expansion::Report, package::Error},
    sema::{
        compiler_items::clone_shims::{Shim, callable, members, shim},
        members::Selection,
        model::Declarations,
    },
    types::{
        ItemId, Kind, RegionId, TypeId, callable::Convention, instance::Instance, substitute::Bound,
    },
};

#[cfg(test)]
#[path = "../../tests/invariants/memberwise_shims.rs"]
mod tests;

/// Only this module's validator mints the proof that a body is the real shim.
#[derive(Clone, Copy)]
pub(crate) struct Permit {
    instance: Instance,
}
impl Permit {
    pub(crate) const fn instance(self) -> Instance {
        self.instance
    }
}

/// The shim's own proof universe: its receiver borrow, then Self's lifetimes.
struct Interface {
    input: Contract,
    receiver: TypeId,
    subject: TypeId,
    /// Stored member types in clone order; a function has none and is copied.
    members: Option<Vec<TypeId>>,
}

fn interface(facts: &mut Declarations, selected: Shim) -> Result<Interface, &'static str> {
    let inputs = selected.inputs(facts)?;
    let subject = inputs.subject;
    let region = RegionId::bound(0, 0)?;
    let receiver = facts.types.intern(Kind::Reference {
        referent: subject,
        mutable: false,
        region,
    })?;
    let parameters = facts.types.intern_list(&[receiver])?;
    let signature = facts.types.intern(Kind::Function {
        parameters,
        result: subject,
        unsafe_: false,
        convention: Convention::RUST,
        bound_regions: inputs.premise.universals,
    })?;
    let mut input = super::regions::callable(
        facts,
        selected.definition,
        selected.definition,
        selected.source,
        signature,
    )?;
    if !input.type_assumptions.is_empty() {
        return Err("source Clone declares lifetime conditions its language shim cannot assume");
    }
    input.parameters.clear();
    input.assumptions.extend(inputs.premise.regions);
    input.type_assumptions.extend(inputs.premise.types);
    let members = members(&facts.types, subject)?;
    Ok(Interface {
        input,
        receiver,
        subject,
        members,
    })
}

pub(crate) fn build(
    report: &Report,
    facts: &mut Declarations,
    selected: Shim,
) -> Result<(Body, Contract), Error> {
    let error = |message| report.body_error(selected.source, message);
    let Interface {
        mut input,
        receiver,
        subject,
        members,
    } = interface(facts, selected).map_err(error)?;
    for ty in [receiver, subject] {
        facts
            .aggregates
            .prepare(&mut facts.types, ty)
            .map_err(error)?;
    }
    let mut body = Body::new(
        Owner::Function {
            definition: selected.definition,
            signature: selected.signature,
            execution: input.signature.filter(|&ty| ty != selected.signature),
        },
        selected.source,
        subject,
    );
    body.instance = Some(selected.instance);
    body.storage = Some(super::storage::Facts::new(super::scopes::Facts::synthetic(
        selected.source.node,
    )));
    let parameter = local(&mut body, receiver, LocalKind::Parameter(0), false).map_err(error)?;
    let kind = ProjectionKind::Dereference {
        region: RegionId::bound(0, 0).map_err(error)?,
        mutable: false,
    };
    let stored = projection(&mut body, Place::Local(parameter), subject, kind).map_err(error)?;
    let result = match &members {
        None => value(&mut body, subject, Rvalue::Read(stored)),
        Some(members) => memberwise(facts, &mut body, selected, (stored, subject), members),
    }
    .map_err(error)?;
    super::build::finish_factory(&mut body, result).map_err(error)?;
    let mut needed = crate::types::destruction::NeedsDrop::new(
        facts.destructors.implementations().iter().map(|d| d.item),
    );
    super::build::factory_owned_values(facts, &mut body, &mut needed).map_err(error)?;
    if members.is_some() {
        super::dispatch::resolve(facts, &mut body, &mut input)
            .map_err(|message| report.body_error(selected.source, message))?;
    }
    validate(facts, selected, &body, &input).map_err(error)?;
    Ok((body, input))
}

/// Borrow each stored member for the receiver's lifetime, call its own `clone`
/// and rebuild Self from the results. Dispatch then selects every callee.
fn memberwise(
    facts: &mut Declarations,
    body: &mut Body,
    selected: Shim,
    (stored, subject): (Place, TypeId),
    members: &[TypeId],
) -> Result<ValueId, &'static str> {
    let count = u32::try_from(members.len()).map_err(|_| "too many source Clone shim members")?;
    let mut cloned = Vec::with_capacity(members.len());
    for (index, &member) in (0..count).zip(members) {
        facts.aggregates.prepare(&mut facts.types, member)?;
        let field = ProjectionKind::Field(Selection::Tuple(index));
        let place = projection(body, stored, member, field)?;
        let callee = callable(facts, ItemId(selected.definition.0), member)?;
        let Some(Kind::FunctionItem { signature, .. }) = facts.types.kind(callee) else {
            return Err("source Clone shim member has no callable clone");
        };
        let opened =
            Bound::new(vec![RegionId::bound(0, 0)?]).signature(&mut facts.types, signature, 0)?;
        let Some(Kind::Function {
            parameters, result, ..
        }) = facts.types.kind(opened)
        else {
            return Err("source Clone shim member clone is not callable");
        };
        let [parameter] = facts
            .types
            .list(parameters)
            .ok_or("source Clone shim member clone lost its receiver")?
        else {
            return Err("source Clone shim member clone has a different arity");
        };
        let parameter = *parameter;
        if result != member {
            return Err("source Clone shim member clone returns a different type");
        }
        let kind = Rvalue::Borrow {
            place,
            kind: BorrowKind::Immediate,
        };
        let borrowed = value(body, parameter, kind)?;
        body.arguments.push(borrowed);
        let callee = value(body, callee, Rvalue::Function(selected.definition))?;
        let call = Rvalue::Call {
            signature: opened,
            callee,
            arguments: ValueList {
                start: index,
                end: index + 1,
            },
        };
        cloned.push(value(body, member, call)?);
    }
    body.arguments.extend(cloned);
    let rebuilt = if subject == TypeId::UNIT {
        Rvalue::Unit
    } else {
        Rvalue::Aggregate {
            variant: None,
            fields: ValueList {
                start: count,
                end: count
                    .checked_mul(2)
                    .ok_or("too many source Clone shim members")?,
            },
        }
    };
    value(body, subject, rebuilt)
}

pub(crate) fn validate(
    facts: &mut Declarations,
    selected: Shim,
    body: &Body,
    input: &Contract,
) -> Result<Permit, &'static str> {
    if shim(facts, selected.instance)? != Some(selected) {
        return Err("Clone shim differs from its actual canonical source method");
    }
    let expected = interface(facts, selected)?;
    let execution = expected
        .input
        .signature
        .filter(|&ty| ty != selected.signature);
    if body.instance != Some(selected.instance)
        || body.key() != BodyKey::Declaration(selected.definition)
        || body.source != selected.source.into()
        || body.result != expected.subject
        || !matches!(body.owner, Owner::Function {definition,signature,execution:actual} if definition==selected.definition && signature==selected.signature && actual==execution)
    {
        return Err("Clone shim differs from its actual canonical source method");
    }
    if input.key != expected.input.key
        || input.source != expected.input.source
        || input.signature != expected.input.signature
        || input.universals != expected.input.universals
        || input.assumptions != expected.input.assumptions
        || input.type_assumptions != expected.input.type_assumptions
        || !input.parameters.is_empty()
        || input.output.is_some()
        || input.require_result_formation
    {
        return Err("Clone shim lost its declared premise and lifetime contract");
    }
    let [local] = body.locals.as_slice() else {
        return Err("Clone shim requires exactly its receiver parameter");
    };
    if local.ty != expected.receiver
        || local.kind != LocalKind::Parameter(0)
        || local.mutable
        || local.definition.is_some()
        || local.storage_parent.is_some()
        || !matches!(body.projections.first(), Some(Projection {base:Place::Local(LocalId(0)),kind:ProjectionKind::Dereference {mutable:false,..},ty}) if *ty==expected.subject)
        || body.blocks.len() != 1
        || !body.scalars.is_empty()
        || !body.temporaries.is_empty()
        || !body.guards.is_empty()
        || !body.guard_places.is_empty()
        || body.assembly.is_some()
        || body.const_context
    {
        return Err("Clone shim changed its borrowed receiver or single block");
    }
    let last = match &expected.members {
        None => copied(body),
        Some(members) => rebuilt(body, members),
    }?;
    if !matches!(body.values.get(last.0 as usize), Some(value) if value.ty == expected.subject)
        || !matches!(body.blocks[0].terminator, Terminator::Return(value) if value == last)
    {
        return Err("Clone shim must return its rebuilt Self");
    }
    Ok(Permit {
        instance: selected.instance,
    })
}

/// A function item or pointer is one read through the receiver.
fn copied(body: &Body) -> Result<ValueId, &'static str> {
    let [
        Value {
            kind: Rvalue::Read(Place::Projection(ProjectionId(0))),
            ..
        },
    ] = body.values.as_slice()
    else {
        return Err("function Clone shim must copy exactly its receiver's referent");
    };
    if body.projections.len() != 1 || !body.arguments.is_empty() {
        return Err("function Clone shim must copy exactly its receiver's referent");
    }
    Ok(ValueId(0))
}

/// Each member is one field borrow, its selected callee and that call, in
/// order; the last value rebuilds Self from exactly those results.
fn rebuilt(body: &Body, members: &[TypeId]) -> Result<ValueId, &'static str> {
    let error = "Clone shim must clone each stored member once in order";
    let count = u32::try_from(members.len()).map_err(|_| error)?;
    let total = count.checked_mul(3).ok_or(error)?;
    if body.projections.len() != members.len() + 1
        || body.values.len() != total as usize + 1
        || body.arguments.len() != members.len() * 2
    {
        return Err(error);
    }
    for (index, &member) in (0..count).zip(members) {
        let (borrow, call) = (ValueId(index * 3), ValueId(index * 3 + 2));
        let place = ProjectionId(index + 1);
        let field = &body.projections[place.0 as usize];
        let values = &body.values[borrow.0 as usize..=call.0 as usize];
        if field.ty != member
            || field.base != Place::Projection(ProjectionId(0))
            || !matches!(field.kind, ProjectionKind::Field(Selection::Tuple(actual)) if actual==index)
            || !matches!(values[0].kind, Rvalue::Borrow {place:Place::Projection(actual),kind:BorrowKind::Immediate} if actual==place)
            || !matches!(values[1].kind, Rvalue::Function(_))
            || !matches!(values[2].kind, Rvalue::Call {callee,arguments:ValueList {start,end},..} if callee.0==borrow.0+1 && start==index && end==index+1)
            || values[2].ty != member
            || body.arguments[index as usize] != borrow
            || body.arguments[(count + index) as usize] != call
        {
            return Err(error);
        }
    }
    let fields = ValueList {
        start: count,
        end: count * 2,
    };
    match body.values[total as usize].kind {
        Rvalue::Unit if members.is_empty() => {}
        Rvalue::Aggregate {
            variant: None,
            fields: actual,
        } if actual.start == fields.start && actual.end == fields.end => {}
        _ => return Err(error),
    }
    Ok(ValueId(total))
}
