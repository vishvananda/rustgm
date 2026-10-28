//! A source-owned prelude selects a real module without installing a lexical glob.
use super::{
    contextual_lookup::Selected,
    index::{Context, Resolver},
    lookup::{Lookup, LookupMode, Observations},
    model::{DefId, DefKind, Introductions, Namespace, Resolution, ScopeId},
    package::{Error, Package},
    unit::NodeKey,
};
use crate::{
    expand::{configured::Effect, marks::ItemContext},
    support::{names::NameId, source::OriginalImage},
    syntax::{
        ast::{Kind, Node, NodeId},
        attribute::{Attribute, Input, Style},
        item::{Item, ItemKind, UseTree},
    },
};
use std::sync::Arc;
#[cfg(test)]
#[path = "../../tests/invariants/prelude_protocol.rs"]
mod tests;

pub(super) struct Prelude {
    owner: NodeKey,
    item_key: NodeKey,
    owner_node: Node,
    attributes: (usize, usize),
    capture: u32,
    node: Node,
    item: Item,
    selector: UseTree,
    effect: usize,
    attribute: Attribute,
    image: Arc<OriginalImage>,
    context: Context,
    selected: Option<(DefId, ScopeId)>,
}
fn pay(package: &Package, steps: usize, bytes: usize) -> Result<(), &'static str> {
    let context = package
        .macro_context()?
        .ok_or("prelude_import requires its supported local/source request owner")?;
    context.contracts().charge(steps, bytes)?;
    if !context.requested(super::bindings::policy::Feature::PreludeImport) {
        return Err("source prelude requires prelude_import");
    }
    Ok(())
}
pub(super) fn claim_plain(
    package: &Package,
    facts: &mut Resolution,
    context: Context,
    owner: NodeId,
) -> Result<(), Error> {
    let unit = package.unit(context.file);
    let Some(index) = unit.owner_indices[owner.get() as usize] else {
        return Ok(());
    };
    let configured = &unit.configured.owners[index.get() as usize - 1];
    for effect in configured.attributes.clone() {
        let row = &unit.configured.effects[effect];
        let forest = &unit.configured.ast.attributes[row.capture as usize].forest;
        if row.attribute.effect == Effect::Keep
            && row.attribute.attribute.name(forest, unit.source.text()) == Some("prelude_import")
        {
            claim(
                package,
                facts,
                NodeKey {
                    file: context.file,
                    node: owner,
                },
                effect,
                context,
            )?;
        }
    }
    Ok(())
}
pub(super) fn claim(
    package: &Package,
    facts: &mut Resolution,
    owner: NodeKey,
    effect: usize,
    context: Context,
) -> Result<(), Error> {
    let error = |message| Error::at(owner.file, 0, message);
    pay(package, 8, 0).map_err(error)?;
    let unit = package
        .units
        .get(owner.file.0 as usize)
        .ok_or_else(|| error("source prelude owner unit disappeared"))?;
    let configured = unit
        .owner_indices
        .get(owner.node.get() as usize)
        .copied()
        .flatten()
        .and_then(|index| unit.configured.owners.get(index.get() as usize - 1))
        .filter(|row| row.enabled && row.node == owner.node && row.attributes.contains(&effect))
        .ok_or_else(|| error("source prelude lost its actual attribute owner"))?;
    let retained = &unit.configured.effects[effect];
    let attribute = retained.attribute.attribute;
    let forest = &unit.configured.ast.attributes[retained.capture as usize].forest;
    if attribute.name(forest, unit.source.text()) != Some("prelude_import") {
        return Err(error(
            "source prelude effect belongs to a different attribute",
        ));
    }
    if retained.attribute.effect != Effect::Keep
        || attribute.input != Input::Empty
        || attribute.unsafe_wrapper
        || attribute.style != Style::Outer
    {
        return Err(error(
            "prelude_import requires an ordinary outer word attribute",
        ));
    }
    let mut item_key = owner;
    loop {
        pay(package, 1, 0).map_err(error)?;
        let Kind::Attributed { value, .. } = unit.configured.ast.node(item_key.node).kind else {
            break;
        };
        item_key.node = value;
    }
    let node = unit.configured.ast.node(item_key.node);
    let Kind::Item(index) = node.kind else {
        return Err(error("prelude_import requires its actual use item"));
    };
    let item = unit.configured.ast.items[index as usize];
    if !matches!(item.kind, ItemKind::Use(_)) || context.file != owner.file {
        return Err(error("prelude_import requires its actual use item"));
    }
    if configured.attributes.is_empty() || facts.introductions.source_prelude().is_some() {
        return Err(error(
            "multiple source prelude selectors are not implemented",
        ));
    }
    let bytes = size_of::<Prelude>()
        + if facts.introductions.item_scopes.is_none() {
            size_of::<super::container_scopes::ItemScopes>()
        } else {
            0
        };
    pay(package, 1, bytes).map_err(error)?;
    facts
        .introductions
        .item_scopes
        .get_or_insert_with(Box::default)
        .prelude = Some(Box::new(Prelude {
        owner,
        item_key,
        owner_node: unit.configured.ast.node(owner.node),
        attributes: (configured.attributes.start, configured.attributes.end),
        capture: retained.capture,
        node,
        item,
        selector: selector(unit, item).map_err(error)?,
        effect,
        attribute,
        image: Arc::clone(unit.source.original_image()),
        context,
        selected: None,
    }));
    Ok(())
}
fn selector(unit: &super::unit::Unit, item: Item) -> Result<UseTree, &'static str> {
    let ItemKind::Use(tree) = item.kind else {
        return Err("source prelude item is not a use");
    };
    let Some(node) = unit.configured.ast.nodes.get(tree.get() as usize) else {
        return Err("source prelude use tree disappeared");
    };
    let Kind::UseTree(index) = node.kind else {
        return Err("source prelude tree kind changed");
    };
    unit.configured
        .ast
        .use_trees
        .get(index as usize)
        .copied()
        .ok_or("source prelude tree disappeared")
}
impl Prelude {
    fn validate(&self, package: &Package, facts: &Resolution) -> Result<(), &'static str> {
        pay(package, 12, 0)?;
        let unit = package
            .units
            .get(self.owner.file.0 as usize)
            .ok_or("source prelude unit disappeared")?;
        let owner = unit
            .owner_indices
            .get(self.owner.node.get() as usize)
            .copied()
            .flatten()
            .and_then(|index| unit.configured.owners.get(index.get() as usize - 1));
        if selector(unit, self.item)? != self.selector
            || !Arc::ptr_eq(&self.image, unit.source.original_image())
            || owner.is_none_or(|row| {
                !row.enabled
                    || row.node != self.owner.node
                    || !row.attributes.contains(&self.effect)
                    || (row.attributes.start, row.attributes.end) != self.attributes
            })
            || unit
                .configured
                .effects
                .get(self.effect)
                .is_none_or(|effect| {
                    effect.attribute.effect != Effect::Keep
                        || effect.attribute.attribute != self.attribute
                        || effect.capture != self.capture
                })
            || unit
                .configured
                .ast
                .nodes
                .get(self.owner.node.get() as usize)
                != Some(&self.owner_node)
            || unit
                .configured
                .ast
                .nodes
                .get(self.item_key.node.get() as usize)
                != Some(&self.node)
            || !matches!(self.node.kind,Kind::Item(index) if unit.configured.ast.items.get(index as usize)==Some(&self.item))
            || facts
                .scopes
                .get(self.context.scope.0 as usize)
                .is_none_or(|scope| scope.module != self.context.module)
        {
            return Err("source prelude actual owner/effect changed");
        }
        Ok(())
    }
}
impl Introductions {
    pub(super) fn has_source_prelude(&self) -> bool {
        self.source_prelude().is_some()
    }
    fn source_prelude(&self) -> Option<&Prelude> {
        self.item_scopes.as_deref()?.prelude.as_deref()
    }
}
impl Resolution {
    pub(crate) fn source_prelude_scope(
        &self,
        package: &Package,
    ) -> Result<Option<ScopeId>, &'static str> {
        let Some(prelude) = self.introductions.source_prelude() else {
            return Ok(None);
        };
        prelude.validate(package, self)?;
        let Some((definition, scope)) = prelude.selected else {
            return Ok(None);
        };
        if self
            .definitions
            .get(definition.0 as usize)
            .is_none_or(|row| row.kind != DefKind::Module || row.members != Some(scope))
        {
            return Err("source prelude selected module changed");
        }
        Ok(Some(scope))
    }
}
impl Resolver<'_> {
    pub(super) fn select_source_prelude(
        &mut self,
        c: Context,
        tree: NodeId,
    ) -> Result<bool, Error> {
        let Some(prelude) = self.facts.introductions.source_prelude() else {
            return Ok(false);
        };
        if prelude.item.kind != ItemKind::Use(tree) || prelude.item_key.file != c.file {
            return Ok(false);
        }
        let error = |message| Error::at(c.file, 0, message);
        prelude.validate(self.package, self.facts).map_err(error)?;
        if c.scope != prelude.context.scope || c.module != prelude.context.module {
            return Err(error("source prelude actual use scope changed"));
        }
        if prelude.selected.is_some() {
            return Err(error("source prelude selector executed twice"));
        }
        let Kind::UseTree(index) = self.package.unit(c.file).configured.ast.node(tree).kind else {
            return Err(error("source prelude lost its actual use tree"));
        };
        let syntax = self.package.unit(c.file).configured.ast.use_trees[index as usize];
        if syntax.glob.is_none()
            || syntax.rename.is_some()
            || syntax.children.start != syntax.children.end
        {
            return Err(error(
                "source prelude requires the supported single glob selector",
            ));
        }
        let prefix = syntax
            .prefix
            .ok_or_else(|| error("source prelude requires a module path"))?;
        let count = match self.package.unit(c.file).configured.ast.node(prefix).kind {
            Kind::Path { segments, .. } => (segments.end - segments.start) as usize,
            _ => 1,
        };
        let mut growth = crate::support::retained::Growth::vector::<NameId>(0, 0, count);
        growth.include(crate::support::retained::Growth::vector::<ItemContext>(
            0, 0, count,
        ));
        pay(self.package, count.saturating_add(1), growth.bytes).map_err(error)?;
        let mut path = super::paths::Buffer::default();
        let absolute = self.package.append_path(c.file, prefix, &mut path)?;
        let found = self.lookup_path(
            c,
            tree,
            absolute,
            path.view(),
            LookupMode::Use(Namespace::Type),
            None,
        )?;
        let Lookup::Found(binding) = found else {
            return Err(error("cannot resolve an early source prelude module"));
        };
        let definition = self
            .facts
            .definitions
            .get(binding.target.0 as usize)
            .filter(|row| row.kind == DefKind::Module)
            .ok_or_else(|| error("source prelude target is not a module"))?;
        let scope = definition
            .members
            .ok_or_else(|| error("source prelude target has no member scope"))?;
        pay(self.package, 1, 0).map_err(error)?;
        let prelude = self
            .facts
            .introductions
            .item_scopes
            .as_deref_mut()
            .and_then(|sites| sites.prelude.as_deref_mut())
            .ok_or_else(|| error("source prelude owner disappeared"))?;
        prelude.selected = Some((binding.target, scope));
        Ok(true)
    }
    pub(super) fn terminal_prelude(
        &self,
        c: Context,
        requested: (NameId, Namespace, ItemContext),
        observed: Option<&mut Observations>,
        filter: Option<super::lookup::MacroFilter<'_>>,
    ) -> Result<Option<Selected>, &'static str> {
        let (name, namespace, context) = requested;
        if let Some(selected) = self
            .source_prelude_selection(c, name, namespace, context, observed)?
            .filter(|selected| filter.is_none_or(|filter| filter.permits(selected.binding.target)))
        {
            return Ok(Some(selected));
        }
        Ok(self
            .lexical_prelude(c.module, name, namespace)
            .filter(|(_, binding)| filter.is_none_or(|filter| filter.permits(binding.target)))
            .map(|(scope, binding)| Selected {
                key: (scope, name, namespace, ItemContext::ROOT),
                binding,
                privacy: c.module,
            }))
    }
    pub(super) fn source_prelude_selection(
        &self,
        c: Context,
        name: NameId,
        namespace: Namespace,
        context: ItemContext,
        observed: Option<&mut Observations>,
    ) -> Result<Option<Selected>, &'static str> {
        if namespace == Namespace::Macro && self.facts.macro_prelude.contains_key(&name)
            || namespace == Namespace::Type
                && self
                    .facts
                    .extern_scope
                    .and_then(|scope| self.facts.bindings.get(scope, name, namespace))
                    .is_some_and(|binding| {
                        self.facts.definitions[binding.target.0 as usize].kind == DefKind::Module
                    })
        {
            return Ok(None);
        }
        let Some(scope) = self.facts.source_prelude_scope(self.package)? else {
            return Ok(None);
        };
        let selected = self
            .item_in_scope(c, (scope, name, namespace, context), observed)?
            .filter(|selected| self.visible(selected.binding.access, selected.privacy));
        if self.package.modules[c.module.0 as usize].implicit_prelude {
            return Ok(selected);
        }
        if namespace == Namespace::Macro
            && let Some(selected) = selected
        {
            let definition = &self.facts.definitions[selected.binding.target.0 as usize];
            if definition.kind == DefKind::BuiltinMacro
                && self.facts.macro_prelude.get(&definition.name) == Some(&selected.binding.target)
            {
                return Ok(Some(selected));
            }
            if let Some(key) = definition.node
                && super::bindings::panic_contracts::checked_definition(self.package, key)
                    .map_err(|_| "source prelude builtin header authentication failed")?
                    .is_some_and(|header| header.summary().builtin.is_some())
            {
                return Ok(Some(selected));
            }
        }
        Ok(None)
    }
}
