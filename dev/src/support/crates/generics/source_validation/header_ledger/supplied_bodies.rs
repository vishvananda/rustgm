//! Bodies are present exactly once; required trait declarations stay bodyless.
use super::{DefinitionKind, HeaderRef, SourceRows, Work, s};
use crate::support::crates::owner_catalog::{BodyKind, DeclarationRef};

pub(super) fn validate(
    rows: &SourceRows,
    headers: &[Option<HeaderRef>],
    work: &mut Work,
) -> Result<(), &'static str> {
    if rows.body_owners.len() != rows.originals.len()
        || rows.body_owners.len() != rows.body_inputs.len()
    {
        return Err("source original/body/input tables have different complete coverage");
    }
    work.charge(rows.body_owners.len())?;
    let mut seen = vec![false; rows.body_owners.len()];
    for function in &rows.functions {
        work.charge(1)?;
        match function.body {
            s::FunctionBody::Absent => {
                if function
                    .parent
                    .and_then(|parent| headers.get(parent.0 as usize))
                    .is_none_or(|header| !matches!(header, Some(HeaderRef::Trait(_))))
                {
                    return Err("source body absence is not an actual required trait method");
                }
            }
            s::FunctionBody::Original(body) => {
                let owner = rows
                    .body_owners
                    .get(body.0 as usize)
                    .ok_or("source function original body reference is unknown")?;
                if owner.declaration != function.declaration
                    || owner.parent.is_some()
                    || owner.kind != BodyKind::Function
                    || owner.ordinal != 0
                    || std::mem::replace(&mut seen[body.0 as usize], true)
                {
                    return Err(
                        "source function body has duplicate, foreign or incompatible ownership",
                    );
                }
            }
        }
    }
    let mut previous = None;
    for (index, owner) in rows.body_owners.iter().enumerate() {
        work.charge(1)?;
        let original = &rows.originals[index];
        let input = &rows.body_inputs[index];
        if !seen[index]
            || original.body.0 as usize != index
            || input.body != original.body
            || original.declaration != owner.declaration
            || original.sites == 0
            || previous.is_some_and(|before| before >= owner.declaration.0)
        {
            return Err("source originals have missing, foreign, duplicate or reordered coverage");
        }
        previous = Some(owner.declaration.0);
    }
    for (index, row) in rows.declarations.iter().enumerate() {
        work.charge(1)?;
        let Some(local) = row.local else {
            continue;
        };
        let body = rows
            .originals
            .get(local.body.0 as usize)
            .ok_or("source local declaration body is unknown")?;
        let value = body
            .locals
            .get(local.ordinal as usize)
            .ok_or("source local declaration ordinal is unknown")?;
        let declaration = DeclarationRef(
            u32::try_from(index).map_err(|_| "source local declaration identity overflow")?,
        );
        if row.definition.parent != Some(body.declaration)
            || value.definition != Some(declaration)
            || value.scope != local.scope
            || matches!(row.definition.kind, DefinitionKind::ValueParameter)
                != matches!(
                    value.kind,
                    crate::support::crates::generics::LocalKind::Parameter(_)
                )
        {
            return Err("source local declaration is foreign to its actual typed body slot");
        }
    }
    Ok(())
}
