//! Actual v9 selected operands retain full goals in their exact original scope.
use super::{
    Body, Declarations, ImportedOriginals, Index, Kind, Pending, Rvalue, SourceOriginal, s,
};
use crate::{support::constness::Constness, types::ItemId};

pub(super) fn prepare(
    index: &Index<'_>,
    view: &SourceOriginal<'_>,
    facts: &Declarations,
    body: &Body,
    work: &mut ImportedOriginals,
) -> Result<Vec<Pending>, &'static str> {
    require_original(view, facts, body, work)?;
    work.charge(body.values.len())?;
    // Aggregate original preflight includes this selected, flat value-domain
    // vector. It doubles as the O(1) call-operand association, never a new map.
    let mut pending = Vec::new();
    for (position, (raw, value)) in view.raw().values.iter().zip(&body.values).enumerate() {
        let s::ValueKind::Function(application) = raw.kind else {
            continue;
        };
        let target = index.rows.function_applications[application.0 as usize]
            .declaration
            .declaration;
        let Some(member) = index.members.get(&target) else {
            continue;
        };
        if pending.is_empty() {
            work.charge(body.values.len().saturating_add(view.raw().calls.len()))?;
            pending.resize(body.values.len(), Pending::Unused);
        }
        work.charge(6)?;
        let Some(s::HeaderRef::Function(header)) = index.rows.header(target)? else {
            return Err("applied source member lost its actual function header");
        };
        let definition = view.declaration(target, &mut work.steps)?;
        if index.rows.functions[header as usize].parent != Some(member.owner)
            || !matches!(value.kind, Rvalue::Function(actual) if actual == definition)
            || !matches!(facts.types.kind(value.ty), Some(Kind::FunctionItem { item, .. }) if item == ItemId(definition.0))
        {
            return Err("applied source member changed its actual owner or complete function item");
        }
        pending[position] = Pending::Applied {
            member: view.declaration(member.declaration, &mut work.steps)?,
            selected: value.ty,
            recorded: None,
        };
    }
    if pending.is_empty() {
        return Ok(pending);
    }
    for request in &view.raw().calls {
        work.charge(8)?;
        view.site(request.site, &mut work.steps)?;
        let call = body
            .values
            .get(request.value as usize)
            .ok_or("applied source call is outside its actual body")?;
        let Rvalue::Call {
            signature, callee, ..
        } = call.kind
        else {
            return Err("applied source call changed its actual operation kind");
        };
        let Some(Pending::Applied {
            member,
            selected,
            recorded,
        }) = pending.get_mut(callee.0 as usize)
        else {
            continue;
        };
        let actual = body
            .values
            .get(callee.0 as usize)
            .ok_or("applied source call lost its actual operand")?;
        if signature != view.ty(request.signature, &mut work.steps)?
            || actual.ty != *selected
            || !matches!(actual.kind, Rvalue::Function(definition) if definition == view.function(request.target, &mut work.steps)?)
        {
            return Err(
                "applied source call changed its exact selected target or opened signature",
            );
        }
        let application = request
            .method
            .ok_or("applied source member call lost its full method goal")?;
        let goal = view.predicate(
            &s::Requirement::Trait {
                application,
                effect: Constness::Ordinary,
            },
            &mut work.steps,
        )?;
        if recorded.is_some_and(|previous| previous != goal)
            || facts.associated.trait_owner(*member).is_none()
        {
            return Err("applied source operand has conflicting goals or a foreign trait member");
        }
        *recorded = Some(goal);
    }
    Ok(pending)
}

fn require_original(
    view: &SourceOriginal<'_>,
    facts: &Declarations,
    body: &Body,
    work: &mut ImportedOriginals,
) -> Result<(), &'static str> {
    work.charge(4)?;
    view.require_store(&facts.types)?;
    if body.source != view.source()?
        || view.raw().values.len() != body.values.len()
        || !matches!(body.owner, crate::body::ir::Owner::Function { definition, signature, execution: None }
            if definition == view.definition() && signature == view.signature())
    {
        return Err(
            "applied source selection changed its actual original body, owner or signature",
        );
    }
    Ok(())
}
