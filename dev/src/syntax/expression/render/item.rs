//! Item/type/pattern spellings extend the PA4 expression vocabulary.
use super::{Ast, Children, Kind, Optional, Write, io, label, spelling};
use crate::syntax::item::{Item, ItemKind, Visibility};

fn visibility(visibility: Visibility, output: &mut impl Write) -> io::Result<()> {
    match visibility {
        Visibility::Private => write!(output, "private"),
        Visibility::Public => write!(output, "public"),
        Visibility::Restricted(path) => write!(output, "restricted:{}", path.0),
    }
}

#[expect(
    clippy::too_many_lines,
    reason = "one exhaustive extension-vocabulary dispatch under the architecture audit limit"
)]
pub(super) fn node(
    source: &str,
    ast: &Ast<'_>,
    kind: Kind,
    output: &mut impl Write,
) -> io::Result<()> {
    match kind {
        Kind::PreciseCapture(parameters) => {
            write!(output, "precise-capture {}", Children(ast, parameters))
        }
        Kind::Receiver {
            lifetime, mutable, ..
        } => {
            write!(output, "receiver {mutable} ")?;
            spelling(source, lifetime, output)
        }
        Kind::GenericMethodCall {
            receiver,
            method,
            arguments,
        } => write!(
            output,
            "generic-method-call {} {} {}",
            receiver.0,
            method.0,
            Children(ast, arguments)
        ),
        Kind::Variadic => write!(output, "variadic"),
        Kind::Crate(items) => write!(output, "crate {}", Children(ast, items)),
        Kind::Attributed { attributes, value } => write!(
            output,
            "attributed {} {} {}",
            attributes.start, attributes.end, value.0
        ),
        Kind::Applied {
            base,
            arguments,
            output: result,
        } => write!(
            output,
            "applied {} {} {}",
            base.0,
            Children(ast, arguments),
            Optional(result)
        ),
        Kind::Qualified {
            ty,
            trait_path,
            path,
        } => write!(
            output,
            "qualified {} {} {}",
            ty.0,
            Optional(trait_path),
            path.0
        ),
        Kind::Constraint {
            target,
            value,
            bounds,
        } => write!(
            output,
            "constraint {} {} {}",
            target.0,
            Optional(value),
            Children(ast, bounds)
        ),
        Kind::Bound {
            optional,
            binders,
            value,
        } => write!(
            output,
            "bound {optional} {} {}",
            Children(ast, binders),
            value.0
        ),
        Kind::Generic(index) => {
            let crate::syntax::item::GenericParameter {
                flavor,
                name,
                ty,
                bounds,
                default,
            } = ast.generics[index as usize];
            write!(output, "generic {flavor:?} ")?;
            spelling(source, name, output)?;
            write!(
                output,
                " {} {} {}",
                Optional(ty),
                Children(ast, bounds),
                Optional(default)
            )
        }
        Kind::Where {
            binders,
            ty,
            bounds,
        } => write!(
            output,
            "where {} {} {}",
            Children(ast, binders),
            ty.0,
            Children(ast, bounds)
        ),
        Kind::FunctionType(index) => {
            let crate::syntax::item::FunctionType {
                unsafe_,
                abi,
                binders,
                parameters,
                result,
            } = ast.function_types[index as usize];
            write!(
                output,
                "function-type {unsafe_} {} {} {} {}",
                Optional(abi),
                Children(ast, binders),
                Children(ast, parameters),
                Optional(result)
            )
        }
        Kind::TraitType { opaque, bounds } => {
            write!(output, "trait-type {opaque} {}", Children(ast, bounds))
        }
        Kind::FieldDefinition {
            name,
            visibility: vis,
            ty,
        } => {
            write!(output, "field-definition ")?;
            label(source, name, output)?;
            write!(output, " ")?;
            visibility(vis, output)?;
            write!(output, " {}", ty.0)
        }
        Kind::Variant(index) => {
            let crate::syntax::item::Variant {
                name,
                shape,
                fields,
                discriminant,
            } = ast.variants[index as usize];
            write!(output, "variant ")?;
            spelling(source, name, output)?;
            write!(
                output,
                " {shape:?} {} {}",
                Children(ast, fields),
                Optional(discriminant)
            )
        }
        Kind::UseTree(index) => {
            let crate::syntax::item::UseTree {
                prefix,
                children,
                rename,
                glob,
            } = ast.use_trees[index as usize];
            write!(
                output,
                "use-tree {} {} {glob} ",
                Optional(prefix),
                Children(ast, children)
            )?;
            label(source, rename.map(|id| ast.node(id).span), output)
        }
        Kind::StructPattern { path, fields, rest } => write!(
            output,
            "struct-pattern {} {} {rest}",
            path.0,
            Children(ast, fields)
        ),
        Kind::PatternField { name, pattern } => {
            write!(output, "pattern-field ")?;
            spelling(source, name, output)?;
            write!(output, " {}", pattern.0)
        }
        Kind::PatternRange {
            start,
            end,
            inclusive,
        } => write!(
            output,
            "pattern-range {} {} {inclusive}",
            Optional(start),
            Optional(end)
        ),
        _ => Err(io::Error::other("expected item/type/pattern node")),
    }
}

pub(super) fn item(
    source: &str,
    ast: &Ast<'_>,
    item: &Item,
    output: &mut impl Write,
) -> io::Result<()> {
    label(source, item.name, output)?;
    write!(output, " ")?;
    visibility(item.visibility, output)?;
    write!(
        output,
        " {} {} ",
        Children(ast, item.generics),
        Children(ast, item.where_clause)
    )?;
    match item.kind {
        ItemKind::Function {
            flags,
            abi,
            parameters,
            result,
            body,
        } => write!(
            output,
            "function {} {} {} {} {} {} {} {}",
            flags.constant,
            flags.asynchronous,
            flags.safety == crate::syntax::item::Safety::Unsafe,
            flags.safety == crate::syntax::item::Safety::Safe,
            Optional(abi),
            Children(ast, parameters),
            Optional(result),
            Optional(body)
        ),
        ItemKind::Struct { shape, fields } => {
            write!(output, "struct {shape:?} {}", Children(ast, fields))
        }
        ItemKind::Enum { variants } => write!(output, "enum {}", Children(ast, variants)),
        ItemKind::Union { fields } => write!(output, "union {}", Children(ast, fields)),
        ItemKind::Trait {
            unsafe_,
            bounds,
            items,
        } => write!(
            output,
            "trait {unsafe_} {} {}",
            Children(ast, bounds),
            Children(ast, items)
        ),
        ItemKind::Impl {
            unsafe_,
            negative,
            trait_path,
            ty,
            items,
        } => write!(
            output,
            "impl {unsafe_} {negative} {} {} {}",
            Optional(trait_path),
            ty.0,
            Children(ast, items)
        ),
        ItemKind::Const { ty, value } => write!(output, "const {} {}", ty.0, Optional(value)),
        ItemKind::Static { mutable, ty, value } => {
            write!(output, "static {mutable} {} {}", ty.0, Optional(value))
        }
        ItemKind::TypeAlias { bounds, ty } => write!(
            output,
            "type-alias {} {}",
            Children(ast, bounds),
            Optional(ty)
        ),
        ItemKind::Module { items } => {
            write!(output, "module ")?;
            if let Some(items) = items {
                write!(output, "{}", Children(ast, items))
            } else {
                write!(output, "external")
            }
        }
        ItemKind::Use(tree) => write!(output, "use {}", tree.0),
        ItemKind::ExternCrate { rename } => {
            write!(output, "extern-crate ")?;
            label(source, rename, output)
        }
        ItemKind::ExternBlock {
            unsafe_,
            abi,
            items,
        } => write!(
            output,
            "extern-block {unsafe_} {} {}",
            Optional(abi),
            Children(ast, items)
        ),
        ItemKind::MacroRules { capture } => write!(output, "macro-rules {capture}"),
        ItemKind::MacroInvocation(value) => write!(output, "macro-invocation {}", value.0),
    }
}
