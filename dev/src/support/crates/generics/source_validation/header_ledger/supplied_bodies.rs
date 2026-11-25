//! Bodies are present exactly once; declarations with no code stay bodyless.
use super::{DefinitionKind, HeaderRef, SourceRows, Work, s};
use crate::support::crates::owner_catalog::{BodyKind, BodyRef, DeclarationRef};

pub(super) fn validate(
    rows: &SourceRows,
    headers: &[Option<HeaderRef>],
    lookup: Option<&s::bodymap::Lookup>,
    work: &mut Work,
) -> Result<(), &'static str> {
    if rows.body_owners.len()
        != rows
            .originals
            .len()
            .saturating_add(rows.compiler_operations.len())
        || rows.originals.len() != rows.body_inputs.len()
    {
        return Err("source original/body/input tables have different complete coverage");
    }
    work.charge(rows.body_owners.len())?;
    let mut seen = vec![false; rows.body_owners.len()];
    for function in &rows.functions {
        work.charge(1)?;
        match function.body {
            s::FunctionBody::Intrinsic(_) => {}
            s::FunctionBody::Absent => {
                if function
                    .parent
                    .and_then(|parent| headers.get(parent.0 as usize))
                    .is_none_or(|header| !matches!(header, Some(HeaderRef::Trait(_))))
                    && !rows.bodyless_runtime(function.declaration, work)?
                    && rows
                        .declared_intrinsic(function.declaration, work)?
                        .is_none()
                {
                    return Err("source body absence is not an actual required trait method");
                }
            }
            s::FunctionBody::CompilerOperation(reference) => {
                let operation = rows
                    .compiler_operations
                    .get(reference.0 as usize)
                    .ok_or("source compiler operation reference is unknown")?;
                let owner = rows
                    .body_owners
                    .get(operation.body.0 as usize)
                    .ok_or("source compiler operation body owner is unknown")?;
                if operation.declaration != function.declaration
                    || owner.declaration != function.declaration
                    || owner.parent.is_some()
                    || owner.kind != BodyKind::Function
                    || owner.ordinal != 0
                    || std::mem::replace(&mut seen[operation.body.0 as usize], true)
                    || lookup.is_some_and(|lookup| lookup.original(operation.body).is_some())
                {
                    return Err(
                        "compiler operation has duplicate, foreign or ordinary code ownership",
                    );
                }
            }
            s::FunctionBody::Original(body) => {
                claim_original(
                    rows,
                    &mut seen,
                    body,
                    function.declaration,
                    BodyKind::Function,
                )?;
            }
        }
    }
    work.charge(rows.constants().len())?;
    if rows
        .constants()
        .iter()
        .any(|row| matches!(row.body, s::ConstantBody::Original(_)))
    {
        work.charge(rows.declarations.len())?;
        for (declaration, constant) in rows.constant_rows() {
            if let s::ConstantBody::Original(body) = constant.body {
                claim_original(rows, &mut seen, body, declaration, BodyKind::Constant)?;
            }
        }
    }
    claim_closures(rows, &mut seen, work)?;
    validate_originals(rows, &seen, lookup, work)?;
    if seen.contains(&false) {
        return Err("source bodies have incomplete exclusive representation coverage");
    }
    local_declarations(rows, lookup, work)?;
    caller_sites(rows, lookup, work)?;
    Ok(())
}

fn claim_closures(
    rows: &SourceRows,
    seen: &mut [bool],
    work: &mut Work,
) -> Result<(), &'static str> {
    let Some(closures) = rows.closures() else {
        return Ok(());
    };
    work.charge(closures.headers.len())?;
    let mut previous = None;
    for header in &closures.headers {
        if previous.is_some_and(|before| before >= header.body.0) {
            return Err("source closure headers lack canonical body order");
        }
        let owner = rows
            .body_owners
            .get(header.body.0 as usize)
            .ok_or("source closure body reference is unknown")?;
        if owner.declaration != header.owner
            || owner.parent.is_none()
            || owner.kind != BodyKind::Closure
            || std::mem::replace(&mut seen[header.body.0 as usize], true)
        {
            return Err("source closure body has duplicate, foreign or incompatible ownership");
        }
        previous = Some(header.body.0);
    }
    Ok(())
}

fn validate_originals(
    rows: &SourceRows,
    seen: &[bool],
    lookup: Option<&s::bodymap::Lookup>,
    work: &mut Work,
) -> Result<(), &'static str> {
    let mut previous_root = None;
    let mut previous_body = None;
    for (index, original) in rows.originals.iter().enumerate() {
        work.charge(1)?;
        let owner = rows
            .body_owners
            .get(original.body.0 as usize)
            .ok_or("unknown original body owner")?;
        let input = &rows.body_inputs[index];
        if lookup.is_none() && original.body.0 as usize != index
            || !seen[original.body.0 as usize]
            || input.body != original.body
            || original.declaration != owner.declaration
            || original.sites == 0
            || previous_body.is_some_and(|before| before >= original.body.0)
            || (owner.parent.is_none()
                && previous_root.is_some_and(|before| before >= owner.declaration.0))
        {
            return Err("source originals have missing, foreign, duplicate or reordered coverage");
        }
        previous_body = Some(original.body.0);
        if owner.parent.is_none() {
            previous_root = Some(owner.declaration.0);
        }
    }
    Ok(())
}

fn caller_sites(
    rows: &SourceRows,
    lookup: Option<&s::bodymap::Lookup>,
    work: &mut Work,
) -> Result<(), &'static str> {
    let Some(sites) = rows.cold().caller_sites.as_deref() else {
        return Ok(());
    };
    if sites.files.is_empty() || sites.rows.is_empty() {
        return Err("source caller site table must contain actual entries");
    }
    for file in &sites.files {
        work.charge(file.len().saturating_add(1))?;
        if file.is_empty() || file.len() > 4096 || file.as_bytes().contains(&0) {
            return Err("source caller site filename is empty, oversized or contains NUL");
        }
    }
    work.charge(sites.files.len())?;
    let mut used = vec![false; sites.files.len()];
    let mut previous = None;
    for row in &sites.rows {
        work.charge(1)?;
        if previous.is_some_and(|before| before >= (row.body.0, row.value)) {
            return Err("source caller sites lack canonical body and value order");
        }
        let index = s::bodymap::original(lookup, row.body)?;
        let original = rows
            .originals
            .get(index)
            .filter(|original| original.body == row.body)
            .ok_or("source caller site references an unknown original body")?;
        let Some(value) = original.values.get(row.value as usize) else {
            return Err("source caller site references an unknown value");
        };
        if !matches!(value.kind, s::ValueKind::Call { .. }) {
            return Err("source caller site must reference an actual Call value");
        }
        let used_file = used
            .get_mut(row.file as usize)
            .ok_or("source caller site file ordinal is out of range")?;
        *used_file = true;
        if row.line == 0 || row.column == 0 {
            return Err("source caller site has invalid line or column");
        }
        previous = Some((row.body.0, row.value));
    }
    if used.contains(&false) {
        return Err("source caller site file table contains an unreferenced path");
    }
    Ok(())
}

fn claim_original(
    rows: &SourceRows,
    seen: &mut [bool],
    body: BodyRef,
    declaration: DeclarationRef,
    kind: BodyKind,
) -> Result<(), &'static str> {
    let owner = rows
        .body_owners
        .get(body.0 as usize)
        .ok_or("source original body reference is unknown")?;
    if owner.declaration != declaration
        || owner.parent.is_some()
        || owner.kind != kind
        || owner.ordinal != 0
        || std::mem::replace(&mut seen[body.0 as usize], true)
    {
        return Err("source original has duplicate, foreign or incompatible ownership");
    }
    Ok(())
}

fn local_declarations(
    rows: &SourceRows,
    lookup: Option<&s::bodymap::Lookup>,
    work: &mut Work,
) -> Result<(), &'static str> {
    for (index, row) in rows.declarations.iter().enumerate() {
        work.charge(1)?;
        let Some(local) = row.local else {
            continue;
        };
        let body = rows
            .originals
            .get(s::bodymap::original(lookup, local.body)?)
            .ok_or("source local declaration body is unknown")?;
        let value = body
            .locals
            .get(local.ordinal as usize)
            .ok_or("source local declaration ordinal is unknown")?;
        let declaration = DeclarationRef(
            u32::try_from(index).map_err(|_| "source local declaration identity overflow")?,
        );
        let kind_ok = matches!(
            (row.definition.kind, value.kind),
            (
                DefinitionKind::ValueParameter,
                crate::support::crates::generics::LocalKind::Parameter(_)
                    | crate::support::crates::generics::LocalKind::Binding,
            ) | (
                DefinitionKind::Local,
                crate::support::crates::generics::LocalKind::Binding
            )
        );
        if row.definition.parent != Some(body.declaration)
            || value.definition != Some(declaration)
            || value.scope != local.scope
            || !kind_ok
        {
            return Err("source local declaration is foreign to its actual typed body slot");
        }
    }
    Ok(())
}
