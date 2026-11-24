//! Version-five exact definitions retain real body owners without granting execution.
use super::super::{
    Argument, ArtifactRef, DeclarationAddress, DefinitionKind, FunctionBody, HeaderRef, Profile,
    Region, Type,
};
use super::{HeaderIndex, SourceRows};
use crate::support::crates::generics::{Convention, Work};
use crate::support::{
    bindings::LangItem,
    crates::{
        MAX_BYTES, MAX_NAME_BYTES,
        owner_catalog::{BodyKind, DeclarationRef},
    },
    hash::{HashMap, HashSet},
    linkage::PANIC_HANDLER_SYMBOL,
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
        .saturating_add(retained::new_map::<&str, bool>(rows.exact_callables.len()))
        > MAX_BYTES
    {
        return Err("exact callable validation exceeds combined storage");
    }
    let mut names = HashMap::with_capacity_and_hasher(
        rows.exact_callables.len(),
        crate::support::hash::BuildFxHasher::default(),
    );
    let (mut runtime, panic_info) = import_roles(rows, profile, work)?;
    let mut previous = None;
    let mut seen_weak = false;
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
        if previous.is_some_and(|id| id >= row.declaration.0) {
            return Err("duplicate or reordered exact source callable identity/symbol");
        }
        if !row.weak_panic {
            let exported = row.body.is_some();
            let duplicate = match names.get(row.symbol.as_ref()).copied() {
                None => {
                    names.insert(row.symbol.as_ref(), exported);
                    false
                }
                Some(previous_export) => {
                    exported || previous_export || !profile.declaration_policies()
                }
            };
            if duplicate {
                return Err("duplicate or reordered exact source callable identity/symbol");
            }
        }
        previous = Some(row.declaration.0);
        let Some(HeaderRef::Function(index)) = headers.header(row.declaration, work)? else {
            return Err("exact callable does not own an actual function header");
        };
        let header = &rows.functions[index as usize];
        let imported = runtime.remove(&row.declaration);
        if row.weak_panic {
            if !profile.declaration_policies()
                || imported
                || names.contains_key(PANIC_HANDLER_SYMBOL)
            {
                return Err("weak panic import requires its own version-fourteen declaration");
            }
            weak_panic_import(rows, headers, header, row, panic_info, work)?;
            seen_weak = true;
            continue;
        }
        if row.symbol.as_ref() == PANIC_HANDLER_SYMBOL && seen_weak {
            return Err("weak panic symbol requires its authenticated source callable");
        }
        let private_import = profile.declaration_policies()
            && row.body.is_none()
            && !rows.declarations[row.declaration.0 as usize].public;
        require_module(rows, headers, header, private_import, work)?;
        if imported || private_import {
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

fn import_roles(
    rows: &SourceRows,
    profile: Profile,
    work: &mut Work,
) -> Result<(HashSet<DeclarationRef>, Option<DeclarationAddress>), &'static str> {
    let mut result = HashSet::default();
    let mut panic_info = None;
    for role in &rows.roles {
        work.charge(1)?;
        if role.role == LangItem::PanicInfo && role.declaration.artifact == ArtifactRef(0) {
            panic_info = Some(role.declaration);
        }
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
    Ok((result, panic_info))
}

fn weak_panic_import(
    rows: &SourceRows,
    headers: &HeaderIndex<'_>,
    header: &super::super::FunctionHeader,
    row: &super::super::ExactCallable,
    panic_info: Option<DeclarationAddress>,
    work: &mut Work,
) -> Result<(), &'static str> {
    let error = "weak panic import lacks its actual lexical Rust PanicInfo signature";
    let declaration = &rows.declarations[header.declaration.0 as usize];
    let parent = declaration.definition.parent.ok_or(error)?;
    let Some(HeaderRef::Function(parent)) = headers.header(parent, work)? else {
        return Err(error);
    };
    let panic_info = panic_info.ok_or(error)?;
    if row.symbol.as_ref() != PANIC_HANDLER_SYMBOL
        || declaration.public
        || row.body.is_some()
        || header.body != FunctionBody::Absent
        || header.parent.is_some()
        || header.receiver.is_some()
        || header.declared_inputs != 0
        || header.regions.captures != 0
        || header.constness != crate::support::constness::Constness::Ordinary
        || !matches!(
            rows.functions[parent as usize].body,
            FunctionBody::Original(_)
        )
    {
        return Err(error);
    }
    for signature in [header.original, header.callable] {
        work.charge(20)?;
        if !weak_panic_signature(rows, signature, panic_info)? {
            return Err(error);
        }
    }
    Ok(())
}

fn weak_panic_signature(
    rows: &SourceRows,
    signature: crate::support::crates::generics::Signature,
    panic_info: DeclarationAddress,
) -> Result<bool, &'static str> {
    let [parameter] = signature.parameters.slice(&rows.roots)? else {
        return Ok(false);
    };
    let Some(Type::Reference {
        mutable: false,
        region,
        referent,
    }) = rows.types.get(parameter.0 as usize)
    else {
        return Ok(false);
    };
    let Some(Type::Nominal {
        declaration,
        arguments,
    }) = rows.types.get(referent.0 as usize)
    else {
        return Ok(false);
    };
    let [Argument::Region(inner)] = arguments.slice(&rows.arguments)? else {
        return Ok(false);
    };
    let bound_slot = |region: super::super::RegionRef| match rows.regions.get(region.0 as usize) {
        Some(Region::Bound {
            binder,
            depth: 0,
            slot,
        }) if *binder == signature.binder => Some(*slot),
        _ => None,
    };
    Ok(signature.convention == Convention::Rust
        && signature.unsafe_
        && !signature.variadic
        && rows.types.get(signature.result.0 as usize) == Some(&Type::Never)
        && *declaration == panic_info
        && rows
            .binders
            .get(signature.binder.0 as usize)
            .is_some_and(|binder| binder.count == 2)
        && matches!(
            (bound_slot(*region), bound_slot(*inner)),
            (Some(0), Some(1)) | (Some(1), Some(0))
        ))
}

fn require_module(
    rows: &SourceRows,
    headers: &HeaderIndex<'_>,
    header: &super::super::FunctionHeader,
    allow_function_parent: bool,
    work: &mut Work,
) -> Result<(), &'static str> {
    let declaration = &rows.declarations[header.declaration.0 as usize];
    let parent = declaration
        .definition
        .parent
        .ok_or("exact callable lacks its actual module owner")?;
    let valid_parent = match rows.declarations.get(parent.0 as usize) {
        Some(owner) if owner.definition.kind == DefinitionKind::Module => true,
        Some(owner)
            if allow_function_parent
                && owner.definition.kind == DefinitionKind::Function
                && matches!(
                    headers.header(parent, work)?,
                    Some(HeaderRef::Function(index))
                        if matches!(rows.functions[index as usize].body, FunctionBody::Original(_))
                ) =>
        {
            true
        }
        _ => false,
    };
    if !valid_parent
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
