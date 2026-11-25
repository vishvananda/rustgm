//! Reprove the canonical conditional Self request in an imported default.
use super::{Body, Declarations, DefId, Kind, Owner, Rvalue, Statement, Work};
use crate::{
    body::ir::{Action, BodyOrigin},
    support::crates::generics::source::{CallContext, EnvironmentOrigin},
    types::parameters::{clauses::Clause, predicates::Predicate},
};

pub(super) fn defining(
    facts: &Declarations,
    body: &Body,
    work: &mut Work,
) -> Result<bool, &'static str> {
    let Owner::Function {
        definition,
        signature,
        execution,
    } = body.owner
    else {
        return Err("imported provided body lost its callable owner");
    };
    work.charge(5)?;
    let header = facts
        .declaration_header(definition)?
        .ok_or("imported provided body lost its canonical declaration")?;
    if header.body != Some(body.source) {
        return Err("imported provided body changed its actual conditional owner");
    }
    if facts.nonconst_trait_member(definition)? {
        if body.const_context {
            return Err("non-const trait methods cannot be called in const contexts");
        }
        return Ok(false);
    }
    if facts
        .imported_conditional_member_trait(definition)?
        .is_none()
    {
        return Err("imported provided body changed its actual conditional owner");
    }
    if let Some(instance) = body.instance {
        if execution.is_none()
            || body.const_context && !facts.imported_const_default(definition)?
            || !matches!(
                facts.types.kind(instance.ty()), Some(Kind::FunctionItem { item, signature: actual, .. })
                if item.0 == definition.0 && actual == signature
            )
        {
            return Err("imported provided body changed its actual runtime instance");
        }
        return Ok(false);
    }
    if execution.is_some() || !body.const_context || header.signature != signature {
        return Err("imported provided body changed its exact original signature or effect");
    }
    Ok(true)
}

pub(in crate::body::constants) struct Calls {
    owner: DefId,
    premise: Predicate,
    body: BodyOrigin,
}
impl Calls {
    pub(in crate::body::constants) fn new(
        facts: &Declarations,
        body: &Body,
        work: &mut Work,
    ) -> Result<Self, &'static str> {
        let Owner::Function { definition, .. } = body.owner else {
            return Err("imported provided call has no defining function");
        };
        let (owner, premise) = facts
            .imported_default_premise(definition)?
            .ok_or("imported provided call lost its exact conditional Self premise")?;
        let Clause::Predicate(predicate) = premise else {
            return Err("imported provided call has an invalid Self predicate");
        };
        let view = crate::sema::crates::source_original(facts, definition, &mut work.steps)?
            .ok_or("imported provided call lost its actual defining original")?;
        work.charge(4)?;
        if view.source()? != body.source
            || view.raw().context != CallContext::ConstDefinition
            || !matches!(
                view.inputs().environment,
                EnvironmentOrigin::ConstTraitDefault { .. }
            )
        {
            return Err("imported provided call changed its original or defining context");
        }
        let mut clauses = view.environment()?.trait_clauses()?;
        loop {
            work.charge(1)?;
            let Some(clause) = clauses.next() else {
                return Err("imported provided call lacks its paid conditional Self premise");
            };
            if clause == premise {
                return Ok(Self {
                    owner,
                    premise: predicate,
                    body: body.source,
                });
            }
        }
    }

    pub(in crate::body::constants) fn check(
        &self,
        facts: &Declarations,
        body: &Body,
        statement: &Statement,
        work: &mut Work,
    ) -> Result<(), &'static str> {
        work.charge(5)?;
        let Action::Evaluate(id) = statement.action else {
            return Err("imported provided call has no evaluated operation");
        };
        let Some(Rvalue::Call { callee, .. }) =
            body.values.get(id.0 as usize).map(|value| value.kind)
        else {
            return Err("imported provided call changed its actual Call operation");
        };
        let target = body
            .values
            .get(callee.0 as usize)
            .ok_or("imported provided call lost its actual typed target")?;
        let Some(Kind::FunctionItem {
            item, arguments, ..
        }) = facts.types.kind(target.ty)
        else {
            return Err("imported provided call requires its actual direct member");
        };
        let definition = DefId(item.0);
        body.source.proof_site(statement.source)?;
        // The sealed body maps the exact request target, signature and effect
        // to this operation. Its conditional Self goal is proved here against
        // this consumer's own defining environment before instance dispatch.
        if body.source != self.body
            || !matches!(target.kind, Rvalue::Function(actual) if actual == definition)
            || facts.imported_conditional_member_trait(definition)? != Some(self.owner)
            || facts.conditional_member_request(definition, arguments, &mut |steps| {
                work.charge(steps)
            })? != self.premise
        {
            return Err("imported provided call changed its actual member or conditional Self");
        }
        Ok(())
    }
}

fn check_instance_owner(facts: &Declarations, body: &Body) -> Result<(), &'static str> {
    match body.owner {
        Owner::Function {
            definition,
            signature,
            execution,
        } => {
            let header = facts
                .declaration_header(definition)?
                .ok_or("imported provided instance lost its canonical declaration")?;
            let matched = match body.instance {
                Some(instance) => {
                    execution.is_some()
                        && matches!(
                            facts.types.kind(instance.ty()),
                            Some(Kind::FunctionItem { item, signature: actual, .. })
                                if item.0 == definition.0 && actual == signature
                        )
                }
                None => execution.is_none() && header.signature == signature,
            };
            if !body.const_context
                || header.body != Some(body.source)
                || !(facts.is_const_function(definition)
                    || facts.imported_const_default(definition)?)
                || !matched
            {
                return Err("imported provided instance changed its authenticated const default");
            }
        }
        Owner::Initializer {
            definition,
            kind: crate::sema::model::InitializerKind::Const,
        }
        | Owner::InitializerInstance {
            instance: crate::sema::values::Instance { definition, .. },
            kind: crate::sema::model::InitializerKind::Const,
        } => {
            let origin = facts
                .crates
                .as_deref()
                .ok_or("imported provided constant lost its crate linkage")?
                .constant_initializer(facts, definition)?;
            if !body.const_context || origin != Some(body.source) {
                return Err(
                    "imported provided constant changed its authenticated original initializer",
                );
            }
        }
        _ => return Err("imported provided instance lost its callable owner"),
    }
    Ok(())
}

/// An imported const function or const default may call a conditional const
/// trait member; runtime execution and CTFE still re-prove the selected impl.
pub(in crate::body::constants) fn check_instance(
    facts: &Declarations,
    body: &Body,
    statement: &Statement,
    work: &mut Work,
) -> Result<(), &'static str> {
    work.charge(8)?;
    check_instance_owner(facts, body)?;
    let Action::Evaluate(id) = statement.action else {
        return Err("imported provided call has no evaluated operation");
    };
    let value = body
        .values
        .get(id.0 as usize)
        .ok_or("imported provided call lost its evaluated value")?;
    let Rvalue::Call {
        callee,
        signature: call_signature,
        arguments,
    } = value.kind
    else {
        return Err("imported provided call changed its actual Call operation");
    };
    let target = body
        .values
        .get(callee.0 as usize)
        .ok_or("imported provided call lost its actual typed target")?;
    let Some(Kind::FunctionItem { item, .. }) = facts.types.kind(target.ty) else {
        return Err("imported provided call requires its actual direct member");
    };
    let target_definition = DefId(item.0);
    if !matches!(target.kind, Rvalue::Function(actual) if actual == target_definition)
        || facts.nonconst_trait_member(target_definition)?
        || facts
            .imported_conditional_member_trait(target_definition)?
            .is_none()
    {
        return Err("imported provided instance call requires an actual const default target");
    }
    let Some(Kind::Function {
        bound_regions: 0,
        convention: crate::types::callable::Convention::RUST,
        parameters,
        result,
        ..
    }) = facts.types.kind(call_signature)
    else {
        return Err("imported provided call lost its opened safe Rust signature");
    };
    let parameters = facts
        .types
        .list(parameters)
        .ok_or("imported provided call lost its parameter list")?;
    let arguments = body
        .arguments
        .get(arguments.start as usize..arguments.end as usize)
        .ok_or("imported provided call lost its argument range")?;
    if result != value.ty || parameters.len() != arguments.len() {
        return Err("imported provided call changed its result or argument count");
    }
    body.source.proof_site(statement.source)?;
    Ok(())
}
