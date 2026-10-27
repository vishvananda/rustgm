//! The provided v1 suffix owns names and explicit written-modifier contracts once.
use super::{Input, Writer, address_read, address_write, id, s, write};
use crate::support::crates::{
    MAX_COMPONENTS, MAX_FUNCTIONS, Namespace, owner_catalog::DeclarationRef,
};

pub(super) fn encode(out: &mut Writer, rows: &s::ProvidedContractRows) -> Result<(), &'static str> {
    id(out, rows.namespace.root.0)?;
    write(out, &rows.namespace.modules, |out, row| id(out, row.0))?;
    write(out, &rows.namespace.bindings, |out, row| {
        id(out, row.module.0)?;
        out.string(&row.name)?;
        out.u8(match row.namespace {
            Namespace::Type => 0,
            Namespace::Value => 1,
        })?;
        address_write(out, row.target)
    })?;
    for column in [
        &rows.modifiers.functions,
        &rows.modifiers.traits,
        &rows.modifiers.implementations,
    ] {
        write(out, column, |out, row| {
            id(out, row.declaration.0)?;
            id(out, row.site)?;
            out.u8(match row.policy {
                s::HeaderModifierPolicy::NoWrittenDefaultOrFinalOrCallerTracking => 0,
            })
        })?;
    }
    Ok(())
}
pub(super) fn decode(read: &mut Input<'_, '_>) -> Result<s::ProvidedContractRows, &'static str> {
    read.reserve(size_of::<s::ProvidedContractRows>())?;
    let root = DeclarationRef(read.id()?);
    let modules = read.list(MAX_FUNCTIONS, 4, |read| Ok(DeclarationRef(read.id()?)))?;
    let bindings = read.list(MAX_COMPONENTS, 17, |read| {
        let module = DeclarationRef(read.id()?);
        let name = read.string()?;
        let namespace = match read.reader.u8()? {
            0 => Namespace::Type,
            1 => Namespace::Value,
            _ => return Err("unknown provided namespace tag"),
        };
        Ok(s::ExportBinding {
            module,
            name,
            namespace,
            target: address_read(read)?,
        })
    })?;
    Ok(s::ProvidedContractRows {
        namespace: s::NamespaceColumns {
            root,
            modules,
            bindings,
        },
        modifiers: s::HeaderModifierColumns {
            functions: modifiers(read)?,
            traits: modifiers(read)?,
            implementations: modifiers(read)?,
        },
    })
}
fn modifiers(read: &mut Input<'_, '_>) -> Result<Vec<s::HeaderModifierRecord>, &'static str> {
    read.list(MAX_FUNCTIONS, 9, |read| {
        let declaration = DeclarationRef(read.id()?);
        let site = read.id()?;
        let policy = match read.reader.u8()? {
            0 => s::HeaderModifierPolicy::NoWrittenDefaultOrFinalOrCallerTracking,
            _ => return Err("unsupported provided written-modifier policy"),
        };
        Ok(s::HeaderModifierRecord {
            declaration,
            site,
            policy,
        })
    })
}
