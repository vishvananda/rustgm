//! Direct const closure uses retain the actual call site and defining source contract.
use super::invocations::Work;
use crate::{
    body::ir::{Action, Body, Rvalue, Statement},
    sema::model::{Declarations, Operation},
    support::{call_traits::CallTrait, constness::Constness},
    types::Kind,
};

pub(super) fn check(
    report: Option<&crate::resolve::expansion::Report>,
    facts: &Declarations,
    body: &Body,
    statement: &Statement,
    work: &mut Work,
) -> Result<bool, &'static str> {
    let Action::Evaluate(id) = statement.action else {
        return Ok(false);
    };
    work.charge(3)?;
    let value = body
        .values
        .get(id.0 as usize)
        .ok_or("const closure operation is missing")?;
    let Rvalue::Invoke {
        kind,
        signature,
        arguments,
    } = value.kind
    else {
        return Ok(false);
    };
    let operands = body
        .arguments
        .get(arguments.start as usize..arguments.end as usize)
        .ok_or("const closure operand range is invalid")?;
    let Some((&receiver, operands)) = operands.split_first() else {
        return Err("const closure call has no receiver");
    };
    let actual = body
        .values
        .get(receiver.0 as usize)
        .ok_or("const closure receiver is missing")?
        .ty;
    let subject = if kind == CallTrait::FnOnce {
        actual
    } else {
        let Some(Kind::Reference {
            referent, mutable, ..
        }) = facts.types.kind(actual)
        else {
            return Ok(false);
        };
        if matches!(facts.types.kind(referent), Some(Kind::Closure { .. }))
            && mutable != (kind == CallTrait::FnMut)
        {
            return Err("const closure receiver has the wrong borrow capability");
        }
        referent
    };
    let Some(Kind::Closure {
        identity,
        signature: public,
        ..
    }) = facts.types.kind(subject)
    else {
        return Ok(false);
    };
    work.charge(12 + operands.len())?;
    crate::sema::compiler_items::callable_role(facts, kind)?;
    let _ = source(
        facts,
        crate::types::instance::Instance::closure(&facts.types, subject)?,
    )?;
    if let Some(report) = report
        && !facts.closures.check_const_source(report, identity)?
    {
        return Err("non-const closures cannot be called in a const context");
    }
    if !facts
        .types
        .closure_kind(identity)
        .is_some_and(|permission| {
            permission.implements(crate::types::closure::CallKind::from_trait(kind))
        })
    {
        return Err("const closure call exceeds its actual capture permission");
    }
    public_signature(facts, body, value, public, operands)?;
    source_site(facts, body, statement, (kind, signature, identity), work)?;
    Ok(true)
}

fn public_signature(
    facts: &Declarations,
    body: &Body,
    value: &crate::body::ir::Value,
    public: crate::types::TypeId,
    operands: &[crate::body::ir::ValueId],
) -> Result<(), &'static str> {
    let Rvalue::Invoke { signature, .. } = value.kind else {
        return Err("const closure call lost its actual Invoke signature");
    };
    let Some(Kind::Function {
        parameters,
        result,
        bound_regions: 0,
        unsafe_: false,
        convention,
    }) = facts.types.kind(signature)
    else {
        return Err("const closure call requires its opened safe signature");
    };
    if public != signature
        || convention != crate::types::callable::Convention::RUST
        || result != value.ty
    {
        return Err("const closure call changed its actual public signature or output");
    }
    let parameters = facts
        .types
        .list(parameters)
        .ok_or("const closure parameters are invalid")?;
    if parameters.len() != operands.len()
        || parameters.iter().zip(operands).any(|(&ty, id)| {
            body.values
                .get(id.0 as usize)
                .is_none_or(|value| value.ty != ty)
        })
    {
        return Err("const closure call changed its complete argument list");
    }
    Ok(())
}

pub(crate) fn source(
    facts: &Declarations,
    instance: crate::types::instance::Instance,
) -> Result<crate::sema::closures::Body, &'static str> {
    let Some(Kind::Closure { identity, .. }) = facts.types.kind(instance.ty()) else {
        return Err("const closure requires its actual complete application");
    };
    let body = *facts
        .closures
        .identity(identity)
        .ok_or("const closure source is not complete")?;
    if facts
        .types
        .closure_declaration(identity)
        .and_then(crate::types::closure::Declaration::constant)
        != Some(true)
        || !facts.closures.is_const(identity)
    {
        return Err("non-const closures cannot be called in a const context");
    }
    if scoped(facts, body.owner)?
        && !(facts
            .closures
            .origin(identity)
            .is_some_and(|origin| origin.source().is_none())
            && facts.types.contains_parameters(instance.ty()) == Some(false)
            && facts.types.contains_projections(instance.ty()) == Some(false))
    {
        return Err("const closure defining obligations require a scoped continuation");
    }
    Ok(body)
}

/// Imported closed instances can proceed to the ordinary const selection when
/// reached. The execution path still proves their defining obligations.
pub(crate) fn scoped(
    facts: &Declarations,
    owner: Option<crate::resolve::model::DefId>,
) -> Result<bool, &'static str> {
    let Some(owner) = owner else {
        return Ok(false);
    };
    if facts.generics.parameters_in_scope(owner).next().is_some()
        || facts.imported_const_default(owner)?
    {
        return Ok(true);
    }
    if facts.function_indices.contains_key(&owner) || facts.constant_functions.contains(&owner) {
        let catalog = facts
            .source_bindings
            .as_deref()
            .ok_or("const closure source catalog is absent")?;
        return catalog.requires_const_payment(owner);
    }
    Ok(false)
}

fn source_site(
    facts: &Declarations,
    body: &Body,
    statement: &Statement,
    selected: (
        CallTrait,
        crate::types::TypeId,
        crate::types::closure::ClosureId,
    ),
    work: &mut Work,
) -> Result<(), &'static str> {
    let (kind, signature, identity) = selected;
    work.charge(8)?;
    if body.source.source().is_none() {
        return imported_site(facts, body, statement, work);
    }
    body.owner
        .source_caller(facts, body.source, body.instance)?;
    let owner = body
        .source
        .source()
        .ok_or("imported const closure callers are not implemented")?;
    let site = body
        .source
        .proof_site(statement.source)?
        .source()
        .ok_or("const closure call requires its actual source site")?;
    let retained = facts
        .invocations
        .get(&site)
        .ok_or("const closure lost its typed source invocation")?;
    let Some(Operation::Call {
        signature: original,
        source_use,
    }) = facts.node(site).and_then(|n| n.operation)
    else {
        return Err("const closure call has no typed source operation");
    };
    if retained.kind != Some(kind)
        || retained.dereferences != 0
        || source_use
            .ok_or("const closure call has no source-use context")?
            .effect_for(facts, owner)?
            == Constness::Ordinary
        || (body.instance.is_none() && original != signature)
    {
        return Err("const closure call differs from its actual source use or signature");
    }
    let original = if kind == CallTrait::FnOnce {
        retained.receiver
    } else {
        let Some(Kind::Reference {
            referent, mutable, ..
        }) = facts.types.kind(retained.receiver)
        else {
            return Err("const closure source receiver lost its actual borrow");
        };
        if mutable != (kind == CallTrait::FnMut) {
            return Err("const closure source borrow capability changed");
        }
        referent
    };
    if !matches!(facts.types.kind(original), Some(Kind::Closure { identity: actual, .. }) if actual == identity)
    {
        return Err("const closure call belongs to another source closure identity");
    }
    Ok(())
}

fn imported_site(
    facts: &Declarations,
    body: &Body,
    statement: &Statement,
    work: &mut Work,
) -> Result<(), &'static str> {
    use crate::{body::ir::Owner, support::crates::generics::source::CallContext};
    if !body.const_context {
        return Err("imported const closure call requires its actual const body context");
    }
    body.source.proof_site(statement.source)?;
    let view = match body.owner {
        Owner::Function { definition, .. } => {
            crate::sema::crates::source_original(facts, definition, &mut work.steps)?
                .ok_or("imported const closure caller lost its actual function original")?
        }
        Owner::Closure { identity, .. } => {
            if !facts.closures.is_const(identity) {
                return Err("non-const imported closure cannot invoke const closures");
            }
            crate::sema::crates::source_closure_original(facts, identity, &mut work.steps)?
                .ok_or("imported const closure caller lost its actual closure original")?
        }
        Owner::AnonymousConstant => {
            let crate::resolve::body_origin::BodyOrigin::Imported(origin) = body.source else {
                return Err("imported const closure caller lost its anonymous body origin");
            };
            crate::sema::crates::source_anonymous_original(facts, origin.id(), &mut work.steps)?
                .ok_or("imported const closure caller lost its actual anonymous original")?
        }
        _ => {
            return Err(
                "imported const closure caller requires a function, closure or anonymous owner",
            );
        }
    };
    work.charge(2)?;
    if !view.rows().descriptor().profile().declaration_policies()
        || view.source()? != body.source
        || !matches!(
            view.raw().context,
            CallContext::ConstDefinition | CallContext::ClosedConst
        )
    {
        return Err("imported const closure caller changed its defining body or effect");
    }
    Ok(())
}
