//! Flat CFG/storage coordinates stay owned by the same original body.
use super::{DeclarationRef, Input, Writer, g, id, option, range};

pub(super) fn local_write(out: &mut Writer, row: &g::Local) -> Result<(), &'static str> {
    id(out, row.scope)?;
    option(out, row.storage_parent)?;
    id(out, row.ty.0)?;
    match row.kind {
        g::LocalKind::Parameter(index) => {
            out.u8(0)?;
            id(out, index)?;
        }
        g::LocalKind::Binding => out.u8(1)?,
        g::LocalKind::Merge => out.u8(2)?,
    }
    option(out, row.definition.map(|r| r.0))?;
    out.u8(u8::from(row.mutable))?;
    id(out, row.site)
}
pub(super) fn local_read(read: &mut Input<'_, '_>) -> Result<g::Local, &'static str> {
    let scope = read.id()?;
    let storage_parent = read.option()?;
    let ty = g::TypeRef(read.id()?);
    let kind = match read.reader.u8()? {
        0 => g::LocalKind::Parameter(read.id()?),
        1 => g::LocalKind::Binding,
        2 => g::LocalKind::Merge,
        _ => return Err("unknown source original local kind"),
    };
    Ok(g::Local {
        scope,
        storage_parent,
        ty,
        kind,
        definition: read.option()?.map(DeclarationRef),
        mutable: read.boolean()?,
        site: read.id()?,
    })
}
pub(super) fn parameter_write(
    out: &mut Writer,
    row: &g::ParameterBinding,
) -> Result<(), &'static str> {
    for value in [row.declaration.0, row.input, row.local] {
        id(out, value)?;
    }
    Ok(())
}
pub(super) fn parameter_read(
    read: &mut Input<'_, '_>,
) -> Result<g::ParameterBinding, &'static str> {
    Ok(g::ParameterBinding {
        declaration: DeclarationRef(read.id()?),
        input: read.id()?,
        local: read.id()?,
    })
}
pub(super) fn block_write(out: &mut Writer, row: &g::Block) -> Result<(), &'static str> {
    range(out, row.statements)?;
    id(out, row.site)?;
    match row.terminator {
        g::Terminator::Return(value) => {
            out.u8(0)?;
            id(out, value)
        }
        g::Terminator::Jump(block) => {
            out.u8(1)?;
            id(out, block)
        }
        g::Terminator::Unreachable => out.u8(2),
    }
}
pub(super) fn block_read(read: &mut Input<'_, '_>) -> Result<g::Block, &'static str> {
    let statements = read.range()?;
    let site = read.id()?;
    let terminator = match read.reader.u8()? {
        0 => g::Terminator::Return(read.id()?),
        1 => g::Terminator::Jump(read.id()?),
        2 => g::Terminator::Unreachable,
        _ => return Err("unknown source original terminator"),
    };
    Ok(g::Block {
        statements,
        site,
        terminator,
    })
}
pub(super) fn scope_write(out: &mut Writer, row: &g::Scope) -> Result<(), &'static str> {
    option(out, row.parent)?;
    out.u8(match row.kind {
        g::ScopeKind::Body => 0,
        g::ScopeKind::Block => 1,
        g::ScopeKind::Statement => 2,
        g::ScopeKind::Tail => 3,
        g::ScopeKind::Condition => 4,
        g::ScopeKind::IfThen => 5,
        g::ScopeKind::Iteration => 6,
        g::ScopeKind::Arm => 7,
        g::ScopeKind::Guard => 8,
    })?;
    id(out, row.site)
}
pub(super) fn scope_read(read: &mut Input<'_, '_>) -> Result<g::Scope, &'static str> {
    let parent = read.option()?;
    let kind = match read.reader.u8()? {
        0 => g::ScopeKind::Body,
        1 => g::ScopeKind::Block,
        2 => g::ScopeKind::Statement,
        3 => g::ScopeKind::Tail,
        4 => g::ScopeKind::Condition,
        5 => g::ScopeKind::IfThen,
        6 => g::ScopeKind::Iteration,
        7 => g::ScopeKind::Arm,
        8 => g::ScopeKind::Guard,
        _ => return Err("unknown source original scope kind"),
    };
    Ok(g::Scope {
        parent,
        kind,
        site: read.id()?,
    })
}
pub(super) fn location_write(out: &mut Writer, location: g::Location) -> Result<(), &'static str> {
    let (tag, block, point) = match location {
        g::Location::At { block, before } => (0, block, before),
        g::Location::Edge { block, target } => (1, block, target),
    };
    out.u8(tag)?;
    id(out, block)?;
    id(out, point)
}
pub(super) fn location_read(read: &mut Input<'_, '_>) -> Result<g::Location, &'static str> {
    Ok(match read.reader.u8()? {
        0 => g::Location::At {
            block: read.id()?,
            before: read.id()?,
        },
        1 => g::Location::Edge {
            block: read.id()?,
            target: read.id()?,
        },
        _ => return Err("unknown source original location"),
    })
}
pub(super) fn exit_write(out: &mut Writer, row: &g::Exit) -> Result<(), &'static str> {
    location_write(out, row.location)?;
    match row.end {
        g::End::Scopes { inner, outer } => {
            out.u8(0)?;
            id(out, inner)?;
            option(out, outer)
        }
        g::End::Temporaries(values) => {
            out.u8(1)?;
            range(out, values)
        }
    }
}
pub(super) fn exit_read(read: &mut Input<'_, '_>) -> Result<g::Exit, &'static str> {
    let location = location_read(read)?;
    let end = match read.reader.u8()? {
        0 => g::End::Scopes {
            inner: read.id()?,
            outer: read.option()?,
        },
        1 => g::End::Temporaries(read.range()?),
        _ => return Err("unknown source original exit kind"),
    };
    Ok(g::Exit { location, end })
}
pub(super) fn temporary_write(out: &mut Writer, row: &g::Temporary) -> Result<(), &'static str> {
    match row.lifetime {
        g::Lifetime::Scope(scope) => {
            out.u8(0)?;
            id(out, scope)?;
        }
        g::Lifetime::Static => out.u8(1)?,
    }
    for value in [row.value, row.site, row.block, row.before, row.exits_before] {
        id(out, value)?;
    }
    Ok(())
}
pub(super) fn temporary_read(read: &mut Input<'_, '_>) -> Result<g::Temporary, &'static str> {
    let lifetime = match read.reader.u8()? {
        0 => g::Lifetime::Scope(read.id()?),
        1 => g::Lifetime::Static,
        _ => return Err("unknown source original temporary lifetime"),
    };
    Ok(g::Temporary {
        lifetime,
        value: read.id()?,
        site: read.id()?,
        block: read.id()?,
        before: read.id()?,
        exits_before: read.id()?,
    })
}
