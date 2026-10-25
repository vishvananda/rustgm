//! Exact lexical owners and diagnostic partitions are built once per artifact.
use super::{
    BodyBinding, DeclarationBinding, DefId, DefKind, Error, Expected, Imported, Interface,
    ModuleId, Resolver, ScopeId, Work, error, g,
};
use crate::resolve::body_origin::Label;

pub(super) fn scopes(
    resolver: &mut Resolver<'_>,
    body: &g::Original,
    owner: ScopeId,
    work: &mut Work,
) -> Result<Vec<ScopeId>, Error> {
    let mut scopes = Vec::with_capacity(body.scopes.len());
    for scope in &body.scopes {
        work.charge(1)?;
        let parent = scope
            .parent
            .map_or(Some(owner), |p| scopes.get(p as usize).copied());
        scopes.push(resolver.new_scope(parent, ModuleId(0), false)?);
    }
    Ok(scopes)
}
pub(super) fn declarations(
    rows: &g::Rows,
    scopes: &[Vec<ScopeId>],
    work: &mut Work,
) -> Result<Vec<Option<ScopeId>>, Error> {
    let mut result = vec![None; rows.declarations.len()];
    for (ordinal, body) in rows.originals.iter().enumerate() {
        for local in &body.locals {
            work.charge(1)?;
            let Some(declaration) = local.definition else {
                continue;
            };
            let expected_body = match rows.declarations.get(declaration.0 as usize) {
                Some(
                    g::Declaration::ValueParameter { body, .. }
                    | g::Declaration::Local { body, .. },
                ) => *body,
                _ => return Err(error("imported binding is not an actual local declaration")),
            };
            let scope = scopes
                .get(ordinal)
                .and_then(|s| s.get(local.scope as usize))
                .copied()
                .ok_or_else(|| error("imported local scope is unknown"))?;
            if expected_body.0 as usize != ordinal
                || result[declaration.0 as usize].replace(scope).is_some()
            {
                return Err(error("imported local declaration changed its exact owner"));
            }
        }
    }
    Ok(result)
}
pub(super) fn authenticate(
    resolver: &Resolver<'_>,
    artifact: &Interface,
    imported: &Imported,
    definitions: &[DefId],
    expected: &[Expected],
    work: &mut Work,
) -> Result<(), Error> {
    let rows = artifact
        .generic_payload()
        .ok_or_else(|| error("missing authenticated original rows"))?
        .legacy_rows()
        .map_err(error)?;
    if rows.declarations.len() != definitions.len() || expected.len() != definitions.len() {
        return Err(error("imported original remap is incomplete"));
    }
    for ((&id, expected), row) in definitions.iter().zip(expected).zip(&rows.declarations) {
        work.charge(1)?;
        let actual = resolver
            .facts
            .definitions
            .get(id.0 as usize)
            .ok_or_else(|| error("imported original remap is unknown"))?;
        if actual.node.is_some()
            || actual.kind != expected.kind
            || actual.scope != expected.scope
            || actual.namespace != expected.namespace
            || actual.name != expected.name
        {
            return Err(error(
                "imported declaration changed its actual owner, name or kind",
            ));
        }
        if let g::Declaration::Function { identity, public } = row
            && (imported.definition(identity) != Some(id)
                || actual.kind != DefKind::Function
                || actual.access
                    != if *public {
                        crate::resolve::model::Access::Public
                    } else {
                        crate::resolve::model::Access::ExternalPrivate
                    })
        {
            return Err(error(
                "imported function remap belongs to another external identity",
            ));
        }
    }
    Ok(())
}
pub(super) fn declaration_sites(
    rows: &g::Rows,
    work: &mut Work,
) -> Result<(Vec<DeclarationBinding>, Vec<Option<Label>>), Error> {
    let mut bindings = Vec::with_capacity(rows.declarations.len());
    let mut labels = Vec::new();
    for (declaration, &sites) in rows.declarations.iter().zip(&rows.declaration_sites) {
        work.charge(sites as usize)?;
        let first = u32::try_from(labels.len())
            .map_err(|_| error("too many imported declaration sites"))?;
        let end = first
            .checked_add(sites)
            .ok_or_else(|| error("imported declaration site overflow"))?;
        labels.resize(end as usize, None);
        let kind = match declaration {
            g::Declaration::Function { .. } => DefKind::Function,
            g::Declaration::TypeParameter { .. } => DefKind::Generic,
            g::Declaration::ValueParameter { .. } => DefKind::Parameter,
            g::Declaration::Local { .. } => DefKind::Local,
        };
        bindings.push(DeclarationBinding {
            kind,
            sites: first..end,
        });
    }
    Ok((bindings, labels))
}
pub(super) fn body_sites(
    rows: &g::Rows,
    work: &mut Work,
) -> Result<(Vec<BodyBinding>, Vec<Option<Label>>), Error> {
    let mut bindings = Vec::with_capacity(rows.originals.len());
    let mut labels = Vec::new();
    for body in &rows.originals {
        work.charge(body.sites as usize)?;
        let first =
            u32::try_from(labels.len()).map_err(|_| error("too many imported operation sites"))?;
        let end = first
            .checked_add(body.sites)
            .ok_or_else(|| error("imported operation site overflow"))?;
        labels.resize(end as usize, None);
        bindings.push(BodyBinding {
            unit: body.body,
            sites: first..end,
        });
    }
    Ok((bindings, labels))
}
