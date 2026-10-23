//! Defining const permission consumes actual typed source facts, not signatures alone.
use crate::{
    body::ir::{Action, Body, Owner, Rvalue},
    resolve::model::DefId,
    sema::{
        invocations::Invocation,
        model::{Declarations, Operation},
    },
    support::{call_traits::CallTrait, constness::Constness, hash::HashSet},
    types::{Kind, TypeId, arguments::Argument, parameters::ParameterKind},
};
pub(super) struct Work {
    pub steps: usize,
    limit: usize,
}
impl Work {
    pub(super) const fn new(limit: usize) -> Self {
        Self { steps: 0, limit }
    }
    fn charge(&mut self, amount: usize) -> Result<(), &'static str> {
        self.steps = self.steps.saturating_add(amount);
        if self.steps > self.limit {
            return Err("const invocation validation work budget exceeded");
        }
        Ok(())
    }
}
pub(super) fn check(
    facts: &Declarations,
    body: &Body,
    expected: usize,
    work: &mut Work,
) -> Result<(), &'static str> {
    let definition = defining_owner(facts, body, work)?;
    let mut seen = HashSet::default();
    for statement in &body.statements {
        work.charge(1)?;
        let Action::Evaluate(id) = statement.action else {
            continue;
        };
        work.charge(1)?;
        let value = body
            .values
            .get(id.0 as usize)
            .ok_or("const invocation has invalid evaluated value")?;
        if !matches!(value.kind, Rvalue::Invoke { .. }) {
            continue;
        }
        // Every insertion is preceded by a charged visit, bounding set growth.
        work.charge(1)?;
        if !seen.insert(id) {
            return Err("const invocation value is evaluated at multiple source sites");
        }
        check_invocation(facts, body, definition, statement, work)?;
    }
    if seen.len() != expected {
        return Err("const invocation has no exact evaluated source site");
    }
    Ok(())
}
fn defining_owner(
    facts: &Declarations,
    body: &Body,
    work: &mut Work,
) -> Result<DefId, &'static str> {
    work.charge(1)?;
    let Owner::Function { definition, .. } = body.owner else {
        return Err("const call operators require their actual source function owner");
    };
    work.charge(2)?;
    let function = facts
        .function_indices
        .get(&definition)
        .and_then(|&index| facts.functions.get(index))
        .ok_or("const invocation has no actual source function")?;
    work.charge(1)?;
    if function.body != body.source.source() || !facts.is_const_function(definition) {
        return Err("const invocation belongs to another defining source body");
    }
    work.charge(3)?;
    crate::sema::compiler_items::callable_role(facts, CallTrait::FnOnce)?;
    Ok(definition)
}
fn check_invocation(
    facts: &Declarations,
    body: &Body,
    definition: DefId,
    statement: &crate::body::ir::Statement,
    work: &mut Work,
) -> Result<(), &'static str> {
    let Action::Evaluate(id) = statement.action else {
        return Err("const invocation has no evaluated statement");
    };
    work.charge(1)?;
    let value = body
        .values
        .get(id.0 as usize)
        .ok_or("const invocation has invalid evaluated value")?;
    let Rvalue::Invoke {
        kind,
        signature,
        arguments,
    } = value.kind
    else {
        return Err("const invocation changed its operation");
    };
    work.charge(1)?;
    let source = body
        .source
        .proof_site(statement.source)?
        .source()
        .ok_or("imported const invocation facts are not implemented")?;
    work.charge(1)?;
    let retained = facts
        .invocations
        .get(&source)
        .ok_or("const invocation has no retained source fact")?;
    if kind != CallTrait::FnOnce
        || retained.kind != Some(kind)
        || retained.borrow.is_some()
        || retained.dereferences != 0
    {
        return Err("const invocation differs from its defining call-trait or effect fact");
    }
    work.charge(1)?;
    let Some(Operation::Call {
        signature: original,
        source_use,
    }) = facts.node(source).and_then(|node| node.operation)
    else {
        return Err("const invocation has no retained typed Call operation");
    };
    let owner = body
        .source
        .source()
        .ok_or("imported const invocation owners are not implemented")?;
    work.charge(1)?;
    let effect = source_use
        .ok_or("const invocation has no retained source-use fact")?
        .effect_for(facts, owner)?;
    if effect == Constness::Ordinary {
        return Err("const invocation has no defining conditional or const demand");
    }
    work.charge(2)?;
    if original != signature || !simple_signature(&facts.types, signature, value.ty) {
        return Err("const invocation changed its retained closed signature or result");
    }
    work.charge(1)?;
    let arguments = body
        .arguments
        .get(arguments.start as usize..arguments.end as usize)
        .ok_or("invalid const invocation arguments")?;
    let [receiver, scalar] = arguments else {
        return Err("const invocation requires one owned receiver and one u8 argument");
    };
    work.charge(1)?;
    let actual = body
        .values
        .get(receiver.0 as usize)
        .ok_or("invalid const invocation receiver")?
        .ty;
    work.charge(1)?;
    if body
        .values
        .get(scalar.0 as usize)
        .is_none_or(|v| v.ty != value.ty)
        || receiver_type(facts, body, definition, *retained, work)? != actual
    {
        return Err("const invocation changed its exact instantiated receiver or argument");
    }
    Ok(())
}
fn simple_signature(store: &crate::types::Store, signature: TypeId, result: TypeId) -> bool {
    let u8 = TypeId::primitive(crate::types::Primitive::U8);
    matches!(store.kind(signature), Some(Kind::Function { parameters, result: output, bound_regions: 0,
        unsafe_: false, convention: crate::types::callable::Convention::RUST })
        if output == result && result == u8 && store.list(parameters) == Some(&[u8][..]))
}
fn receiver_type(
    facts: &Declarations,
    body: &Body,
    owner: DefId,
    retained: Invocation,
    work: &mut Work,
) -> Result<TypeId, &'static str> {
    work.charge(1)?;
    let Some(Kind::Parameter(parameter)) = facts.types.kind(retained.receiver) else {
        return Err("const invocation currently requires a direct generic receiver parameter");
    };
    work.charge(2)?;
    let parameter = facts
        .types
        .parameter(parameter)
        .ok_or("const invocation has invalid receiver parameter")?;
    let parameters = facts
        .generics
        .parameters
        .get(&owner)
        .ok_or("const invocation has no actual generic row")?;
    work.charge(1)?;
    if parameter.owner != crate::types::ItemId(owner.0)
        || parameter.index != 0
        || !matches!(parameter.kind, ParameterKind::Type { .. })
        || parameters.len() != 1
        || parameters[0].argument != Argument::Type(retained.receiver)
        || facts.associated.owner(owner).is_some()
    {
        return Err(
            "const invocation inherited, lifetime or composite receivers are not implemented",
        );
    }
    let Some(instance) = body.instance else {
        return Ok(retained.receiver);
    };
    work.charge(1)?;
    let Some(Kind::FunctionItem {
        item,
        arguments,
        signature,
    }) = facts.types.kind(instance.ty())
    else {
        return Err("const invocation lost its complete function instance");
    };
    if item.0 != owner.0 || body.owner.signatures().map(|(public, _)| public) != Some(signature) {
        return Err("const invocation belongs to another full callable instance");
    }
    work.charge(1)?;
    let mut arguments = facts
        .types
        .arguments(arguments)
        .ok_or("invalid const invocation instance arguments")?;
    let Some(Argument::Type(actual)) = arguments.next() else {
        return Err("const invocation lacks its exact receiver argument");
    };
    work.charge(3)?;
    if arguments.next().is_some()
        || !matches!(
            facts.types.kind(actual),
            Some(Kind::FunctionItem { .. } | Kind::Function { .. })
        )
        || facts.types.needs_region_context(actual) != Some(false)
        || facts.types.contains_parameters(actual) != Some(false)
    {
        return Err("const invocation requires a complete closed function or pointer receiver");
    }
    Ok(actual)
}

#[cfg(test)]
#[path = "../../../tests/invariants/invocation_facts.rs"]
mod tests;
