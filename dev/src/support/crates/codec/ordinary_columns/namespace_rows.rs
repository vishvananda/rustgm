//! Version-two module/site/implementation-domain rows have no legacy fallback.
use super::{Input, Writer, address_read, address_write, id, s, write};
use crate::support::crates::{
    MAX_COMPONENTS, MAX_FUNCTIONS, Namespace, owner_catalog::DeclarationRef,
};
pub(super) fn encode(out: &mut Writer, rows: &s::ModuleContractRows) -> Result<(), &'static str> {
    id(out, rows.namespace.root.0)?;
    write(out, &rows.namespace.modules, |out, row| {
        id(out, row.declaration.0)?;
        id(out, row.declaration_site)
    })?;
    write(out, &rows.namespace.bindings, binding_write)?;
    write(out, &rows.domains, domain_write)
}
pub(super) fn decode(read: &mut Input<'_, '_>) -> Result<s::ModuleContractRows, &'static str> {
    read.reserve(size_of::<s::ModuleContractRows>())?;
    let root = DeclarationRef(read.id()?);
    let modules = read.list(MAX_FUNCTIONS, 8, |read| {
        Ok(s::ModuleRow {
            declaration: DeclarationRef(read.id()?),
            declaration_site: read.id()?,
        })
    })?;
    let bindings = read.list(MAX_COMPONENTS, 23, binding_read)?;
    let domains = read.list(MAX_FUNCTIONS, 13, domain_read)?;
    Ok(s::ModuleContractRows {
        namespace: s::ModuleNamespace {
            root,
            modules,
            bindings,
        },
        domains,
    })
}
fn binding_write(out: &mut Writer, row: &s::ModuleBindingRow) -> Result<(), &'static str> {
    id(out, row.module.0)?;
    out.string(&row.name)?;
    out.u8(match row.namespace {
        Namespace::Type => 0,
        Namespace::Value => 1,
    })?;
    address_write(out, row.target)?;
    match row.visibility {
        s::ModuleVisibility::Public => out.u8(0)?,
        s::ModuleVisibility::Within(module) => {
            out.u8(1)?;
            id(out, module.0)?;
        }
    }
    out.u8(match row.source {
        s::BindingSource::Declaration => 0,
        s::BindingSource::Import => 1,
        s::BindingSource::Glob => 2,
    })?;
    id(out, row.source_site)
}
fn binding_read(read: &mut Input<'_, '_>) -> Result<s::ModuleBindingRow, &'static str> {
    let module = DeclarationRef(read.id()?);
    let name = read.string()?;
    let namespace = match read.reader.u8()? {
        0 => Namespace::Type,
        1 => Namespace::Value,
        _ => return Err("unknown module binding namespace"),
    };
    let target = address_read(read)?;
    let visibility = match read.reader.u8()? {
        0 => s::ModuleVisibility::Public,
        1 => s::ModuleVisibility::Within(DeclarationRef(read.id()?)),
        _ => return Err("unknown module binding visibility"),
    };
    let source = match read.reader.u8()? {
        0 => s::BindingSource::Declaration,
        1 => s::BindingSource::Import,
        2 => s::BindingSource::Glob,
        _ => return Err("unknown module binding source"),
    };
    Ok(s::ModuleBindingRow {
        module,
        name,
        namespace,
        target,
        visibility,
        source,
        source_site: read.id()?,
    })
}
fn domain_write(out: &mut Writer, row: &s::TraitDomainRow) -> Result<(), &'static str> {
    for value in [
        row.declaration.0,
        row.declaration_site,
        row.defining_module.0,
    ] {
        id(out, value)?;
    }
    match row.domain {
        s::ImplementationDomain::Unrestricted => out.u8(0),
        s::ImplementationDomain::RestrictedTo {
            module,
            clause_site,
            path_site,
        } => {
            out.u8(1)?;
            id(out, module.0)?;
            id(out, clause_site)?;
            id(out, path_site)
        }
    }
}
fn domain_read(read: &mut Input<'_, '_>) -> Result<s::TraitDomainRow, &'static str> {
    let declaration = DeclarationRef(read.id()?);
    let declaration_site = read.id()?;
    let defining_module = DeclarationRef(read.id()?);
    let domain = match read.reader.u8()? {
        0 => s::ImplementationDomain::Unrestricted,
        1 => s::ImplementationDomain::RestrictedTo {
            module: DeclarationRef(read.id()?),
            clause_site: read.id()?,
            path_site: read.id()?,
        },
        _ => return Err("unknown implementation domain policy"),
    };
    Ok(s::TraitDomainRow {
        declaration,
        declaration_site,
        defining_module,
        domain,
    })
}
