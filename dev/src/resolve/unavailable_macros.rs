//! Scoped declarations publish real items while their execution stays unavailable.
use super::{Context, Resolver};
use crate::{
    expand::{macros::compile, marks::Transparency},
    resolve::{
        bindings::panic_contracts::{self, HeaderData},
        model::{Access, DefKind, Namespace},
        package::{Error, Package},
        unit::{NodeKey, Unit, editions},
    },
    support::edition::Edition,
    syntax::{
        ast::{Kind, NodeId},
        attribute::{Attribute, Input},
        item::{Item, ItemKind},
        token_tree::Forest,
    },
};
use std::rc::Rc;
#[cfg(test)]
#[path = "../../tests/invariants/unavailable_declarations.rs"]
pub(in crate::resolve) mod observation;

pub(in crate::resolve) struct ScopedDeclaration {
    pub mode: Transparency,
    pub edition: Edition,
    pub header: Option<Rc<HeaderData>>,
}

impl Resolver<'_> {
    pub(in crate::resolve) fn scoped_declared(
        &self,
        context: Context,
        node: NodeId,
    ) -> Result<(), Error> {
        let key = NodeKey {
            file: context.file,
            node,
        };
        let actual = self
            .facts
            .item_definitions
            .get(&(context.module, node, Namespace::Macro))
            .and_then(|id| self.facts.definitions.get(id.0 as usize));
        if actual.is_none_or(|definition| {
            definition.kind != DefKind::Macro
                || definition.node != Some(key)
                || definition.scope != context.scope
        }) {
            return Err(self.error(context, node, "scoped body has no validated declaration"));
        }
        Ok(())
    }
    pub(in crate::resolve) fn declare_scoped(
        &mut self,
        context: Context,
        node: NodeId,
        item: Item,
        access: Access,
        owner: Option<NodeId>,
    ) -> Result<(Context, ScopedDeclaration), Error> {
        let key = NodeKey {
            file: context.file,
            node,
        };
        let (parameters, capture, span) = self.scoped_syntax(context, node, item)?;
        let mode = declaration_attributes(self.package, key, owner)?;
        let checked = panic_contracts::checked(
            self.package,
            NodeKey {
                node: owner.unwrap_or(node),
                ..key
            },
        )?;
        let (header, header_edition) = if let Some(checked) = checked {
            let summary = checked.summary();
            if summary.node != key {
                return Err(self.error(context, node, "scoped header names another declaration"));
            }
            (checked.into_data(), Some(summary.edition))
        } else {
            (None, None)
        };
        let edition = if let Some(edition) = header_edition {
            edition
        } else {
            editions::introducer(context.file, span, |file| {
                self.package.units.get(file.0 as usize)
            })
            .map_err(|failure| Error::at(context.file, failure.offset, failure.message))?
        };
        self.compile_scoped(
            context,
            node,
            (parameters, capture),
            edition,
            #[cfg(test)]
            owner,
        )?;
        self.item_definition(
            context,
            node,
            item,
            (Namespace::Macro, DefKind::Macro),
            access,
            None,
        )?;
        self.facts
            .node_scopes
            .insert(context.module, context.file, node, context.scope);
        #[cfg(test)]
        observation::note(observation::Event::Published {
            module: context.module,
            key,
        });
        Ok((
            context,
            ScopedDeclaration {
                mode,
                edition,
                header,
            },
        ))
    }
    fn compile_scoped(
        &mut self,
        context: Context,
        node: NodeId,
        grammar: (Option<u32>, u32),
        edition: Edition,
        #[cfg(test)] owner: Option<NodeId>,
    ) -> Result<(), Error> {
        let (parameters, capture) = grammar;
        let unit = &self.package.units[context.file.0 as usize];
        let body = unit
            .configured
            .ast
            .captures
            .get(capture as usize)
            .ok_or_else(|| self.error(context, node, "missing scoped macro body forest"))?;
        let parameters = parameters
            .map(|index| {
                unit.configured
                    .ast
                    .captures
                    .get(index as usize)
                    .ok_or_else(|| {
                        self.error(context, node, "missing scoped macro parameter forest")
                    })
            })
            .transpose()?;
        // Validation happens once in the common producer. No executable matcher
        // is retained by the declaration-only marker.
        #[cfg(test)]
        observation::note(observation::Event::Compile {
            module: context.module,
            key: NodeKey {
                file: context.file,
                node,
            },
            owner,
            trees: body.nodes().len() + parameters.map_or(0, |forest| forest.nodes().len()),
        });
        compile::declarative_in_edition(
            unit.source.text(),
            parameters,
            body,
            &mut self.package.names,
            edition,
        )
        .map_err(|failure| Error::at(context.file, failure.offset, failure.message))?;
        Ok(())
    }
    fn scoped_syntax(
        &self,
        context: Context,
        node: NodeId,
        item: Item,
    ) -> Result<(Option<u32>, u32, crate::syntax::token_tree::Span), Error> {
        if self
            .package
            .modules
            .get(context.module.0 as usize)
            .is_none_or(|module| module.file != context.file)
            || self
                .facts
                .scopes
                .get(context.scope.0 as usize)
                .is_none_or(|scope| scope.module != context.module)
        {
            return Err(Error::at(
                context.file,
                0,
                "scoped declaration has a foreign context",
            ));
        }
        let unit = self
            .package
            .units
            .get(context.file.0 as usize)
            .ok_or_else(|| Error::at(context.file, 0, "unknown scoped declaration file"))?;
        let actual = unit
            .configured
            .ast
            .nodes
            .get(node.get() as usize)
            .ok_or_else(|| Error::at(context.file, 0, "unknown scoped macro item"))?;
        let Kind::Item(index) = actual.kind else {
            return Err(self.error(context, node, "scoped declaration is not an actual item"));
        };
        if !unit.configured.active[node.get() as usize]
            || unit.configured.ast.items[index as usize] != item
            || context.foreign.is_some()
        {
            return Err(self.error(context, node, "scoped declaration item or owner changed"));
        }
        let ItemKind::DeclarativeMacro {
            parameters,
            capture,
        } = item.kind
        else {
            return Err(self.error(
                context,
                node,
                "scoped declaration requires its actual grammar",
            ));
        };
        self.package
            .macro_features
            .require_scoped(context.file, actual.span.start)?;
        Ok((parameters, capture, actual.span))
    }
}

fn owner_range(
    package: &Package,
    key: NodeKey,
    owner: Option<NodeId>,
) -> Result<std::ops::Range<usize>, Error> {
    let unit = package.unit(key.file);
    let Some(root) = owner else {
        if unit.owner_indices[key.node.get() as usize].is_some() {
            return Err(Error::at(
                key.file,
                0,
                "scoped declaration is missing its configured owner",
            ));
        }
        return Ok(0..0);
    };
    let actual = unit
        .owner_indices
        .get(root.get() as usize)
        .copied()
        .flatten()
        .and_then(|index| unit.configured.owners.get(index.get() as usize - 1))
        .filter(|owner| owner.enabled && owner.node == root)
        .ok_or_else(|| Error::at(key.file, 0, "unknown scoped declaration attribute owner"))?;
    let mut current = root;
    for _ in 0..=unit.configured.ast.nodes.len() {
        let node = unit
            .configured
            .ast
            .nodes
            .get(current.get() as usize)
            .ok_or_else(|| Error::at(key.file, 0, "invalid scoped declaration owner chain"))?;
        if let Kind::Attributed { value, .. } = node.kind {
            current = value;
        } else {
            if current != key.node {
                return Err(Error::at(
                    key.file,
                    node.span.start,
                    "scoped declaration attributes name another item",
                ));
            }
            return Ok(actual.attributes.clone());
        }
    }
    Err(Error::at(
        key.file,
        0,
        "cyclic scoped declaration owner chain",
    ))
}

fn declaration_attributes(
    package: &Package,
    key: NodeKey,
    owner: Option<NodeId>,
) -> Result<Transparency, Error> {
    let unit = package.unit(key.file);
    let mut mode = None;
    for (attribute, forest) in unit.retained_attributes(owner_range(package, key, owner)?) {
        match attribute.name(forest, unit.source.text()) {
            Some("rustc_macro_transparency") => {
                if mode.is_some() {
                    return Err(Error::at(
                        key.file,
                        attribute.span.start,
                        "multiple macro transparency attributes",
                    ));
                }
                mode = Some(transparency(package, unit, key, attribute, forest)?);
            }
            Some("rustc_builtin_macro") => {
                return Err(Error::at(
                    key.file,
                    attribute.span.start,
                    "scoped builtin macro declarations are not implemented",
                ));
            }
            Some("macro_export" | "macro_use") => {
                return Err(Error::at(
                    key.file,
                    attribute.span.start,
                    "legacy publication attributes on scoped macros are not implemented",
                ));
            }
            _ => {}
        }
    }
    Ok(mode.unwrap_or(Transparency::Opaque))
}

pub(in crate::resolve) fn transparency(
    package: &Package,
    unit: &Unit,
    key: NodeKey,
    attribute: Attribute,
    forest: &Forest,
) -> Result<Transparency, Error> {
    let error = |message| Error::at(key.file, attribute.span.start, message);
    package
        .macro_features
        .require_internal(key.file, attribute.span.start)?;
    if attribute.unsafe_wrapper {
        return Err(error(
            "macro transparency cannot use an unsafe attribute wrapper",
        ));
    }
    let Kind::Item(index) = unit.configured.ast.node(key.node).kind else {
        return Err(error("macro transparency requires a macro declaration"));
    };
    if !matches!(
        unit.configured.ast.items[index as usize].kind,
        ItemKind::MacroRules { .. } | ItemKind::DeclarativeMacro { .. }
    ) {
        return Err(error("macro transparency requires a macro declaration"));
    }
    let Input::Equals(range) = attribute.input else {
        return Err(error("macro transparency requires one string literal"));
    };
    if range.end != range.start + 1 {
        return Err(error("macro transparency requires one string literal"));
    }
    let token = forest
        .token(range.start)
        .ok_or_else(|| error("macro transparency requires one string literal"))?;
    let Some(crate::literal::Value::String(bytes)) =
        crate::literal::interpret(unit.source.text(), token)
            .map_err(|failure| Error::at(key.file, failure.offset, failure.message))?
    else {
        return Err(error("macro transparency requires one string literal"));
    };
    let mode = match bytes.as_ref() {
        b"transparent" => Transparency::Transparent,
        b"semiopaque" => Transparency::SemiOpaque,
        b"opaque" => Transparency::Opaque,
        _ => return Err(error("unknown macro transparency mode")),
    };
    if matches!(
        unit.configured.ast.items[index as usize].kind,
        ItemKind::MacroRules { .. }
    ) && mode != Transparency::SemiOpaque
    {
        return Err(error(
            "legacy macro transparency overrides are not implemented",
        ));
    }
    Ok(mode)
}

pub(in crate::resolve) fn transparency_on_owner(
    package: &Package,
    file: crate::resolve::unit::FileId,
    owner: NodeId,
    attribute: Attribute,
    forest: &Forest,
    seen: &mut bool,
) -> Result<(), Error> {
    if *seen {
        return Err(Error::at(
            file,
            attribute.span.start,
            "multiple macro transparency attributes",
        ));
    }
    *seen = true;
    let unit = package.unit(file);
    let mut node = owner;
    for _ in 0..=unit.configured.ast.nodes.len() {
        if let Kind::Attributed { value, .. } = unit.configured.ast.node(node).kind {
            node = value;
        } else {
            transparency(package, unit, NodeKey { file, node }, attribute, forest)?;
            return Ok(());
        }
    }
    Err(Error::at(
        file,
        attribute.span.start,
        "cyclic macro attribute owner",
    ))
}

pub(in crate::resolve) fn collect_transparency(
    package: &Package,
    unit: &Unit,
    key: NodeKey,
    attribute: Attribute,
    forest: &Forest,
    seen: &mut bool,
) -> Result<bool, Error> {
    if attribute.name(forest, unit.source.text()) != Some("rustc_macro_transparency") {
        return Ok(false);
    }
    if *seen {
        return Err(Error::at(
            key.file,
            attribute.span.start,
            "multiple macro transparency attributes",
        ));
    }
    *seen = true;
    transparency(package, unit, key, attribute, forest)?;
    Ok(true)
}
