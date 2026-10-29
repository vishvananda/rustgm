//! Version-five exact definitions retain real body owners without granting execution.
use super::super::{DefinitionKind, FunctionBody, HeaderRef, Type};
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
        let declaration = &rows.declarations[row.declaration.0 as usize];
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
            || header.body != FunctionBody::Original(row.body)
            || headers.scopes().complete_slots(header.scope, work)? != 0
        {
            return Err("exact callable requires its actual closed module function/original");
        }
        let body = rows
            .body_owners
            .get(row.body.0 as usize)
            .ok_or("exact callable lost its actual body owner")?;
        if body.declaration != row.declaration
            || body.kind != BodyKind::Function
            || body.parent.is_some()
            || body.ordinal != 0
            || rows
                .originals
                .get(row.body.0 as usize)
                .is_none_or(|original| {
                    original.body != row.body || original.declaration != row.declaration
                })
        {
            return Err("exact callable changed its actual original owner");
        }
        for signature in [header.original, header.callable] {
            work.charge(
                signature
                    .parameters
                    .end
                    .saturating_sub(signature.parameters.start) as usize
                    + 2,
            )?;
            if signature.convention != Convention::C
                || signature.variadic
                || signature.unsafe_
                || rows
                    .binders
                    .get(signature.binder.0 as usize)
                    .is_none_or(|binder| binder.count != 0)
                || signature
                    .parameters
                    .slice(&rows.roots)?
                    .iter()
                    .chain(std::iter::once(&signature.result))
                    .any(|ty| !matches!(rows.types.get(ty.0 as usize), Some(Type::Scalar(_))))
            {
                return Err("exact callable requires its actual fixed primitive C signature");
            }
        }
    }
    Ok(())
}
