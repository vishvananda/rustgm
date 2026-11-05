//! Nominal identity and field completeness belong to the source header ledger.
use super::{DefinitionKind, HeaderRef, Profile, SourceRows, Type};
use crate::support::{crates::generics::Work, hash::HashSet};
pub(super) fn require_profile(
    rows: &SourceRows,
    profile: Profile,
    work: &mut Work,
) -> Result<(), &'static str> {
    work.charge(
        rows.types
            .len()
            .saturating_add(rows.declarations.len())
            .saturating_add(1),
    )?;
    for body in &rows.originals {
        work.charge(
            body.values
                .len()
                .saturating_add(body.projections.len())
                .saturating_add(1),
        )?;
        if !profile.nominals()
            && (body
                .values
                .iter()
                .any(|value| matches!(value.kind, super::ValueKind::Aggregate { .. }))
                || body.projections.iter().any(|projection| {
                    matches!(projection.kind, super::ProjectionKind::Field { .. })
                }))
        {
            return Err("source nominal operations require their explicit version-ten profile");
        }
    }
    if !profile.nominals()
        && (!rows.nominals.is_empty()
            || !rows.fields.is_empty()
            || rows
                .types
                .iter()
                .any(|row| matches!(row, Type::Nominal { .. }))
            || rows
                .declarations
                .iter()
                .any(|row| row.definition.kind == DefinitionKind::Struct))
    {
        return Err("source nominal contracts require their explicit version-ten profile");
    }
    Ok(())
}
pub(super) fn headers(
    rows: &SourceRows,
    headers: &mut [Option<HeaderRef>],
    work: &mut Work,
) -> Result<(), &'static str> {
    let mut cursor = 0;
    for (index, row) in rows.nominals.iter().enumerate() {
        work.charge(1)?;
        let mut contract_bytes = 0;
        super::HeaderAttributeColumns::check_row(
            &row.contract,
            row.declaration,
            &rows.declarations,
            &mut contract_bytes,
            work,
        )?;
        if row.contract.inline.is_some() {
            return Err("source nominal cannot carry effective callable inline policy");
        }
        let declaration = rows
            .declarations
            .get(row.declaration.0 as usize)
            .ok_or("unknown source nominal declaration")?;
        let cell = headers
            .get_mut(row.declaration.0 as usize)
            .ok_or("unknown source nominal header")?;
        if declaration.definition.kind != DefinitionKind::Struct
            || row.scope != row.declaration
            || cell.is_some()
            || row.fields.start != cursor
            || row.fields.end < cursor
        {
            return Err("source nominal changed its actual owner, scope or field range");
        }
        let fields = row.fields.slice(&rows.fields)?;
        let mut names = HashSet::default();
        for (ordinal, field) in fields.iter().enumerate() {
            work.charge(field.name.len().saturating_add(1))?;
            if field.owner != row.declaration
                || field.ordinal as usize != ordinal
                || field.site >= declaration.sites
                || field.ty.0 as usize >= rows.types.len()
                || !names.insert(field.name.as_str())
            {
                return Err("source nominal field changed its owner, ordinal, type, site or name");
            }
            if let super::ModuleVisibility::Within(module) = field.visibility
                && rows
                    .declarations
                    .get(module.0 as usize)
                    .is_none_or(|declaration| declaration.definition.kind != DefinitionKind::Module)
            {
                return Err("source nominal field visibility has a foreign actual module");
            }
            crate::support::crates::identity::validate_component(&field.name)?;
        }
        let Some(Type::Nominal {
            declaration: address,
            ..
        }) = rows.types.get(row.self_type.0 as usize)
        else {
            return Err("source nominal omitted its actual nominal Self type");
        };
        if address.artifact.0 != 0 || address.declaration != row.declaration {
            return Err("source nominal Self type belongs to another actual declaration");
        }
        *cell = Some(HeaderRef::Nominal(
            u32::try_from(index).map_err(|_| "source nominal ordinal overflow")?,
        ));
        cursor = row.fields.end;
    }
    if cursor as usize != rows.fields.len() {
        return Err("source nominal fields have incomplete owner coverage");
    }
    Ok(())
}

pub(super) fn scopes(
    rows: &SourceRows,
    scopes: &super::ScopeIndex<'_>,
    work: &mut Work,
) -> Result<(), &'static str> {
    for header in &rows.nominals {
        work.charge(1)?;
        let range = scopes.declared_slots(header.scope, work)?;
        let Some(Type::Nominal { arguments, .. }) = rows.types.get(header.self_type.0 as usize)
        else {
            return Err("source nominal Self lacks its genuine application recipe");
        };
        let arguments = arguments.slice(&rows.arguments)?;
        if arguments.len() != scopes.complete_slots(header.scope, work)? as usize
            || arguments.len() != (range.end - range.start) as usize
        {
            return Err("source nominal Self changed its complete actual formal domain");
        }
        for (offset, argument) in arguments.iter().enumerate() {
            work.charge(1)?;
            let slot = super::SlotRef(
                range
                    .start
                    .checked_add(
                        u32::try_from(offset)
                            .map_err(|_| "source nominal formal ordinal overflow")?,
                    )
                    .ok_or("source nominal formal ordinal overflow")?,
            );
            let valid = match (argument, rows.slots[slot.0 as usize].kind) {
                (super::Argument::Type(root), super::ParameterKind::Type { .. }) => {
                    rows.types.get(root.0 as usize) == Some(&Type::Parameter(slot))
                }
                (super::Argument::Region(region), super::ParameterKind::Region) => {
                    rows.regions.get(region.0 as usize)
                        == Some(&super::Region::Parameter { slot, depth: 0 })
                }
                _ => false,
            };
            if !valid {
                return Err(
                    "source nominal Self changed its exact formal identity or argument order",
                );
            }
        }
    }
    Ok(())
}

#[cfg(test)]
#[path = "../../../../tests/invariants/forged_records.rs"]
mod tests;
