//! Lexical ownership is opened once; generic inheritance stays in its own ledger.
use super::{DefId, Error, ModuleId, Namespace, Resolver, ScopeId, Work, error, kind, s};

pub(super) fn owners(
    resolver: &mut Resolver<'_>,
    root: ScopeId,
    root_index: usize,
    rows: &s::Columns,
    work: &mut Work,
) -> Result<Vec<ScopeId>, Error> {
    owners_mode(resolver, root, root_index, rows, work, false)
}
pub(super) fn namespace_owners(
    resolver: &mut Resolver<'_>,
    root: ScopeId,
    root_index: usize,
    rows: &s::Columns,
    work: &mut Work,
) -> Result<Vec<ScopeId>, Error> {
    owners_mode(resolver, root, root_index, rows, work, true)
}
fn owners_mode(
    resolver: &mut Resolver<'_>,
    root: ScopeId,
    root_index: usize,
    rows: &s::Columns,
    work: &mut Work,
    modules: bool,
) -> Result<Vec<ScopeId>, Error> {
    let mut scopes = vec![None; rows.declarations.len()];
    let mut visiting = vec![false; rows.declarations.len()];
    let mut path = Vec::with_capacity(rows.declarations.len());
    scopes[root_index] = Some(root);
    for index in 0..rows.declarations.len() {
        work.charge(1)?;
        if scopes[index].is_some() {
            continue;
        }
        let mut next = index;
        while scopes[next].is_none() {
            work.charge(1)?;
            if std::mem::replace(&mut visiting[next], true) {
                return Err(error("source owner scope has a lexical cycle"));
            }
            path.push(next);
            next = rows.declarations[next]
                .definition
                .parent
                .map(|parent| parent.0 as usize)
                .filter(|&parent| parent < rows.declarations.len())
                .ok_or_else(|| error("source owner scope has no authenticated parent"))?;
        }
        while let Some(index) = path.pop() {
            work.charge(1)?;
            let row = &rows.declarations[index];
            let parent = row
                .definition
                .parent
                .ok_or_else(|| error("source owner scope lost its parent"))?;
            let parent = scopes[parent.0 as usize]
                .ok_or_else(|| error("source owner scope is incomplete"))?;
            let scope = if modules && row.definition.kind == s::DefinitionKind::Module
                || matches!(
                    row.definition.kind,
                    s::DefinitionKind::Trait
                        | s::DefinitionKind::SelfType
                        | s::DefinitionKind::Function
                ) {
                let scope = resolver.new_scope(Some(parent), ModuleId(0), true)?;
                resolver.facts.scopes[scope.0 as usize].associated = matches!(
                    row.definition.kind,
                    s::DefinitionKind::Trait | s::DefinitionKind::SelfType
                );
                scope
            } else {
                parent
            };
            scopes[index] = Some(scope);
            visiting[index] = false;
        }
    }
    scopes
        .into_iter()
        .map(|scope| scope.ok_or_else(|| error("source owner scope coverage is incomplete")))
        .collect()
}
pub(super) fn bodies(
    resolver: &mut Resolver<'_>,
    rows: &s::Columns,
    owners: &[ScopeId],
    body_count: usize,
    work: &mut Work,
) -> Result<Vec<Vec<ScopeId>>, Error> {
    let mut result: Vec<Option<Vec<ScopeId>>> = (0..body_count).map(|_| None).collect();
    for original in &rows.originals {
        work.charge(1)?;
        let owner = *owners
            .get(original.declaration.0 as usize)
            .ok_or_else(|| error("source body has no actual declaration scope"))?;
        let mut scopes = Vec::with_capacity(original.scopes.len());
        for scope in &original.scopes {
            work.charge(1)?;
            let parent = scope.parent.map_or(Ok(owner), |parent| {
                scopes
                    .get(parent as usize)
                    .copied()
                    .ok_or_else(|| error("source body lexical scope parent is unknown or forward"))
            })?;
            scopes.push(resolver.new_scope(Some(parent), ModuleId(0), false)?);
        }
        let slot = result
            .get_mut(original.body.0 as usize)
            .ok_or_else(|| error("source original scope has an unknown actual body"))?;
        if slot.replace(scopes).is_some() {
            return Err(error("source original scope was mapped twice"));
        }
    }
    result
        .into_iter()
        .map(|row| row.ok_or_else(|| error("source original scope coverage is incomplete")))
        .collect()
}
pub(super) fn assign(
    resolver: &mut Resolver<'_>,
    rows: &s::Columns,
    definitions: &[DefId],
    owners: &[ScopeId],
    body_scopes: &[Vec<ScopeId>],
    work: &mut Work,
) -> Result<(), Error> {
    assign_mode(
        resolver,
        rows,
        definitions,
        owners,
        body_scopes,
        work,
        false,
    )
}
pub(super) fn namespace_assign(
    resolver: &mut Resolver<'_>,
    rows: &s::Columns,
    definitions: &[DefId],
    owners: &[ScopeId],
    body_scopes: &[Vec<ScopeId>],
    work: &mut Work,
) -> Result<(), Error> {
    assign_mode(resolver, rows, definitions, owners, body_scopes, work, true)
}
fn assign_mode(
    resolver: &mut Resolver<'_>,
    rows: &s::Columns,
    definitions: &[DefId],
    owners: &[ScopeId],
    body_scopes: &[Vec<ScopeId>],
    work: &mut Work,
    modules: bool,
) -> Result<(), Error> {
    let mut ordinals = vec![0_u32; rows.declarations.len()];
    for (index, row) in rows.declarations.iter().enumerate() {
        work.charge(1)?;
        let Some(parent) = row.definition.parent else {
            continue;
        };
        if ordinals[parent.0 as usize] != row.ordinal {
            return Err(error(
                "source declaration changed its exact sibling ordinal",
            ));
        }
        ordinals[parent.0 as usize] = row
            .ordinal
            .checked_add(1)
            .ok_or_else(|| error("source declaration sibling ordinal overflow"))?;
        let scope = if let Some(local) = row.local {
            *body_scopes
                .get(local.body.0 as usize)
                .and_then(|scopes| scopes.get(local.scope as usize))
                .ok_or_else(|| error("source local has no exact original-owned scope"))?
        } else {
            owners[parent.0 as usize]
        };
        let actual = resolver
            .facts
            .definitions
            .get_mut(definitions[index].0 as usize)
            .ok_or_else(|| error("source declaration reservation is absent"))?;
        let expected = kind(row.definition.kind);
        if actual.node.is_some() || (actual.kind, actual.namespace) != expected {
            return Err(error(
                "source declaration reservation changed its actual kind or namespace",
            ));
        }
        actual.scope = scope;
        actual.members = if modules && row.definition.kind == s::DefinitionKind::Module
            || matches!(
                row.definition.kind,
                s::DefinitionKind::Trait | s::DefinitionKind::SelfType
            ) {
            Some(owners[index])
        } else {
            None
        };
        if row.definition.kind == s::DefinitionKind::RegionParameter
            && actual.namespace != Namespace::Lifetime
        {
            return Err(error("source lifetime declaration changed its namespace"));
        }
    }
    Ok(())
}
