//! Raw operation validation is independent of later region/ownership/loan proof.
use super::{
    Action, Declaration, End, Function, Lifetime, LocalKind, Location, Original, Rows, ScopeKind,
    Terminator, TypeRef, ValueKind, Work, graph::Graph,
};
use crate::support::crates::MAX_COMPONENTS;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Relation {
    pub actual: TypeRef,
    pub expected: TypeRef,
    pub site: u32,
}
/// The importer consumes this once while constructing the original Contract.
/// The rows cannot omit a relation needed by one of these supported operations.
pub(super) fn relations(original: &Original) -> impl Iterator<Item = Relation> + '_ {
    let constraints = original.values.iter().filter_map(|value| match value.kind {
        ValueKind::Constrain(input) => Some(Relation {
            actual: original.values[input as usize].ty,
            expected: value.ty,
            site: 0,
        }),
        _ => None,
    });
    constraints.chain(
        original
            .blocks
            .iter()
            .filter_map(|block| match block.terminator {
                Terminator::Return(value) => Some(Relation {
                    actual: original.values[value as usize].ty,
                    expected: original.result,
                    site: block.site,
                }),
                _ => None,
            }),
    )
}
pub(super) fn validate(rows: &Rows, graph: &Graph, work: &mut Work) -> Result<(), &'static str> {
    let mut definitions = vec![false; rows.declarations.len()];
    let mut applications = vec![false; rows.calls.as_ref().map_or(0, |c| c.applications.len())];
    let mut site_count = 0_usize;
    for (index, original) in rows.originals.iter().enumerate() {
        work.charge(1)?;
        let header = &rows.functions[index];
        if original.body.0 as usize != index
            || original.body != header.body
            || original.declaration != header.declaration
        {
            return Err("generic original changed its exact header or body identity");
        }
        site_count = site_count.saturating_add(original.sites as usize);
        if original.sites == 0 || site_count > MAX_COMPONENTS {
            return Err("generic body operation sites exceed profile budget");
        }
        scopes(original, work)?;
        locals(rows, graph, header, original, &mut definitions, work)?;
        bindings(rows, original, work)?;
        operations(rows, graph, header, original, &mut applications, work)?;
        storage(original, work)?;
    }
    if applications.contains(&false) {
        return Err("generic application has no actual function-item operand");
    }
    for (index, declaration) in rows.declarations.iter().enumerate() {
        work.charge(1)?;
        if matches!(
            declaration,
            Declaration::Local { .. } | Declaration::ValueParameter { .. }
        ) && !definitions[index]
        {
            return Err("generic local declaration has no exact binding association");
        }
    }
    Ok(())
}
fn site(original: &Original, site: u32) -> Result<(), &'static str> {
    if site >= original.sites {
        return Err("generic operation site belongs outside its body");
    }
    Ok(())
}
fn scopes(original: &Original, work: &mut Work) -> Result<(), &'static str> {
    if original.scopes.is_empty() {
        return Err("generic body lacks its real root storage scope");
    }
    for (index, scope) in original.scopes.iter().enumerate() {
        work.charge(1)?;
        site(original, scope.site)?;
        if index == 0 {
            if scope.parent.is_some() || scope.kind != ScopeKind::Body {
                return Err("generic root storage scope changed");
            }
        } else if scope.parent.is_none_or(|parent| parent as usize >= index)
            || scope.kind == ScopeKind::Body
        {
            return Err("generic storage scopes are foreign or cyclic");
        }
    }
    Ok(())
}
fn locals(
    rows: &Rows,
    graph: &Graph,
    header: &Function,
    original: &Original,
    definitions: &mut [bool],
    work: &mut Work,
) -> Result<(), &'static str> {
    let signature = Graph::signature(rows, header.original)?;
    let parameters = signature.parameters.slice(&rows.roots)?;
    let mut inputs = vec![false; parameters.len()];
    for (index, local) in original.locals.iter().enumerate() {
        work.charge(1)?;
        site(original, local.site)?;
        graph.root(header.declaration, local.ty, work)?;
        if local.scope as usize >= original.scopes.len()
            || local
                .storage_parent
                .is_some_and(|parent| parent as usize >= index)
        {
            return Err("generic local changed its storage scope or predecessor");
        }
        if let LocalKind::Parameter(ordinal) = local.kind {
            if parameters.get(ordinal as usize) != Some(&local.ty) || inputs[ordinal as usize] {
                return Err("generic input storage differs from the actual callable parameter");
            }
            inputs[ordinal as usize] = true;
        }
        if let Some(definition) = local.definition {
            let valid = match rows.declarations.get(definition.0 as usize) {
                Some(Declaration::ValueParameter { body, ordinal, .. }) => {
                    *body == original.body && parameters.get(*ordinal as usize) == Some(&local.ty)
                }
                Some(Declaration::Local { body, ordinal, .. }) => {
                    *body == original.body && *ordinal as usize == index
                }
                _ => false,
            };
            if !valid || definitions[definition.0 as usize] {
                return Err("generic binding belongs to a foreign owner or local slot");
            }
            definitions[definition.0 as usize] = true;
        }
    }
    if inputs.contains(&false) {
        return Err("generic original omitted an incoming parameter slot");
    }
    graph.root(header.declaration, original.result, work)?;
    if original.result != signature.result {
        return Err("generic original result differs from its header");
    }
    Ok(())
}
fn operations(
    rows: &Rows,
    graph: &Graph,
    header: &Function,
    original: &Original,
    applications: &mut [bool],
    work: &mut Work,
) -> Result<(), &'static str> {
    super::operands::validate(rows, graph, header, original, applications, work)?;
    for statement in &original.statements {
        work.operations = work.operations.saturating_add(1);
        work.charge(1)?;
        site(original, statement.site)?;
        let local = |index: u32| {
            original
                .locals
                .get(index as usize)
                .ok_or("generic statement local is unknown")
        };
        let value = |index: u32| {
            original
                .values
                .get(index as usize)
                .ok_or("generic statement value is unknown")
        };
        match statement.action {
            Action::StorageLive(index)
            | Action::StorageDead(index)
            | Action::CheckInitialized(index)
            | Action::Mention(index) => {
                local(index)?;
            }
            Action::StorageEnd { first, last } => {
                local(first)?;
                local(last)?;
                if first > last {
                    return Err("generic storage suffix is reversed");
                }
            }
            Action::Evaluate(index) => {
                value(index)?;
            }
            Action::Store {
                local: index,
                value: input,
            } => {
                if local(index)?.ty != value(input)?.ty {
                    return Err("generic store type differs from its actual place");
                }
            }
        }
    }
    let mut covered = vec![false; original.statements.len()];
    if original.blocks.is_empty() {
        return Err("generic original has no entry block");
    }
    for block in &original.blocks {
        work.operations = work.operations.saturating_add(1);
        work.charge(1)?;
        site(original, block.site)?;
        block.statements.slice(&original.statements)?;
        for index in block.statements.start..block.statements.end {
            work.charge(1)?;
            if std::mem::replace(&mut covered[index as usize], true) {
                return Err("generic block statement ranges overlap");
            }
        }
        match block.terminator {
            Terminator::Return(value) if value as usize >= original.values.len() => {
                return Err("generic return value is unknown");
            }
            Terminator::Jump(target) if target as usize >= original.blocks.len() => {
                return Err("generic successor belongs to another body");
            }
            _ => {}
        }
    }
    if covered.contains(&false) {
        return Err("generic statements have no actual block owner");
    }
    Ok(())
}
fn storage(original: &Original, work: &mut Work) -> Result<(), &'static str> {
    let at = |block: u32, before: u32| {
        original
            .blocks
            .get(block as usize)
            .is_some_and(|block| before >= block.statements.start && before <= block.statements.end)
    };
    for exit in &original.exits {
        work.charge(1)?;
        match exit.location {
            Location::At { block, before } if at(block, before) => {}
            Location::Edge { block, target }
                if original
                    .blocks
                    .get(block as usize)
                    .is_some_and(|block| block.terminator == Terminator::Jump(target)) => {}
            _ => return Err("generic storage exit has a foreign location or successor"),
        }
        match exit.end {
            End::Scopes { inner, outer }
                if inner as usize >= original.scopes.len()
                    || outer.is_some_and(|outer| outer >= inner) =>
            {
                return Err("generic storage exit names foreign scopes");
            }
            End::Temporaries(range) => {
                range.slice(&original.temporaries)?;
            }
            End::Scopes { .. } => {}
        }
    }
    for temporary in &original.temporaries {
        work.charge(1)?;
        site(original, temporary.site)?;
        if temporary.value as usize >= original.values.len()
            || temporary.exits_before as usize > original.exits.len()
            || !at(temporary.block, temporary.before)
            || matches!(temporary.lifetime,Lifetime::Scope(scope) if scope as usize >= original.scopes.len())
        {
            return Err("generic temporary changed its value, scope or materialization point");
        }
    }
    // Pipeline ingestion must additionally call the existing validating storage
    // index before builder-trust paths can consume these rows.
    Ok(())
}

fn bindings(rows: &Rows, original: &Original, work: &mut Work) -> Result<(), &'static str> {
    let mut origins = Vec::with_capacity(original.values.len());
    for value in &original.values {
        work.charge(1)?;
        let origin = match value.kind {
            ValueKind::Read(local) => Some(local),
            ValueKind::Constrain(input) => *origins
                .get(input as usize)
                .ok_or("parameter binding has a cyclic value input")?,
            ValueKind::Unit | ValueKind::Function(_) | ValueKind::Call { .. } => None,
        };
        origins.push(origin);
    }
    let mut stores = vec![None; original.locals.len()];
    for statement in &original.statements {
        work.charge(1)?;
        if let Action::Store { local, value } = statement.action {
            let slot = stores
                .get_mut(local as usize)
                .ok_or("parameter binding store has no local")?;
            if slot.is_none() {
                *slot = Some(value);
            }
        }
    }
    let mut seen = vec![false; original.locals.len()];
    for binding in &original.parameter_bindings {
        work.charge(1)?;
        let Some(Declaration::ValueParameter { body, ordinal, .. }) =
            rows.declarations.get(binding.declaration.0 as usize)
        else {
            return Err("parameter binding names another declaration kind");
        };
        let input = original
            .locals
            .get(binding.input as usize)
            .ok_or("parameter binding input is unknown")?;
        let local = original
            .locals
            .get(binding.local as usize)
            .ok_or("parameter binding local is unknown")?;
        if *body != original.body
            || input.kind != LocalKind::Parameter(*ordinal)
            || local.definition != Some(binding.declaration)
            || local.ty != input.ty
            || std::mem::replace(&mut seen[binding.local as usize], true)
        {
            return Err("parameter binding changed its exact formal, owner or local association");
        }
        if binding.input != binding.local {
            let value = stores[binding.local as usize]
                .ok_or("named parameter has no actual initializing store")?;
            if origins.get(value as usize) != Some(&Some(binding.input)) {
                return Err("named parameter initializer comes from another actual input");
            }
        }
    }
    for (index, local) in original.locals.iter().enumerate() {
        work.charge(1)?;
        if local.definition.is_some_and(|definition| {
            matches!(
                rows.declarations.get(definition.0 as usize),
                Some(Declaration::ValueParameter { .. })
            )
        }) && !seen[index]
        {
            return Err("named parameter has no complete formal association");
        }
    }
    Ok(())
}
