//! Version-five exact definitions retain real body owners without granting execution.
use super::super::{ArtifactRef, DefinitionKind, FunctionBody, HeaderRef, Profile, Type};
use super::{HeaderIndex, SourceRows};
use crate::support::crates::generics::{Convention, Work};
use crate::support::{
    crates::{MAX_BYTES, MAX_NAME_BYTES, owner_catalog::BodyKind},
    hash::HashSet,
    retained,
};

pub(super) fn legacy(rows: &SourceRows, work: &mut Work) -> Result<(), &'static str> {
    work.charge(1)?;
    if !rows.exact_callables.is_empty() {
        return Err("exact source callable definitions require explicit version five");
    }
    Ok(())
}
pub(super) fn validate(
    rows: &SourceRows,
    headers: &HeaderIndex<'_>,
    profile: Profile,
    work: &mut Work,
) -> Result<(), &'static str> {
    work.charge(rows.exact_callables.len().saturating_add(1))?;
    if headers
        .retained_bytes()
        .saturating_add(retained::new_map::<&str, ()>(rows.exact_callables.len()))
        > MAX_BYTES
    {
        return Err("exact callable validation exceeds combined storage");
    }
    let mut names = HashSet::with_capacity_and_hasher(
        rows.exact_callables.len(),
        crate::support::hash::BuildFxHasher::default(),
    );
    let mut runtime = runtime_roles(rows, profile, work)?;
    let mut previous = None;
    let mut bytes = 0_usize;
    for row in &rows.exact_callables {
        work.charge(row.symbol.len().saturating_add(12))?;
        bytes = bytes.saturating_add(row.symbol.len());
        if row.symbol.is_empty()
            || row.symbol.len() > 4096
            || row.symbol.contains('\0')
            || bytes > MAX_NAME_BYTES
        {
            return Err("invalid exact source symbol bytes");
        }
        if previous.is_some_and(|id| id >= row.declaration.0) || !names.insert(row.symbol.as_ref())
        {
            return Err("duplicate or reordered exact source callable identity/symbol");
        }
        previous = Some(row.declaration.0);
        let Some(HeaderRef::Function(index)) = headers.header(row.declaration, work)? else {
            return Err("exact callable does not own an actual function header");
        };
        let header = &rows.functions[index as usize];
        require_module(rows, headers, header, work)?;
        let imported = runtime.remove(&row.declaration);
        if imported {
            if row.body.is_some()
                || header.body != FunctionBody::Absent
                || header.constness != crate::support::constness::Constness::Ordinary
            {
                return Err("source runtime import cannot acquire an original body");
            }
            runtime_signatures(rows, header, work)?;
        } else {
            let body = row
                .body
                .ok_or("source exact import has no actual runtime role")?;
            original_owner(rows, headers, header, body)?;
            for signature in [header.original, header.callable] {
                signature_shape(rows, signature, false, work)?;
            }
        }
    }
    if !runtime.is_empty() {
        return Err("source runtime role has no actual exact import");
    }
    Ok(())
}

fn runtime_roles(
    rows: &SourceRows,
    profile: Profile,
    work: &mut Work,
) -> Result<HashSet<crate::support::crates::owner_catalog::DeclarationRef>, &'static str> {
    let mut result = HashSet::default();
    for role in &rows.roles {
        work.charge(1)?;
        if role.role.is_runtime_import()
            && (!profile.declaration_policies()
                || role.declaration.artifact != ArtifactRef(0)
                || result.len() >= 6
                || !result.insert(role.declaration.declaration))
        {
            return Err(
                "source foreign runtime role requires its own version-fourteen declaration",
            );
        }
    }
    Ok(result)
}

fn require_module(
    rows: &SourceRows,
    headers: &HeaderIndex<'_>,
    header: &super::super::FunctionHeader,
    work: &mut Work,
) -> Result<(), &'static str> {
    let declaration = &rows.declarations[header.declaration.0 as usize];
    let parent = declaration
        .definition
        .parent
        .ok_or("exact callable lacks its actual module owner")?;
    if rows
        .declarations
        .get(parent.0 as usize)
        .is_none_or(|d| d.definition.kind != DefinitionKind::Module)
        || header.parent.is_some()
        || header.receiver.is_some()
        || header.regions.total != 0
        || headers.scopes().complete_slots(header.scope, work)? != 0
    {
        return Err("exact callable requires its actual closed module function/original");
    }
    Ok(())
}

fn original_owner(
    rows: &SourceRows,
    headers: &HeaderIndex<'_>,
    header: &super::super::FunctionHeader,
    reference: crate::support::crates::owner_catalog::BodyRef,
) -> Result<(), &'static str> {
    let body = rows
        .body_owners
        .get(reference.0 as usize)
        .ok_or("exact callable lost its actual body owner")?;
    if header.body != FunctionBody::Original(reference)
        || body.declaration != header.declaration
        || body.kind != BodyKind::Function
        || body.parent.is_some()
        || body.ordinal != 0
        || rows
            .originals
            .get(headers.original_index(reference)?)
            .is_none_or(|original| {
                original.body != reference || original.declaration != header.declaration
            })
    {
        return Err("exact callable changed its actual original owner");
    }
    Ok(())
}

fn runtime_signatures(
    rows: &SourceRows,
    header: &super::super::FunctionHeader,
    work: &mut Work,
) -> Result<(), &'static str> {
    let (original, callable) = (header.original, header.callable);
    for signature in [original, callable] {
        signature_shape(rows, signature, true, work)?;
    }
    if original.unsafe_ != callable.unsafe_
        || original.result != callable.result
        || original.parameters.slice(&rows.roots)? != callable.parameters.slice(&rows.roots)?
    {
        return Err("source runtime import changed its actual foreign callable signature");
    }
    Ok(())
}

fn signature_shape(
    rows: &SourceRows,
    signature: crate::support::crates::generics::Signature,
    runtime: bool,
    work: &mut Work,
) -> Result<(), &'static str> {
    let parameters = signature.parameters.slice(&rows.roots)?;
    work.charge(parameters.len().saturating_add(2))?;
    if signature.convention != Convention::C
        || signature.variadic
        || !runtime && signature.unsafe_
        || rows
            .binders
            .get(signature.binder.0 as usize)
            .is_none_or(|binder| binder.count != 0)
        || parameters
            .iter()
            .chain(std::iter::once(&signature.result))
            .any(|ty| !match rows.types.get(ty.0 as usize) {
                Some(Type::Scalar(_)) => true,
                Some(Type::RawPointer { pointee, .. }) if runtime => matches!(
                    rows.types.get(pointee.0 as usize),
                    Some(Type::Nominal { .. } | Type::Scalar(_))
                ),
                _ => false,
            })
    {
        return Err(if runtime {
            "source runtime import requires its actual closed scalar/pointer C signature"
        } else {
            "exact callable requires its actual fixed primitive C signature"
        });
    }
    Ok(())
}
