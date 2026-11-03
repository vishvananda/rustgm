//! Diagnostic spelling traverses completed types and resolved names, never source text.
use crate::{
    resolve::expansion::Report,
    types::{Kind, Store, TypeId, arguments::Argument},
};
use std::fmt::Write;
enum Part {
    Type(TypeId),
    Text(&'static str),
    Number(u128),
    Name(u32),
}
pub(super) fn name(report: &Report, types: &Store, root: TypeId) -> Result<String, &'static str> {
    let mut pending = vec![Part::Type(root)];
    let mut output = String::new();
    let mut work = 0usize;
    while let Some(part) = pending.pop() {
        work = work.saturating_add(1);
        if work > 1_000_000 || output.len() > 1_048_576 || pending.len() > 1_000_000 {
            return Err("source panic type spelling exceeds its bounded diagnostic budget");
        }
        output
            .try_reserve(64)
            .map_err(|_| "source panic type spelling allocation failed")?;
        match part {
            Part::Text(text) => output.push_str(text),
            Part::Number(n) => {
                write!(output, "{n}").map_err(|_| "source panic numeric spelling failed")?;
            }
            Part::Name(item) => nominal(report, item, &mut output)?,
            Part::Type(ty) => expand(types, ty, &mut pending)?,
        }
    }
    Ok(output)
}
fn nominal(report: &Report, item: u32, output: &mut String) -> Result<(), &'static str> {
    let definition = report
        .resolution
        .definitions
        .get(item as usize)
        .ok_or("source panic type lost its resolved nominal identity")?;
    let scope = report
        .resolution
        .scopes
        .get(definition.scope.0 as usize)
        .ok_or("source panic type lost its declaration scope")?;
    let mut module = scope.module;
    let mut names = Vec::new();
    loop {
        if names.len() > report.package.modules.len() {
            return Err("cyclic source panic type module path");
        }
        let entry = report
            .package
            .modules
            .get(module.0 as usize)
            .ok_or("source panic type has an invalid module path")?;
        let Some(parent) = entry.parent else {
            break;
        };
        names.push(entry.name);
        module = parent;
    }
    for name in names.into_iter().rev() {
        output.push_str(report.package.names.get(name));
        output.push_str("::");
    }
    output.push_str(report.package.names.get(definition.name));
    Ok(())
}
fn expand(types: &Store, ty: TypeId, pending: &mut Vec<Part>) -> Result<(), &'static str> {
    pending
        .try_reserve(8)
        .map_err(|_| "source panic type traversal allocation failed")?;
    match types
        .kind(ty)
        .ok_or("source panic lost its completed element type")?
    {
        Kind::Never => pending.push(Part::Text("!")),
        Kind::Primitive(p) => pending.push(Part::Text(p.name())),
        Kind::Pattern { base, .. } => pending.push(Part::Type(base)),
        Kind::Array { element, length } => {
            pending.extend([
                Part::Text("]"),
                Part::Number(u128::from(length)),
                Part::Text("; "),
                Part::Type(element),
                Part::Text("["),
            ]);
        }
        Kind::Slice(element) => {
            pending.extend([Part::Text("]"), Part::Type(element), Part::Text("[")]);
        }
        Kind::Reference {
            referent, mutable, ..
        } => pending.extend([
            Part::Type(referent),
            Part::Text(if mutable { "&mut " } else { "&" }),
        ]),
        Kind::Pointer { pointee, mutable } => pending.extend([
            Part::Type(pointee),
            Part::Text(if mutable { "*mut " } else { "*const " }),
        ]),
        Kind::Tuple(list) => {
            let children = types
                .list(list)
                .ok_or("source panic lost its tuple fields")?;
            pending
                .try_reserve(children.len().saturating_mul(2).saturating_add(3))
                .map_err(|_| "source panic tuple spelling allocation failed")?;
            pending.push(Part::Text(")"));
            if children.len() == 1 {
                pending.push(Part::Text(","));
            }
            for (index, &child) in children.iter().enumerate().rev() {
                if index + 1 != children.len() {
                    pending.push(Part::Text(", "));
                }
                pending.push(Part::Type(child));
            }
            pending.push(Part::Text("("));
        }
        Kind::Adt(item, args) => {
            arguments(types, args, pending)?;
            pending.push(Part::Name(item.0));
        }
        Kind::Foreign(item) => pending.push(Part::Name(item.0)),
        _ => {
            return Err(
                "source panic type spelling requires its complete non-storage type profile",
            );
        }
    }
    Ok(())
}
fn arguments(
    types: &Store,
    args: crate::types::GenericArgs,
    pending: &mut Vec<Part>,
) -> Result<(), &'static str> {
    let arguments = types
        .arguments(args)
        .ok_or("source panic lost nominal arguments")?;
    let count = arguments.len();
    if count == 0 {
        return Ok(());
    }
    pending
        .try_reserve(count.saturating_mul(2).saturating_add(2))
        .map_err(|_| "source panic argument spelling allocation failed")?;
    // Arguments use declaration order; the traversal stack emits them in reverse.
    let mut parts = Vec::new();
    for argument in arguments {
        parts.push(match argument {
            Argument::Type(ty) => Part::Type(ty),
            Argument::Lifetime(region) => Part::Text(if region == crate::types::RegionId::STATIC {
                "'static"
            } else {
                "'_"
            }),
            Argument::Const(id) => Part::Number(
                types
                    .constant(id)
                    .ok_or("source panic lost its evaluated const argument")?
                    .bits,
            ),
        });
    }
    pending.push(Part::Text(">"));
    for (index, part) in parts.into_iter().enumerate().rev() {
        if index + 1 != count {
            pending.push(Part::Text(", "));
        }
        pending.push(part);
    }
    pending.push(Part::Text("<"));
    Ok(())
}
