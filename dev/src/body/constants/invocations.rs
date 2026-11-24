//! Defining const permission consumes actual typed source facts, not signatures alone.
mod fold;
mod receivers;
use crate::{
    body::ir::{Action, Body, Owner, Rvalue},
    resolve::model::DefId,
    sema::model::{Declarations, Operation},
    support::{call_traits::CallTrait, constness::Constness, hash::HashSet},
    types::Kind,
};
pub(super) struct Work {
    pub steps: usize,
    limit: usize,
}
impl Work {
    pub(super) const fn new(limit: usize) -> Self {
        Self { steps: 0, limit }
    }
    pub(super) fn charge(&mut self, amount: usize) -> Result<(), &'static str> {
        self.steps = self.steps.saturating_add(amount);
        if self.steps > self.limit {
            return Err("const invocation validation work budget exceeded");
        }
        Ok(())
    }
}
#[cfg(test)]
pub(super) fn check(
    facts: &Declarations,
    body: &Body,
    expected: usize,
    work: &mut Work,
) -> Result<(), &'static str> {
    check_calls(None, facts, body, (expected, 0), work)
}
pub(super) fn check_calls(
    report: Option<&crate::resolve::expansion::Report>,
    facts: &Declarations,
    body: &Body,
    expected: (usize, usize),
    work: &mut Work,
) -> Result<(), &'static str> {
    let mut receivers = None;
    let mut provided = ProvidedCalls::default();
    let mut seen = HashSet::default();
    let mut members = 0;
    let defining_default = super::provided_calls::defining(facts, body, work)?;
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
        if expected.1 != 0 && matches!(value.kind, Rvalue::Call { .. }) {
            work.charge(7)?;
            if matches!(
                super::function_call(facts, body, value.kind),
                super::CallPermission::Member
            ) {
                work.charge(1)?;
                if !seen.insert(id) {
                    return Err("const member value is evaluated at multiple source sites");
                }
                provided.check(report, facts, body, statement, defining_default, work)?;
                members += 1;
                continue;
            }
        }
        if !matches!(value.kind, Rvalue::Invoke { .. }) {
            continue;
        }
        // Every insertion is preceded by a charged visit, bounding set growth.
        work.charge(1)?;
        if !seen.insert(id) {
            return Err("const invocation value is evaluated at multiple source sites");
        }
        if body.source.source().is_none() && fold::check(report, facts, body, statement, work)? {
            continue;
        }
        if super::qualified_invocations::check(report, facts, body, statement, work)? {
            continue;
        }
        if receivers.is_none() {
            work.charge(3)?;
            crate::sema::compiler_items::callable_role(facts, CallTrait::FnOnce)?;
            receivers = Some(if matches!(body.owner, Owner::Closure { .. }) {
                receivers::Bindings::closure(report, facts, body, work)?
            } else {
                let definition = defining_owner(facts, body, work)?;
                receivers::Bindings::new(facts, body, definition, work)?
            });
        }
        check_invocation(
            report,
            facts,
            body,
            receivers
                .as_ref()
                .ok_or("const invocation lost its defining function")?,
            statement,
            work,
        )?;
    }
    if members != expected.1 || seen.len() != expected.0.saturating_add(expected.1) {
        return Err("const invocation has no exact evaluated source site");
    }
    Ok(())
}
#[derive(Default)]
struct ProvidedCalls<'a> {
    local: Option<crate::sema::function_context::DefiningCalls<'a>>,
    imported: Option<super::provided_calls::pinned_defaults::Calls>,
}
impl<'a> ProvidedCalls<'a> {
    fn check(
        &mut self,
        report: Option<&crate::resolve::expansion::Report>,
        facts: &'a Declarations,
        body: &Body,
        statement: &crate::body::ir::Statement,
        defining_default: bool,
        work: &mut Work,
    ) -> Result<(), &'static str> {
        use super::provided_calls;
        if defining_default && body.source.source().is_none() {
            if self.imported.is_none() {
                self.imported = Some(provided_calls::pinned_defaults::Calls::new(
                    facts, body, work,
                )?);
            }
            return self
                .imported
                .as_ref()
                .ok_or("missing imported defining member context")?
                .check(facts, body, statement, work);
        }
        if defining_default && self.local.is_none() {
            let owner = defining_owner(facts, body, work)?;
            self.local = Some(provided_calls::prepare(facts, body, owner, work)?);
        }
        // Authenticate the completed source call, including its generic
        // arguments. Dispatch still proves the executable implementation.
        if let Some(report) = report
            && provided_calls::source_request_in_scope(
                report,
                facts,
                body,
                statement,
                self.local.as_ref(),
                work,
            )?
        {
            return Ok(());
        }
        if self.local.is_none() {
            let owner = defining_owner(facts, body, work)?;
            self.local = Some(provided_calls::prepare(facts, body, owner, work)?);
        }
        provided_calls::check(
            self.local
                .as_ref()
                .ok_or("missing defining member context")?,
            facts,
            body,
            statement,
            work,
        )
    }
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
    let source = body
        .source
        .source()
        .ok_or("imported const invocation owners are not implemented")?;
    if function.body != Some(source) {
        return Err("const invocation belongs to another defining source body");
    }
    // The defining-context facade performs at most two indexed function and
    // three indexed associated-owner accesses, including failed lookups.
    work.charge(5)?;
    if !facts.function_context(definition, source)?.checks_const() {
        return Err("const invocation belongs to another defining source body");
    }
    Ok(definition)
}
/// Defining permission authenticates this actual closure, without granting Code.
pub(super) fn closure_source(
    report: Option<&crate::resolve::expansion::Report>,
    facts: &Declarations,
    body: &Body,
    work: &mut Work,
) -> Result<crate::sema::closures::Body, &'static str> {
    let crate::body::ir::Owner::Closure { identity, .. } = body.owner else {
        return Err("const captured invocation requires its closure owner");
    };
    work.charge(8)?;
    body.owner
        .source_caller(facts, body.source, body.instance)?;
    let source = *facts
        .closures
        .identity(identity)
        .ok_or("const captured invocation lost its source closure")?;
    if !source
        .check_const_source(report.ok_or("const captured invocation requires source syntax")?)?
        || facts
            .types
            .closure_declaration(identity)
            .and_then(crate::types::closure::Declaration::constant)
            != Some(true)
        || !body.const_context
        || (body.instance.is_none()
            && body.owner.signatures()
                != Some((
                    source.signature,
                    source.execution.map_or(source.signature, |e| e.signature),
                )))
    {
        return Err("const captured invocation changed its authenticated closure contract");
    }
    Ok(source)
}
fn check_invocation(
    report: Option<&crate::resolve::expansion::Report>,
    facts: &Declarations,
    body: &Body,
    receivers: &receivers::Bindings,
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
    let original_receiver = source_receiver(
        report,
        facts,
        body,
        statement,
        (kind, signature, value.ty),
        work,
    )?;
    let Some(Kind::Function {
        parameters,
        result,
        bound_regions: 0,
        unsafe_: false,
        convention: crate::types::callable::Convention::RUST,
    }) = facts.types.kind(signature)
    else {
        return Err("const invocation requires its opened safe source signature");
    };
    work.charge(1)?;
    let parameters = facts
        .types
        .list(parameters)
        .ok_or("invalid const invocation parameters")?;
    work.charge(1)?;
    let arguments = body
        .arguments
        .get(arguments.start as usize..arguments.end as usize)
        .ok_or("invalid const invocation arguments")?;
    let [receiver, operands @ ..] = arguments else {
        return Err("const invocation requires its owned receiver");
    };
    if result != value.ty || operands.len() != parameters.len() {
        return Err("const invocation changed its exact argument or result arity");
    }
    work.charge(1)?;
    let actual = body
        .values
        .get(receiver.0 as usize)
        .ok_or("invalid const invocation receiver")?
        .ty;
    if receivers.receiver(facts, original_receiver, actual, signature, work)? != actual {
        return Err("const invocation changed its exact instantiated receiver or argument");
    }
    for (&operand, &parameter) in operands.iter().zip(parameters) {
        work.charge(1)?;
        if body
            .values
            .get(operand.0 as usize)
            .is_none_or(|v| v.ty != parameter)
        {
            return Err("const invocation argument differs from its exact signature");
        }
    }
    Ok(())
}

/// Source-use authority is checked independently of instantiated operand shape.
fn source_receiver(
    report: Option<&crate::resolve::expansion::Report>,
    facts: &Declarations,
    body: &Body,
    statement: &crate::body::ir::Statement,
    call: (CallTrait, crate::types::TypeId, crate::types::TypeId),
    work: &mut Work,
) -> Result<crate::types::TypeId, &'static str> {
    let (kind, signature, result) = call;
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
    let borrow_matches = match (kind, retained.borrow) {
        (CallTrait::FnOnce, None) => true,
        (CallTrait::Fn | CallTrait::FnMut, Some(borrow)) => {
            work.charge(2)?;
            borrow == retained.receiver
                && matches!(facts.types.kind(borrow), Some(Kind::Reference { mutable, .. })
                    if mutable == (kind == CallTrait::FnMut))
        }
        _ => false,
    };
    if retained.kind != Some(kind) || !borrow_matches {
        return Err("const invocation differs from its defining call-trait or effect fact");
    }
    if retained.dereferences != 0 {
        receivers::source_adjustment(report, facts, source, *retained, work)?;
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
    work.charge(3)?;
    if !matches!(
        facts.types.kind(original),
        Some(Kind::Function {
            bound_regions: 0,
            unsafe_: false,
            convention: crate::types::callable::Convention::RUST,
            ..
        })
    ) || (body.instance.is_none()
        && (original != signature || facts.node(source).and_then(|n| n.ty) != Some(result)))
    {
        return Err("const invocation changed its retained closed signature or result");
    }
    Ok(retained.receiver)
}

#[cfg(test)]
#[path = "../../../tests/invariants/invocation_facts.rs"]
mod tests;

/// Explicit language calls retain rust-call only for their actual validated
/// Fn member. Other members keep the original safe Rust convention boundary.
pub(super) fn member_signature(
    facts: &Declarations,
    target: crate::types::TypeId,
    signature: crate::types::TypeId,
    opened: bool,
    work: &mut Work,
) -> Result<bool, &'static str> {
    work.charge(3)?;
    let Some(Kind::Function {
        bound_regions,
        unsafe_: false,
        convention,
        ..
    }) = facts.types.kind(signature)
    else {
        return Ok(false);
    };
    if opened && bound_regions != 0 {
        return Ok(false);
    }
    if convention == crate::types::callable::Convention::RUST {
        return Ok(true);
    }
    if convention != crate::types::callable::Convention::RUST_CALL {
        return Ok(false);
    }
    work.charge(4)?;
    let Some(Kind::FunctionItem { item, .. }) = facts.types.kind(target) else {
        return Ok(false);
    };
    Ok(crate::sema::compiler_items::callable_member(facts, DefId(item.0))?.is_some())
}
