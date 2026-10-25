//! Item/type/pattern spellings extend the PA4 expression vocabulary.
use super::{CallableAbi, Children, Kind, Optional, Syntax, Write, io, label, spelling};
use crate::syntax::item::{Item, ItemKind, Visibility};

fn visibility(visibility: Visibility, output: &mut impl Write) -> io::Result<()> {
    match visibility {
        Visibility::Private => write!(output, "private"),
        Visibility::Public => write!(output, "public"),
        Visibility::Restricted(path) => write!(output, "restricted:{}", path.get()),
    }
}

#[expect(
    clippy::too_many_lines,
    reason = "one exhaustive extension-vocabulary dispatch under the architecture audit limit"
)]
pub(super) fn node<L>(
    source: &str,
    ast: &Syntax<L>,
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
            receiver.get(),
            method.get(),
            Children(ast, arguments)
        ),
        Kind::Variadic => write!(output, "variadic"),
        Kind::Crate(items) => write!(output, "crate {}", Children(ast, items)),
        Kind::Attributed { attributes, value } => write!(
            output,
            "attributed {} {} {}",
            attributes.start,
            attributes.end,
            value.get()
        ),
        Kind::Applied {
            base,
            arguments,
            output: result,
            ..
        } => write!(
            output,
            "applied {} {} {}",
            base.get(),
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
            ty.get(),
            Optional(trait_path),
            path.get()
        ),
        Kind::Constraint {
            target,
            value,
            bounds,
        } => write!(
            output,
            "constraint {} {} {}",
            target.get(),
            Optional(value),
            Children(ast, bounds)
        ),
        Kind::Bound {
            optional,
            constness,
            binders,
            value,
            ..
        } => {
            write!(output, "bound {optional} ")?;
            if constness != crate::support::constness::Constness::Ordinary {
                write!(output, "{constness:?} ")?;
            }
            write!(output, "{} {}", Children(ast, binders), value.get())
        }
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
            ..
        } => write!(
            output,
            "where {} {} {}",
            Children(ast, binders),
            ty.get(),
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
                CallableAbi(abi),
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
            write!(output, " {}", ty.get())
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
                "use-tree {} {} {} ",
                Optional(prefix),
                Children(ast, children),
                glob.is_some()
            )?;
            label(source, rename.map(|id| ast.node(id).span), output)
        }
        Kind::StructPattern { path, fields, rest } => write!(
            output,
            "struct-pattern {} {} {rest}",
            path.get(),
            Children(ast, fields)
        ),
        Kind::PatternField { name, pattern } => {
            write!(output, "pattern-field ")?;
            spelling(source, name, output)?;
            write!(output, " {}", pattern.get())
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

#[expect(
    clippy::too_many_lines,
    reason = "one exhaustive item-vocabulary dispatch preserves the existing ordinary rendering"
)]
pub(super) fn item<L>(
    source: &str,
    ast: &Syntax<L>,
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
            CallableAbi(abi),
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
            restriction,
            auto_,
            unsafe_,
            constant,
            bounds,
            items,
        } => {
            if let Some(clause) = restriction {
                write!(output, "implementation-domain={} ", clause.get())?;
            }
            write!(
                output,
                "trait {unsafe_} {}{}{} {}",
                if auto_ { "auto " } else { "" },
                if constant { "const " } else { "" },
                Children(ast, bounds),
                Children(ast, items)
            )
        }
        ItemKind::Impl {
            unsafe_,
            constant,
            negative,
            trait_path,
            ty,
            items,
        } => write!(
            output,
            "impl {unsafe_} {negative} {}{} {} {}",
            if constant { "const " } else { "" },
            Optional(trait_path),
            ty.get(),
            Children(ast, items)
        ),
        ItemKind::Const { ty, value } => write!(output, "const {} {}", ty.get(), Optional(value)),
        ItemKind::Static {
            safety,
            mutable,
            ty,
            value,
        } => {
            write!(output, "static ")?;
            static_safety(safety, output)?;
            write!(output, "{mutable} {} {}", ty.get(), Optional(value))
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
        ItemKind::Use(tree) => write!(output, "use {}", tree.get()),
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
        ItemKind::DeclarativeMacro {
            parameters,
            capture,
        } => {
            write!(output, "declarative-macro ")?;
            if let Some(parameters) = parameters {
                write!(output, "{parameters}")?;
            } else {
                write!(output, "-")?;
            }
            write!(output, " {capture}")
        }
        ItemKind::MacroInvocation(value) => write!(output, "macro-invocation {}", value.get()),
    }
}

fn static_safety(safety: crate::syntax::item::Safety, output: &mut impl Write) -> io::Result<()> {
    match safety {
        crate::syntax::item::Safety::Default => Ok(()),
        crate::syntax::item::Safety::Safe => write!(output, "safe "),
        crate::syntax::item::Safety::Unsafe => write!(output, "unsafe "),
    }
}
