//! Fixed table order of generic row version zero; no enum discriminants are wire IDs.
use super::{
    BodyRef, DeclarationRef, ExternalDef, ExternalKind, Input, Writer, crates, g, id, option, range,
};
use crate::support::{crates::CrateIdentity, edition::Edition};

pub(super) fn encode(out: &mut Writer, rows: &g::Rows) -> Result<(), &'static str> {
    out.u32(rows.declarations.len())?;
    for declaration in &rows.declarations {
        declaration_write(out, declaration)?;
    }
    for &sites in &rows.declaration_sites {
        id(out, sites)?;
    }
    out.u32(rows.binders.len())?;
    for binder in &rows.binders {
        id(out, binder.owner.0)?;
        option(out, binder.parent.map(|p| p.0))?;
        out.u8(match binder.kind {
            g::BinderKind::OriginalFunction => 0,
            g::BinderKind::PublicFunction => 1,
            g::BinderKind::Clause => 2,
        })?;
        id(out, binder.count)?;
    }
    out.u32(rows.types.len())?;
    for &ty in &rows.types {
        type_write(out, ty, rows.calls.is_some())?;
    }
    out.u32(rows.roots.len())?;
    for root in &rows.roots {
        id(out, root.0)?;
    }
    out.u32(rows.parameters.len())?;
    for parameter in &rows.parameters {
        id(out, parameter.0)?;
    }
    out.u32(rows.clauses.len())?;
    for clause in &rows.clauses {
        id(out, clause.owner.0)?;
        id(out, clause.site)?;
        id(out, clause.subject.0)?;
        option(out, clause.quantifier.map(|q| q.0))?;
        match clause.bound {
            g::Bound::Sized => out.u8(0)?,
            g::Bound::Copy(g::CopyIdentity::LegacyBuiltin) => out.bytes(&[1, 0])?,
            g::Bound::Copy(g::CopyIdentity::Source { .. }) => {
                return Err("source language roles require a later generic row version");
            }
        }
    }
    out.u32(rows.functions.len())?;
    for function in &rows.functions {
        for value in [
            function.declaration.0,
            function.body.0,
            function.original.0,
            function.callable.0,
            function.original_frame.0,
            function.callable_frame.0,
        ] {
            id(out, value)?;
        }
        range(out, function.parameters)?;
        range(out, function.clauses)?;
        id(out, function.declared_clauses)?;
        out.u8(u8::from(function.constant))?;
    }
    Ok(())
}
fn declaration_write(out: &mut Writer, declaration: &g::Declaration) -> Result<(), &'static str> {
    match declaration {
        g::Declaration::Function { identity, public } => {
            out.u8(0)?;
            out.path(identity.path())?;
            out.u8(u8::from(*public))
        }
        g::Declaration::TypeParameter {
            owner,
            ordinal,
            name,
            sized,
        } => {
            out.u8(1)?;
            id(out, owner.0)?;
            id(out, *ordinal)?;
            out.string(name)?;
            out.u8(u8::from(*sized))
        }
        g::Declaration::ValueParameter {
            body,
            ordinal,
            name,
        }
        | g::Declaration::Local {
            body,
            ordinal,
            name,
        } => {
            out.u8(
                if matches!(declaration, g::Declaration::ValueParameter { .. }) {
                    2
                } else {
                    3
                },
            )?;
            id(out, body.0)?;
            id(out, *ordinal)?;
            out.string(name)
        }
    }
}
fn type_write(out: &mut Writer, ty: g::Type, calls: bool) -> Result<(), &'static str> {
    match ty {
        g::Type::Scalar(scalar) => out.bytes(&[0, scalar as u8]),
        g::Type::Parameter(declaration) => {
            out.u8(1)?;
            id(out, declaration.0)
        }
        g::Type::Function(signature) => {
            out.u8(2)?;
            id(out, signature.binder.0)?;
            range(out, signature.parameters)?;
            id(out, signature.result.0)?;
            out.u8(u8::from(signature.unsafe_))?;
            out.u8(match signature.convention {
                g::Convention::Rust => 0,
                g::Convention::RustCall => 1,
                g::Convention::C => 2,
            })?;
            out.u8(u8::from(signature.variadic))
        }
        g::Type::FunctionItem(application) | g::Type::ApplicationSignature(application) => {
            if !calls {
                return Err("generic application types require row version one");
            }
            out.u8(if matches!(ty, g::Type::FunctionItem(_)) {
                3
            } else {
                4
            })?;
            id(out, application.0)
        }
    }
}
pub(super) fn decode(
    read: &mut Input<'_, '_>,
    owner: &CrateIdentity,
) -> Result<g::Rows, &'static str> {
    let count = read.count::<g::Declaration>(crates::MAX_FUNCTIONS, 9)?;
    let mut declarations = Vec::with_capacity(count);
    for _ in 0..count {
        declarations.push(declaration_read(read, owner)?);
    }
    read.fixed::<u32>(count, 4)?;
    let mut declaration_sites = Vec::with_capacity(count);
    for _ in 0..count {
        declaration_sites.push(read.id()?);
    }
    let count = read.count::<g::Binder>(crates::MAX_COMPONENTS, 10)?;
    let mut binders = Vec::with_capacity(count);
    for _ in 0..count {
        let owner = DeclarationRef(read.id()?);
        let parent = read.option()?.map(g::BinderRef);
        let kind = match read.reader.u8()? {
            0 => g::BinderKind::OriginalFunction,
            1 => g::BinderKind::PublicFunction,
            2 => g::BinderKind::Clause,
            _ => return Err("unknown generic binder kind"),
        };
        binders.push(g::Binder {
            owner,
            parent,
            kind,
            count: read.id()?,
        });
    }
    let count = read.count::<g::Type>(crates::MAX_COMPONENTS, 2)?;
    let mut types = Vec::with_capacity(count);
    for _ in 0..count {
        types.push(type_read(read)?);
    }
    let count = read.count::<g::TypeRef>(crates::MAX_COMPONENTS, 4)?;
    let mut roots = Vec::with_capacity(count);
    for _ in 0..count {
        roots.push(g::TypeRef(read.id()?));
    }
    let count = read.count::<DeclarationRef>(crates::MAX_COMPONENTS, 4)?;
    let mut parameters = Vec::with_capacity(count);
    for _ in 0..count {
        parameters.push(DeclarationRef(read.id()?));
    }
    let count = read.count::<g::Clause>(crates::MAX_COMPONENTS, 14)?;
    let mut clauses = Vec::with_capacity(count);
    for _ in 0..count {
        clauses.push(clause(read)?);
    }
    let count = read.count::<g::Function>(crates::MAX_FUNCTIONS, 45)?;
    let mut functions = Vec::with_capacity(count);
    for _ in 0..count {
        functions.push(g::Function {
            declaration: DeclarationRef(read.id()?),
            body: BodyRef(read.id()?),
            original: g::TypeRef(read.id()?),
            callable: g::TypeRef(read.id()?),
            original_frame: g::BinderRef(read.id()?),
            callable_frame: g::BinderRef(read.id()?),
            parameters: read.range()?,
            clauses: read.range()?,
            declared_clauses: read.id()?,
            constant: read.bool()?,
        });
    }
    Ok(g::Rows {
        edition: Edition::Rust2024,
        declarations,
        declaration_sites,
        binders,
        types,
        roots,
        parameters,
        clauses,
        functions,
        originals: Vec::new(),
        calls: None,
    })
}
fn declaration_read(
    read: &mut Input<'_, '_>,
    owner: &CrateIdentity,
) -> Result<g::Declaration, &'static str> {
    match read.reader.u8()? {
        0 => Ok(g::Declaration::Function {
            identity: ExternalDef::new(owner.clone(), read.path()?, ExternalKind::Function)?,
            public: read.bool()?,
        }),
        1 => Ok(g::Declaration::TypeParameter {
            owner: DeclarationRef(read.id()?),
            ordinal: read.id()?,
            name: read.string()?,
            sized: read.bool()?,
        }),
        kind @ (2 | 3) => {
            let body = BodyRef(read.id()?);
            let ordinal = read.id()?;
            let name = read.string()?;
            Ok(if kind == 2 {
                g::Declaration::ValueParameter {
                    body,
                    ordinal,
                    name,
                }
            } else {
                g::Declaration::Local {
                    body,
                    ordinal,
                    name,
                }
            })
        }
        _ => Err("unknown generic declaration kind"),
    }
}
fn type_read(read: &mut Input<'_, '_>) -> Result<g::Type, &'static str> {
    match read.reader.u8()? {
        0 => Ok(g::Type::Scalar(read.reader.scalar()?)),
        1 => Ok(g::Type::Parameter(DeclarationRef(read.id()?))),
        3 if read.version == 1 => Ok(g::Type::FunctionItem(g::ApplicationRef(read.id()?))),
        4 if read.version == 1 => Ok(g::Type::ApplicationSignature(g::ApplicationRef(read.id()?))),
        2 => {
            let binder = g::BinderRef(read.id()?);
            let parameters = read.range()?;
            let result = g::TypeRef(read.id()?);
            let unsafe_ = read.bool()?;
            let convention = match read.reader.u8()? {
                0 => g::Convention::Rust,
                1 => g::Convention::RustCall,
                2 => g::Convention::C,
                _ => return Err("unknown generic calling convention"),
            };
            Ok(g::Type::Function(g::Signature {
                binder,
                parameters,
                result,
                unsafe_,
                convention,
                variadic: read.bool()?,
            }))
        }
        _ => Err("unknown generic type node"),
    }
}
fn clause(read: &mut Input<'_, '_>) -> Result<g::Clause, &'static str> {
    let owner = DeclarationRef(read.id()?);
    let site = read.id()?;
    let subject = g::TypeRef(read.id()?);
    let quantifier = read.option()?.map(g::BinderRef);
    let bound = match read.reader.u8()? {
        0 => g::Bound::Sized,
        1 => match read.reader.u8()? {
            0 => g::Bound::Copy(g::CopyIdentity::LegacyBuiltin),
            _ => return Err("source language roles require a later generic row version"),
        },
        _ => return Err("unknown generic input bound"),
    };
    Ok(g::Clause {
        owner,
        site,
        subject,
        quantifier,
        bound,
    })
}
