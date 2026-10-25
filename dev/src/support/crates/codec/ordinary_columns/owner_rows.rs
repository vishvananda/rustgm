//! Distinct lexical declarations, generic slots, binders and executable owners.
use super::{Input, Writer, address_read, address_write, g, id, option, range, s};
use crate::support::crates::owner_catalog::{BodyKind, BodyOwner, BodyRef, DeclarationRef};

pub(super) fn declaration_write(
    out: &mut Writer,
    row: &s::Declaration,
) -> Result<(), &'static str> {
    out.u8(match row.definition.kind {
        s::DefinitionKind::Module => 0,
        s::DefinitionKind::Trait => 1,
        s::DefinitionKind::SelfType => 2,
        s::DefinitionKind::Function => 3,
        s::DefinitionKind::TypeParameter => 4,
        s::DefinitionKind::RegionParameter => 5,
        s::DefinitionKind::ValueParameter => 6,
        s::DefinitionKind::Local => 7,
    })?;
    option(out, row.definition.parent.map(|r| r.0))?;
    id(out, row.ordinal)?;
    out.u8(u8::from(row.name.is_some()))?;
    if let Some(name) = &row.name {
        out.string(name)?;
    }
    out.u8(u8::from(row.public))?;
    id(out, row.sites)?;
    out.u8(u8::from(row.local.is_some()))?;
    if let Some(local) = row.local {
        for value in [local.body.0, local.scope, local.ordinal] {
            id(out, value)?;
        }
    }
    Ok(())
}
pub(super) fn declaration_read(read: &mut Input<'_, '_>) -> Result<s::Declaration, &'static str> {
    let kind = match read.reader.u8()? {
        0 => s::DefinitionKind::Module,
        1 => s::DefinitionKind::Trait,
        2 => s::DefinitionKind::SelfType,
        3 => s::DefinitionKind::Function,
        4 => s::DefinitionKind::TypeParameter,
        5 => s::DefinitionKind::RegionParameter,
        6 => s::DefinitionKind::ValueParameter,
        7 => s::DefinitionKind::Local,
        _ => return Err("unknown source declaration kind"),
    };
    Ok(s::Declaration {
        definition: s::Definition {
            kind,
            parent: read.option()?.map(DeclarationRef),
        },
        ordinal: read.id()?,
        name: if read.boolean()? {
            Some(read.string()?)
        } else {
            None
        },
        public: read.boolean()?,
        sites: read.id()?,
        local: if read.boolean()? {
            Some(s::LocalDeclaration {
                body: BodyRef(read.id()?),
                scope: read.id()?,
                ordinal: read.id()?,
            })
        } else {
            None
        },
    })
}
pub(super) fn body_write(out: &mut Writer, row: &BodyOwner) -> Result<(), &'static str> {
    id(out, row.declaration.0)?;
    option(out, row.parent.map(|r| r.0))?;
    out.u8(match row.kind {
        BodyKind::Function => 0,
        BodyKind::Constant => 1,
        BodyKind::Static => 2,
        BodyKind::Closure => 3,
        BodyKind::Anonymous => 4,
    })?;
    id(out, row.ordinal)
}
pub(super) fn body_read(read: &mut Input<'_, '_>) -> Result<BodyOwner, &'static str> {
    let declaration = DeclarationRef(read.id()?);
    let parent = read.option()?.map(BodyRef);
    let kind = match read.reader.u8()? {
        0 => BodyKind::Function,
        1 => BodyKind::Constant,
        2 => BodyKind::Static,
        3 => BodyKind::Closure,
        4 => BodyKind::Anonymous,
        _ => return Err("unknown source body owner kind"),
    };
    Ok(BodyOwner {
        declaration,
        parent,
        kind,
        ordinal: read.id()?,
    })
}
pub(super) fn slot_write(out: &mut Writer, row: &s::ParameterSlot) -> Result<(), &'static str> {
    for value in [row.owner.0, row.ordinal, row.index] {
        id(out, value)?;
    }
    option(out, row.definition.map(|r| r.0))?;
    match row.kind {
        s::ParameterKind::Type { sized } => {
            out.u8(0)?;
            out.u8(u8::from(sized.is_some()))?;
            if let Some(address) = sized {
                address_write(out, address)?;
            }
        }
        s::ParameterKind::Region => out.u8(1)?,
    }
    Ok(())
}
pub(super) fn slot_read(read: &mut Input<'_, '_>) -> Result<s::ParameterSlot, &'static str> {
    let owner = DeclarationRef(read.id()?);
    let ordinal = read.id()?;
    let index = read.id()?;
    let definition = read.option()?.map(DeclarationRef);
    let kind = match read.reader.u8()? {
        0 => s::ParameterKind::Type {
            sized: if read.boolean()? {
                Some(address_read(read)?)
            } else {
                None
            },
        },
        1 => s::ParameterKind::Region,
        _ => return Err("unknown source parameter kind"),
    };
    Ok(s::ParameterSlot {
        owner,
        ordinal,
        index,
        definition,
        kind,
    })
}
pub(super) fn scope_write(out: &mut Writer, row: &s::ParameterScope) -> Result<(), &'static str> {
    id(out, row.owner.0)?;
    option(out, row.parent.map(|r| r.0))?;
    range(out, row.declared)?;
    id(out, row.captures)
}
pub(super) fn scope_read(read: &mut Input<'_, '_>) -> Result<s::ParameterScope, &'static str> {
    Ok(s::ParameterScope {
        owner: DeclarationRef(read.id()?),
        parent: read.option()?.map(DeclarationRef),
        declared: read.range()?,
        captures: read.id()?,
    })
}
pub(super) fn binder_write(out: &mut Writer, row: &s::Binder) -> Result<(), &'static str> {
    id(out, row.owner.0)?;
    option(out, row.parent.map(|r| r.0))?;
    out.u8(match row.kind {
        s::BinderKind::DeclarationHeader => 0,
        s::BinderKind::OriginalFunction => 1,
        s::BinderKind::PublicFunction => 2,
        s::BinderKind::Clause => 3,
        s::BinderKind::CallableType => 4,
        s::BinderKind::CallSignature { .. } => {
            return Err("source call binder requires a wider source profile");
        }
    })?;
    id(out, row.count)
}
pub(super) fn binder_read(read: &mut Input<'_, '_>) -> Result<s::Binder, &'static str> {
    let owner = DeclarationRef(read.id()?);
    let parent = read.option()?.map(g::BinderRef);
    let kind = match read.reader.u8()? {
        0 => s::BinderKind::DeclarationHeader,
        1 => s::BinderKind::OriginalFunction,
        2 => s::BinderKind::PublicFunction,
        3 => s::BinderKind::Clause,
        4 => s::BinderKind::CallableType,
        _ => return Err("unsupported source binder kind"),
    };
    Ok(s::Binder {
        owner,
        parent,
        kind,
        count: read.id()?,
    })
}
pub(super) fn slot_ref_write(out: &mut Writer, row: s::SlotRef) -> Result<(), &'static str> {
    id(out, row.0)
}
pub(super) fn slot_ref_read(read: &mut Input<'_, '_>) -> Result<s::SlotRef, &'static str> {
    Ok(s::SlotRef(read.id()?))
}
pub(super) fn declaration_ref_write(
    out: &mut Writer,
    row: DeclarationRef,
) -> Result<(), &'static str> {
    id(out, row.0)
}
pub(super) fn declaration_ref_read(
    read: &mut Input<'_, '_>,
) -> Result<DeclarationRef, &'static str> {
    Ok(DeclarationRef(read.id()?))
}
