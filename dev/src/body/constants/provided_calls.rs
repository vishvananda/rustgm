//! Convert an exact original IR statement into a neutral defining-call query.
use super::invocations::Work;
use crate::{
    body::ir::{Action, Body, Owner, Rvalue, Statement},
    resolve::model::DefId,
    sema::{
        function_context::{Call, DefiningCalls, FunctionContext},
        model::Declarations,
    },
    types::Kind,
};

/// Only the actual freshly materialized body may consume a const-default
/// selection. Original and runtime templates retain their defining context.
pub(crate) fn prepare_default_execution(
    report: &crate::resolve::expansion::Report,
    facts: &mut Declarations,
    execution: &crate::sema::function_context::DefaultExecution,
    body: &mut Body,
) -> Result<(), &'static str> {
    let callable = match body.owner {
        Owner::Function {
            definition,
            signature,
            execution: Some(_),
        } => Some((definition, signature)),
        _ => None,
    };
    execution.require_materialization(
        report,
        facts,
        crate::sema::function_context::DefaultMaterialization {
            source: body.source,
            instance: body.instance,
            callable,
            const_context: body.const_context,
        },
    )?;
    body.const_context = true;
    Ok(())
}

/// One preparation is shared by the readonly and cumulative-budget entrypoints.
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
        return Ok(false);
    };
    // Only actual const-trait members enter defining preparation. Ordinary
    // source bodies need no body walk, context allocation or validation owner.
    let Some(id) = facts.associated.owner(definition) else {
        return Ok(false);
    };
    let Some(owner) = facts.associated.get(id) else {
        work.charge(1)?;
        return Err("provided defining function has an invalid associated owner");
    };
    if !matches!(
        owner.kind,
        crate::sema::associated::OwnerKind::Trait { constant: true, .. }
    ) {
        return Ok(false);
    }
    let source = body
        .source
        .source()
        .ok_or("imported provided-body defining checks are not implemented")?;
    work.charge(5)?;
    if !matches!(
        facts.function_context(definition, source)?,
        crate::sema::function_context::FunctionContext::ConstTraitDefault { .. }
    ) {
        return Ok(false);
    }
    if let Some(instance) = body.instance {
        work.charge(1)?;
        if execution.is_none()
            || !matches!(facts.types.kind(instance.ty()), Some(Kind::FunctionItem {
                item, signature: actual, ..
            }) if item.0 == definition.0 && actual == signature)
        {
            return Err("provided runtime body lost its actual materialized callable relation");
        }
        // Runtime instances still require the normal paired original/instance
        // proof. Reached CTFE additionally selects a concrete const impl;
        // direct root entry still cannot borrow that call's permission.
        return Ok(false);
    }
    work.charge(2)?;
    let header = facts
        .function_header(definition)?
        .ok_or("provided defining function header is pending")?;
    if execution.is_some() || header.signature != signature || header.body != Some(body.source) {
        return Err("provided defining body changed its exact original signature or source");
    }
    Ok(true)
}

pub(super) fn prepare<'a>(
    facts: &'a Declarations,
    body: &Body,
    definition: DefId,
    work: &mut Work,
) -> Result<DefiningCalls<'a>, &'static str> {
    if body.instance.is_some() {
        return Err("specialized abstract const-trait defining calls are not implemented");
    }
    let source = body
        .source
        .source()
        .ok_or("imported const member facts are not implemented")?;
    let Owner::Function {
        signature,
        execution: None,
        ..
    } = body.owner
    else {
        return Err("const member defining body changed its original public callable");
    };
    work.charge(2)?;
    let header = facts
        .function_header(definition)?
        .ok_or("const member defining body requires its completed function header")?;
    if header.signature != signature || header.body != Some(body.source) {
        return Err("const member defining body changed its exact original header");
    }
    DefiningCalls::new(facts, definition, source, |n| work.charge(n))
}

pub(super) fn check(
    context: &DefiningCalls<'_>,
    facts: &Declarations,
    body: &Body,
    statement: &Statement,
    work: &mut Work,
) -> Result<(), &'static str> {
    context.check(source_call(facts, body, statement, work)?, |n| {
        work.charge(n)
    })
}

fn source_call(
    facts: &Declarations,
    body: &Body,
    statement: &Statement,
    work: &mut Work,
) -> Result<Call, &'static str> {
    let Action::Evaluate(id) = statement.action else {
        return Err("const member requires an evaluated statement");
    };
    work.charge(1)?;
    let value = body
        .values
        .get(id.0 as usize)
        .ok_or("invalid const member value")?;
    let Rvalue::Call {
        callee, signature, ..
    } = value.kind
    else {
        return Err("const member changed its direct Call operation");
    };
    work.charge(1)?;
    let target = body
        .values
        .get(callee.0 as usize)
        .ok_or("invalid const member callee")?;
    work.charge(1)?;
    let Some(Kind::FunctionItem { item, .. }) = facts.types.kind(target.ty) else {
        return Err("const member callee lost its full actual function item");
    };
    if !matches!(target.kind, Rvalue::Function(definition) if definition.0 == item.0) {
        return Err("const member callee value changed its exact actual definition");
    }
    work.charge(1)?;
    let site = body
        .source
        .proof_site(statement.source)?
        .source()
        .ok_or("imported const member sites are not implemented")?;
    Ok(Call {
        site,
        target: target.ty,
        signature,
        result: value.ty,
    })
}

/// Defining legality only. A source-use location is not implementation proof;
/// actual dispatch, borrow proof and const Code entry remain independently required.
pub(super) fn closed_request(
    report: &crate::resolve::expansion::Report,
    facts: &Declarations,
    body: &Body,
    statement: &Statement,
    work: &mut Work,
) -> Result<bool, &'static str> {
    if !body.const_context {
        return Ok(false);
    }
    work.charge(9)?;
    if let Some(catalog) = facts.source_bindings.as_deref() {
        catalog.require_provider_report(report)?;
    } else if report.package.uses_source_bindings() {
        return Err("const member has no source catalog");
    } else {
        facts
            .require_lint_context(report)
            .map_err(|_| "const member changed its actual source report")?;
    }
    body.owner
        .source_caller(facts, body.source, body.instance)?;
    let source = body
        .source
        .source()
        .ok_or("const member has no source owner")?;
    let call = source_call(facts, body, statement, work)?;
    work.charge(1)?;
    if facts.types.contains_parameters(call.target) == Some(true) {
        // The actual defining environment, rather than closed-member dispatch,
        // must pay an abstract call's exact conditional trait premise.
        return Ok(false);
    }
    work.charge(5)?;
    let node = facts
        .node(call.site)
        .ok_or("const member lost its typed site")?;
    if let Some(crate::sema::model::Operation::Field { plan: Some(id), .. }) = node.operation {
        return closed_field(report, facts, body, source, call, id, work);
    }
    let Some(crate::sema::model::Operation::Call {
        signature,
        source_use,
    }) = node.operation
    else {
        return Err("const member lost its typed Call operation");
    };
    if signature != call.signature {
        return Err("const member changed its current signature or result");
    }
    let indexed = if node.ty == Some(call.result) {
        None
    } else {
        Some(index_result(report, facts, call, node.ty, work)?)
    };
    let effect = match (facts.source_bindings.as_deref(), source_use) {
        (_, Some(use_)) => use_.effect_for(facts, source)?,
        (None, None) => ordinary_effect(report, facts, body, source, work)?,
        _ => return Err("const member has no current source use"),
    };
    if !closed_effect(report, facts, body, source, effect, work)? {
        return Ok(false);
    }
    let definition = match indexed {
        Some(definition) => definition,
        None => closed_member(facts, call, work)?,
    };
    work.charge(2)?;
    if let Some(method) = facts.methods.calls.get(&call.site) {
        if method.method != definition || method.callable != call.target {
            return Err("const member changed its current selected method");
        }
    } else if let Some(invocation) = facts.invocations.get(&call.site) {
        if invocation.kind.is_some()
            || invocation.receiver != call.target
            || invocation.dereferences != 0
            || invocation.borrow.is_some()
        {
            return Err("const member changed its current selected UFCS callable");
        }
    } else {
        direct_source(report, facts, call, definition, work)?;
    }
    Ok(true)
}

/// A Field expression owns several actual calls, each with its real reference
/// result. Its final selected field type is not any call's result contract.
fn closed_field(
    report: &crate::resolve::expansion::Report,
    facts: &Declarations,
    body: &Body,
    source: crate::resolve::unit::NodeKey,
    call: Call,
    id: crate::sema::bodies::ProtocolPlanId,
    work: &mut Work,
) -> Result<bool, &'static str> {
    work.charge(8)?;
    let plan = id.current(facts, source, call.site)?;
    let crate::sema::protocol_plans::Kind::Field { steps, .. } = plan.kind else {
        return Err("const field call changed its actual protocol");
    };
    work.charge(
        steps
            .end
            .saturating_sub(steps.start)
            .checked_ilog2()
            .map_or(1, |n| n as usize + 2),
    )?;
    crate::sema::protocol_plans::access::lookup(
        report,
        facts,
        plan,
        source,
        crate::sema::iteration::Call {
            callable: call.target,
            signature: call.signature,
        },
    )
    .map_err(|_| "const field call lost its actual owner, source or complete call")?;
    if !matches!(facts.types.kind(call.signature), Some(Kind::Function { result, .. }) if result == call.result)
    {
        return Err("const field call changed its actual reference result");
    }
    closed_member(facts, call, work)?;
    let effect = plan.source_use.effect_for(facts, source)?;
    closed_effect(report, facts, body, source, effect, work)
}

/// A retained Index call produces a reference before its source expression forms
/// a place. This relation is separate from ordinary direct call results.
fn index_result(
    report: &crate::resolve::expansion::Report,
    facts: &Declarations,
    call: Call,
    node_type: Option<crate::types::TypeId>,
    work: &mut Work,
) -> Result<DefId, &'static str> {
    work.charge(15)?;
    let definition = closed_member(facts, call, work)?;
    let method = facts
        .methods
        .calls
        .get(&call.site)
        .ok_or("const index result lacks its actual selected method")?;
    if method.method != definition || method.callable != call.target || method.dereferences != 0 {
        return Err("const index result changed its actual selected callable");
    }
    let operand = report
        .package
        .units
        .get(call.site.file.0 as usize)
        .and_then(|unit| unit.configured.ast.nodes.get(call.site.node.get() as usize))
        .and_then(|node| match node.kind {
            crate::syntax::ast::Kind::Index { value, .. } => Some(value),
            _ => None,
        })
        .ok_or("const member changed its current signature or result")?;
    let owner = facts
        .associated
        .trait_owner(definition)
        .ok_or("const index method lacks its actual trait owner")?;
    let catalog = facts
        .source_bindings
        .as_deref()
        .ok_or("const index result lacks its actual source catalog")?;
    let mutable = if catalog
        .language(crate::support::bindings::LangItem::Index)
        .is_some_and(|d| d.definition == owner)
    {
        false
    } else if catalog
        .language(crate::support::bindings::LangItem::IndexMut)
        .is_some_and(|d| d.definition == owner)
    {
        true
    } else {
        return Err("const index result belongs to another actual language trait");
    };
    let Some(Kind::Function {
        parameters, result, ..
    }) = facts.types.kind(call.signature)
    else {
        return Err("const index result lost its actual call signature");
    };
    let Some([receiver, _]) = facts.types.list(parameters) else {
        return Err("const index result changed its actual parameter list");
    };
    if method.receiver != *receiver || result != call.result {
        return Err("const index result changed its actual receiver borrow or call result");
    }
    index_operand(facts, call.site, operand, method.borrow, mutable, work)?;
    let Some(Kind::Reference {
        mutable: receiver_mutable,
        region: receiver_region,
        ..
    }) = facts.types.kind(*receiver)
    else {
        return Err("const index result lost its borrowed receiver");
    };
    let Some(Kind::Reference {
        mutable: result_mutable,
        region: result_region,
        referent,
    }) = facts.types.kind(call.result)
    else {
        return Err("const index result lost its actual reference result");
    };
    if receiver_mutable != mutable
        || result_mutable != mutable
        || receiver_region != result_region
        || node_type != Some(referent)
    {
        return Err("const index result changed its checked mutability, lifetime or referent");
    }
    Ok(definition)
}

/// The source place is borrowed before its checked directed receiver conversion.
/// This defining check authenticates that place; it creates no region or const proof.
fn index_operand(
    facts: &Declarations,
    site: crate::resolve::unit::NodeKey,
    operand: crate::syntax::ast::NodeId,
    borrow: Option<crate::types::TypeId>,
    mutable: bool,
    work: &mut Work,
) -> Result<(), &'static str> {
    work.charge(3)?;
    let Some(Kind::Reference {
        mutable: actual_mutable,
        referent,
        ..
    }) = borrow.and_then(|ty| facts.types.kind(ty))
    else {
        return Err("const index result lost its actual operand borrow");
    };
    let mut actual = facts
        .node(crate::resolve::unit::NodeKey {
            node: operand,
            ..site
        })
        .and_then(|node| node.ty)
        .ok_or("const index result lost its actual operand type")?;
    for _ in 0..facts.autoderefs.get(&site).copied().unwrap_or(0) {
        work.charge(1)?;
        let Some(Kind::Reference { referent, .. }) = facts.types.kind(actual) else {
            return Err("const index result changed its actual operand dereferences");
        };
        actual = referent;
    }
    if actual_mutable != mutable || referent != actual {
        return Err("const index result changed its actual operand borrow");
    }
    Ok(())
}

fn ordinary_effect(
    report: &crate::resolve::expansion::Report,
    facts: &Declarations,
    body: &Body,
    source: crate::resolve::unit::NodeKey,
    work: &mut Work,
) -> Result<crate::support::constness::Constness, &'static str> {
    use crate::support::constness::Constness;
    work.charge(8)?;
    // source_caller has already authenticated this exact initializer/function.
    // An ordinary typed body has no SourceCore context descriptor to borrow.
    match body.owner {
        Owner::Initializer { .. }
        | Owner::InitializerInstance { .. }
        | Owner::AnonymousConstant
        | Owner::AnonymousInstance(_) => Ok(Constness::Const),
        Owner::Function { definition, .. } => {
            match facts.source_function_context(report, definition, source)? {
                FunctionContext::Comptime => Ok(Constness::Const),
                FunctionContext::ConstFunction | FunctionContext::ConstTraitDefault { .. } => {
                    Ok(Constness::Conditional)
                }
                FunctionContext::Runtime => Err("const member has no actual const source owner"),
            }
        }
        _ => Err("const member has no actual const source owner"),
    }
}

fn closed_effect(
    report: &crate::resolve::expansion::Report,
    facts: &Declarations,
    body: &Body,
    source: crate::resolve::unit::NodeKey,
    effect: crate::support::constness::Constness,
    work: &mut Work,
) -> Result<bool, &'static str> {
    use crate::support::constness::Constness;
    match effect {
        Constness::Const => Ok(true),
        Constness::Ordinary => Ok(false),
        Constness::Conditional => {
            let Owner::Function { definition, .. } = body.owner else {
                return Ok(false);
            };
            work.charge(16)?;
            Ok(
                match facts.source_function_context(report, definition, source)? {
                    FunctionContext::ConstFunction => true,
                    FunctionContext::ConstTraitDefault { .. } => body.instance.is_some(),
                    FunctionContext::Runtime | FunctionContext::Comptime => false,
                },
            )
        }
    }
}

fn direct_source(
    report: &crate::resolve::expansion::Report,
    facts: &Declarations,
    call: Call,
    definition: DefId,
    work: &mut Work,
) -> Result<(), &'static str> {
    work.charge(3)?;
    let crate::syntax::ast::Kind::Call { callee, .. } = report
        .package
        .unit(call.site.file)
        .configured
        .ast
        .node(call.site.node)
        .kind
    else {
        return Err("const member requires its actual direct source Call");
    };
    let target = facts
        .node(crate::resolve::unit::NodeKey {
            node: callee,
            ..call.site
        })
        .ok_or("const member lost its typed source callee")?;
    if target.ty != Some(call.target)
        || target.reference != Some(crate::resolve::model::Target::Definition(definition))
    {
        return Err("const member changed its exact direct source callable");
    }
    Ok(())
}

fn closed_member(facts: &Declarations, call: Call, work: &mut Work) -> Result<DefId, &'static str> {
    work.charge(4)?;
    let Some(Kind::FunctionItem {
        item, signature, ..
    }) = facts.types.kind(call.target)
    else {
        return Err("const member lost its actual function item");
    };
    let definition = DefId(item.0);
    if facts.types.contains_parameters(call.target) != Some(false)
        || facts.types.contains_projections(call.target) != Some(false)
        || !matches!(
            facts.types.kind(signature),
            Some(Kind::Function {
                unsafe_: false,
                convention: crate::types::callable::Convention::RUST,
                ..
            })
        )
    {
        return Err("const member requires its closed safe source signature");
    }
    work.charge(1)?;
    if !matches!(
        facts.types.kind(call.signature),
        Some(Kind::Function {
            bound_regions: 0,
            unsafe_: false,
            convention: crate::types::callable::Convention::RUST,
            ..
        })
    ) {
        return Err("const member requires its actual opened safe call signature");
    }
    work.charge(4)?;
    let header = facts
        .function_header(definition)?
        .ok_or("const member header is pending")?;
    work.charge(1)?;
    if facts.nonconst_trait_member(definition)? {
        return Err("non-const trait methods cannot be called in const contexts");
    }
    // A default body does not prevent a concrete impl from supplying its own
    // const override. This validates only the defining request. Ordinary
    // dispatch must select the actual impl and prove its body before Code;
    // direct root entry still cannot execute a trait default without a caller.
    if header.declaration.source().is_none()
        || facts
            .associated
            .trait_owner(definition)
            .is_none_or(|owner| facts.associated.constant_trait(owner) != Some(true))
    {
        return Err("const member requires its actual source const-trait declaration");
    }
    Ok(definition)
}

#[cfg(test)]
#[path = "../../../tests/invariants/defining_members.rs"]
mod tests;
