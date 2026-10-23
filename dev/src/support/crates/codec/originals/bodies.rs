//! Version-zero operation rows preserve flat CFG and lexical storage indices.
use super::{BodyRef, DeclarationRef, Input, Writer, crates, g, id, option, range};

pub(super) fn encode(
    out: &mut Writer,
    body: &g::Original,
    version: u8,
) -> Result<(), &'static str> {
    for value in [body.body.0, body.declaration.0, body.result.0, body.sites] {
        id(out, value)?;
    }
    locals_write(out, &body.locals)?;
    out.u32(body.parameter_bindings.len())?;
    for binding in &body.parameter_bindings {
        id(out, binding.declaration.0)?;
        id(out, binding.input)?;
        id(out, binding.local)?;
    }
    out.u32(body.values.len())?;
    super::opcodes::encode(out, &body.values, version)?;
    out.u32(body.statements.len())?;
    for statement in &body.statements {
        id(out, statement.site)?;
        action_write(out, statement.action)?;
    }
    out.u32(body.blocks.len())?;
    for block in &body.blocks {
        range(out, block.statements)?;
        id(out, block.site)?;
        match block.terminator {
            g::Terminator::Return(value) => {
                out.u8(0)?;
                id(out, value)?;
            }
            g::Terminator::Jump(target) => {
                out.u8(1)?;
                id(out, target)?;
            }
            g::Terminator::Unreachable => out.u8(2)?,
        }
    }
    out.u32(body.scopes.len())?;
    for scope in &body.scopes {
        option(out, scope.parent)?;
        out.u8(scope_write(scope.kind))?;
        id(out, scope.site)?;
    }
    out.u32(body.exits.len())?;
    for exit in &body.exits {
        match exit.location {
            g::Location::At { block, before } => {
                out.u8(0)?;
                id(out, block)?;
                id(out, before)?;
            }
            g::Location::Edge { block, target } => {
                out.u8(1)?;
                id(out, block)?;
                id(out, target)?;
            }
        }
        match exit.end {
            g::End::Scopes { inner, outer } => {
                out.u8(0)?;
                id(out, inner)?;
                option(out, outer)?;
            }
            g::End::Temporaries(values) => {
                out.u8(1)?;
                range(out, values)?;
            }
        }
    }
    out.u32(body.temporaries.len())?;
    for temporary in &body.temporaries {
        match temporary.lifetime {
            g::Lifetime::Scope(scope) => {
                out.u8(0)?;
                id(out, scope)?;
            }
            g::Lifetime::Static => out.u8(1)?,
        }
        for value in [
            temporary.value,
            temporary.site,
            temporary.block,
            temporary.before,
            temporary.exits_before,
        ] {
            id(out, value)?;
        }
    }
    if version == 1 {
        out.u32(body.arguments.len())?;
        for &value in &body.arguments {
            id(out, value)?;
        }
    } else if !body.arguments.is_empty() {
        return Err("generic value lists require row version one");
    }
    Ok(())
}
fn action_write(out: &mut Writer, action: g::Action) -> Result<(), &'static str> {
    let (tag, first, second) = match action {
        g::Action::StorageLive(local) => (0, local, None),
        g::Action::StorageDead(local) => (1, local, None),
        g::Action::StorageEnd { first, last } => (2, first, Some(last)),
        g::Action::Evaluate(value) => (3, value, None),
        g::Action::Store { local, value } => (4, local, Some(value)),
        g::Action::CheckInitialized(local) => (5, local, None),
        g::Action::Mention(local) => (6, local, None),
    };
    out.u8(tag)?;
    id(out, first)?;
    if let Some(value) = second {
        id(out, value)?;
    }
    Ok(())
}
fn scope_write(kind: g::ScopeKind) -> u8 {
    match kind {
        g::ScopeKind::Body => 0,
        g::ScopeKind::Block => 1,
        g::ScopeKind::Statement => 2,
        g::ScopeKind::Tail => 3,
        g::ScopeKind::Condition => 4,
        g::ScopeKind::IfThen => 5,
        g::ScopeKind::Iteration => 6,
        g::ScopeKind::Arm => 7,
        g::ScopeKind::Guard => 8,
    }
}
pub(super) fn decode(read: &mut Input<'_, '_>) -> Result<g::Original, &'static str> {
    let body = BodyRef(read.id()?);
    let declaration = DeclarationRef(read.id()?);
    let result = g::TypeRef(read.id()?);
    let sites = read.id()?;
    let locals = locals_read(read)?;
    let count = read.count::<g::ParameterBinding>(crates::MAX_COMPONENTS, 12)?;
    let mut parameter_bindings = Vec::with_capacity(count);
    for _ in 0..count {
        parameter_bindings.push(g::ParameterBinding {
            declaration: DeclarationRef(read.id()?),
            input: read.id()?,
            local: read.id()?,
        });
    }
    let count = read.count::<g::Value>(crates::MAX_COMPONENTS, 5)?;
    let mut values = Vec::with_capacity(count);
    for _ in 0..count {
        values.push(super::opcodes::decode(read)?);
    }
    let count = read.count::<g::Statement>(crates::MAX_COMPONENTS, 9)?;
    let mut statements = Vec::with_capacity(count);
    for _ in 0..count {
        statements.push(g::Statement {
            site: read.id()?,
            action: action_read(read)?,
        });
    }
    let count = read.count::<g::Block>(crates::MAX_COMPONENTS, 13)?;
    let mut blocks = Vec::with_capacity(count);
    for _ in 0..count {
        let statements = read.range()?;
        let site = read.id()?;
        let terminator = match read.reader.u8()? {
            0 => g::Terminator::Return(read.id()?),
            1 => g::Terminator::Jump(read.id()?),
            2 => g::Terminator::Unreachable,
            _ => return Err("unknown generic terminator"),
        };
        blocks.push(g::Block {
            statements,
            site,
            terminator,
        });
    }
    let count = read.count::<g::Scope>(crates::MAX_COMPONENTS, 6)?;
    let mut scopes = Vec::with_capacity(count);
    for _ in 0..count {
        scopes.push(g::Scope {
            parent: read.option()?,
            kind: scope_read(read.reader.u8()?)?,
            site: read.id()?,
        });
    }
    let count = read.count::<g::Exit>(crates::MAX_COMPONENTS, 15)?;
    let mut exits = Vec::with_capacity(count);
    for _ in 0..count {
        exits.push(exit_read(read)?);
    }
    let count = read.count::<g::Temporary>(crates::MAX_COMPONENTS, 21)?;
    let mut temporaries = Vec::with_capacity(count);
    for _ in 0..count {
        let lifetime = match read.reader.u8()? {
            0 => g::Lifetime::Scope(read.id()?),
            1 => g::Lifetime::Static,
            _ => return Err("unknown generic temporary lifetime"),
        };
        temporaries.push(g::Temporary {
            lifetime,
            value: read.id()?,
            site: read.id()?,
            block: read.id()?,
            before: read.id()?,
            exits_before: read.id()?,
        });
    }
    let mut arguments = Vec::new();
    if read.version == 1 {
        let count = read.count::<u32>(crates::MAX_COMPONENTS, 4)?;
        arguments = Vec::with_capacity(count);
        for _ in 0..count {
            arguments.push(read.id()?);
        }
    }
    Ok(g::Original {
        body,
        declaration,
        result,
        sites,
        locals,
        parameter_bindings,
        values,
        statements,
        blocks,
        scopes,
        exits,
        temporaries,
        arguments,
    })
}
fn action_read(read: &mut Input<'_, '_>) -> Result<g::Action, &'static str> {
    Ok(match read.reader.u8()? {
        0 => g::Action::StorageLive(read.id()?),
        1 => g::Action::StorageDead(read.id()?),
        2 => g::Action::StorageEnd {
            first: read.id()?,
            last: read.id()?,
        },
        3 => g::Action::Evaluate(read.id()?),
        4 => g::Action::Store {
            local: read.id()?,
            value: read.id()?,
        },
        5 => g::Action::CheckInitialized(read.id()?),
        6 => g::Action::Mention(read.id()?),
        _ => return Err("unknown generic statement operation"),
    })
}
fn scope_read(value: u8) -> Result<g::ScopeKind, &'static str> {
    Ok(match value {
        0 => g::ScopeKind::Body,
        1 => g::ScopeKind::Block,
        2 => g::ScopeKind::Statement,
        3 => g::ScopeKind::Tail,
        4 => g::ScopeKind::Condition,
        5 => g::ScopeKind::IfThen,
        6 => g::ScopeKind::Iteration,
        7 => g::ScopeKind::Arm,
        8 => g::ScopeKind::Guard,
        _ => return Err("unknown generic storage scope kind"),
    })
}
fn exit_read(read: &mut Input<'_, '_>) -> Result<g::Exit, &'static str> {
    let location = match read.reader.u8()? {
        0 => g::Location::At {
            block: read.id()?,
            before: read.id()?,
        },
        1 => g::Location::Edge {
            block: read.id()?,
            target: read.id()?,
        },
        _ => return Err("unknown generic storage exit location"),
    };
    let end = match read.reader.u8()? {
        0 => g::End::Scopes {
            inner: read.id()?,
            outer: read.option()?,
        },
        1 => g::End::Temporaries(read.range()?),
        _ => return Err("unknown generic storage exit range"),
    };
    Ok(g::Exit { location, end })
}

fn locals_write(out: &mut Writer, locals: &[g::Local]) -> Result<(), &'static str> {
    out.u32(locals.len())?;
    for local in locals {
        id(out, local.scope)?;
        option(out, local.storage_parent)?;
        id(out, local.ty.0)?;
        match local.kind {
            g::LocalKind::Parameter(index) => {
                out.u8(0)?;
                id(out, index)?;
            }
            g::LocalKind::Binding => out.u8(1)?,
            g::LocalKind::Merge => out.u8(2)?,
        }
        option(out, local.definition.map(|d| d.0))?;
        out.u8(u8::from(local.mutable))?;
        id(out, local.site)?;
    }
    Ok(())
}
fn locals_read(read: &mut Input<'_, '_>) -> Result<Vec<g::Local>, &'static str> {
    let count = read.count::<g::Local>(crates::MAX_COMPONENTS, 16)?;
    let mut locals = Vec::with_capacity(count);
    for _ in 0..count {
        let scope = read.id()?;
        let storage_parent = read.option()?;
        let ty = g::TypeRef(read.id()?);
        let kind = match read.reader.u8()? {
            0 => g::LocalKind::Parameter(read.id()?),
            1 => g::LocalKind::Binding,
            2 => g::LocalKind::Merge,
            _ => return Err("unknown generic local kind"),
        };
        locals.push(g::Local {
            scope,
            storage_parent,
            ty,
            kind,
            definition: read.option()?.map(DeclarationRef),
            mutable: read.bool()?,
            site: read.id()?,
        });
    }
    Ok(locals)
}
