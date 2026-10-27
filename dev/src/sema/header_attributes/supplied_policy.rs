//! Only a complete explicit provided-profile row certifies plain function policy.
use super::{Declarations, DefId, DefKind, s};

pub(in crate::sema) fn plain_function(
    facts: &Declarations,
    definition: DefId,
) -> Result<bool, &'static str> {
    let Some(headers) = facts
        .must_use_contracts
        .as_deref()
        .and_then(|owner| owner.headers.as_deref())
        .and_then(|ledger| ledger.imported.as_deref())
    else {
        return Ok(false);
    };
    headers
        .charge(6)
        .map_err(|_| "provided policy work budget exceeded")?;
    let Some(header) = headers.entries.get(&definition) else {
        return Ok(false);
    };
    let descriptor = headers.inventory.artifacts()[header.artifact]
        .source_descriptor()
        .ok_or("provided policy lost its actual source descriptor")?;
    if descriptor.profile() != s::Profile::Source2024ProvidedContracts {
        return Ok(false);
    }
    let catalog = facts
        .crates
        .as_deref()
        .and_then(crate::sema::crates::Linkage::origin_catalog)
        .ok_or("provided policy lost its current catalog")?;
    if !std::ptr::eq(catalog, headers.catalog.as_ref()) {
        return Err("provided policy changed its actual current catalog");
    }
    catalog.validate_declaration(header.handle, definition, DefKind::Function)?;
    let coordinate = catalog.declaration_coordinate(header.handle)?;
    if coordinate.0 != descriptor.identity()
        || coordinate.1 != header.declaration
        || coordinate.2 != descriptor.edition()
    {
        return Err("provided policy changed its actual function coordinate");
    }
    let semantic = facts
        .declaration_header(definition)?
        .ok_or("provided policy omitted its canonical function header")?;
    if semantic.declaration
        != crate::resolve::body_origin::DeclarationOrigin::Imported(header.handle)
    {
        return Err("provided policy changed its canonical function owner");
    }
    let rows = descriptor.rows();
    let Some(s::HeaderRef::Function(index)) = rows.header(header.declaration)? else {
        return Err("provided policy does not name an actual function header");
    };
    let columns = descriptor.provided_contracts()?;
    let modifier = columns
        .rows()
        .modifiers
        .functions
        .get(index as usize)
        .ok_or("provided policy omitted its complete function row")?;
    let contract = rows
        .header_contract(header.declaration)?
        .ok_or("provided policy omitted its single advisory row")?;
    if modifier.declaration != header.declaration || modifier.site != contract.declaration_site {
        return Err("provided policy changed its actual declaration site");
    }
    match modifier.policy {
        s::HeaderModifierPolicy::NoWrittenDefaultOrFinalOrCallerTracking => Ok(true),
    }
}
