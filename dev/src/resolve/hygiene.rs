//! Local binding histories preserve definition-site visibility across shadowing.
use super::{
    expansion,
    model::{DefKind, Namespace, NodeScopes, Resolution, ScopeId},
    package::ModuleId,
    unit::NodeKey,
};
use crate::support::hash::HashMap;
use crate::{
    expand::marks::{ExpansionId, ItemContext, LocalContext, Marks},
    support::names::NameId,
    syntax::ast::NodeId,
};
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct Ident(u64);
impl Ident {
    pub const fn new(name: NameId, context: LocalContext) -> Self {
        Self((context.key() as u64) << 32 | name.0 as u64)
    }
}
struct MacroSite {
    scope: Option<ScopeId>,
    visible: usize,
}
pub(crate) struct Query {
    pub scope: ScopeId,
    pub name: NameId,
    pub namespace: Namespace,
    pub context: LocalContext,
    pub item: ItemContext,
    /// Parameter ribs use the original item identity even after local rewinding.
    pub parameter: ItemContext,
    pub privacy: ModuleId,
    pub visible: usize,
}
/// Macro definition sites; the locals themselves live in the binding table.
pub(crate) struct Hygiene {
    sites: Vec<MacroSite>,
    definitions: HashMap<(ModuleId, NodeId), usize>,
    expansions: Vec<usize>,
}
impl Hygiene {
    /// Macro sites start at the scopes the expansion scan recorded for their
    /// definitions; the bodies walk later records what each site can see.
    pub fn new(
        definitions: &[expansion::Definition],
        invocations: &[expansion::Invocation],
        scopes: &NodeScopes,
    ) -> Self {
        Self {
            sites: definitions
                .iter()
                .map(|d| MacroSite {
                    scope: d
                        .node
                        .and_then(|node| scopes.get(&(d.module, node.node)).copied()),
                    visible: 0,
                })
                .collect(),
            definitions: definitions
                .iter()
                .enumerate()
                .filter_map(|(index, d)| d.node.map(|node| ((d.module, node.node), index)))
                .collect(),
            expansions: invocations.iter().map(|i| i.definition).collect(),
        }
    }
    pub fn snapshot(&mut self, module: ModuleId, node: NodeId, scope: ScopeId, visible: usize) {
        if let Some(index) = self.definitions.get(&(module, node)) {
            self.sites[*index].scope = Some(scope);
            self.sites[*index].visible = visible;
        }
    }
    pub const fn is_local(kind: DefKind) -> bool {
        matches!(kind, DefKind::Local | DefKind::Parameter | DefKind::Label)
    }
    fn site(&self, context: ExpansionId) -> Result<&MacroSite, &'static str> {
        let index = context
            .0
            .checked_sub(1)
            .ok_or("source context has no macro definition")?;
        let definition = *self
            .expansions
            .get(index as usize)
            .ok_or("invalid expansion context")?;
        self.sites
            .get(definition)
            .ok_or("invalid macro definition context")
    }
    fn rewind(
        &self,
        facts: &Resolution,
        marks: &Marks,
        query: &mut Query,
        parent: LocalContext,
        expansion: ExpansionId,
    ) -> Result<Option<ScopeId>, &'static str> {
        let site = self.site(expansion)?;
        let item = marks.local_item(parent)?;
        if item != query.item {
            let scope = facts.introductions.definition_scope(expansion)?;
            query.privacy = facts
                .scopes
                .get(scope.0 as usize)
                .ok_or("invalid macro definition privacy")?
                .module;
        }
        query.context = parent;
        query.item = item;
        query.visible = query.visible.min(site.visible);
        Ok(site.scope)
    }
    pub fn lookup(
        &self,
        facts: &Resolution,
        marks: &Marks,
        query: &mut Query,
    ) -> Result<Option<super::contextual_lookup::Selected>, &'static str> {
        let mut current = Some(query.scope);
        let mut barriers = 0;
        let mut label_barrier = false;
        let mut remaining = marks.context_count().saturating_mul(facts.scopes.len() + 1);
        while remaining > 0 {
            remaining -= 1;
            let Some(scope) = current else {
                let Some((parent, expansion)) = marks.local_parent(query.context)? else {
                    return Ok(None);
                };
                current = self.rewind(facts, marks, query, parent, expansion)?;
                continue;
            };
            let frame = facts
                .scopes
                .get(scope.0 as usize)
                .ok_or("invalid hygienic scope")?;
            // One probe answers both the local history and the ordinary binding.
            let entry = facts.bindings.probe(scope, query.name, query.namespace);
            if barriers == 0
                && !label_barrier
                && let Some(entry) = entry
                && let Some(binding) = facts.bindings.local(entry, query.context, query.visible)
            {
                return Ok(Some(super::contextual_lookup::Selected {
                    key: (scope, query.name, query.namespace, ItemContext::ROOT),
                    binding,
                    privacy: query.privacy,
                }));
            }
            if let Some((parent, expansion)) = marks.local_parent(query.context)? {
                let site = self.site(expansion)?;
                if site.scope == Some(scope) {
                    self.rewind(facts, marks, query, parent, expansion)?;
                    continue;
                }
            }
            let key = (scope, query.name, query.namespace, query.item);
            let binding = if query.item == ItemContext::ROOT {
                entry.and_then(|entry| facts.bindings.binding(entry))
            } else {
                facts.bindings.get_key(key)
            };
            if let Some(binding) = binding
                && binding.permits_lexical_lookup(
                    &facts.definitions,
                    frame,
                    barriers,
                    query.item == query.parameter,
                )
            {
                return Ok(Some(super::contextual_lookup::Selected {
                    key,
                    binding: *binding,
                    privacy: query.privacy,
                }));
            }
            if frame.parent.is_none()
                && let Some((_, expansion)) = marks.item_parent(query.item)?
                && facts
                    .introductions
                    .ancestor(expansion, facts.introductions.scope(scope))
            {
                return Ok(None);
            }
            barriers += u32::from(frame.barrier);
            label_barrier |= query.namespace == Namespace::Label && frame.label_barrier;
            current = frame.parent;
        }
        Err("hygienic lookup work budget exceeded")
    }
}

impl super::index::Resolver<'_> {
    /// Item identities follow the declared identifier, independently of the
    /// expansion that introduced the surrounding item or module. Implicit Self
    /// deliberately retains the pinned language implementation's root identity.
    pub(crate) fn binding_key(
        &self,
        definition: &super::model::Definition,
    ) -> Result<super::table::BindingKey, super::package::Error> {
        let context = if definition.kind == DefKind::SelfType {
            ItemContext::ROOT
        } else if let Some(node) = definition.node {
            self.package
                .item_context(node.file, name_offset(self.package, node))?
        } else {
            ItemContext::ROOT
        };
        Ok((
            definition.scope,
            definition.name,
            definition.namespace,
            context,
        ))
    }
}

pub(crate) fn name_offset(package: &super::package::Package, key: NodeKey) -> u32 {
    use crate::syntax::ast::Kind;
    let ast = &package.unit(key.file).configured.ast;
    let mut node = key.node;
    loop {
        let value = ast.node(node);
        match value.kind {
            Kind::Name(name, _)
            | Kind::Binding { name, .. }
            | Kind::Label { name, .. }
            | Kind::Receiver { name, .. } => {
                return name.start;
            }
            Kind::Item(index) => {
                let item = ast.items[index as usize];
                let name = match item.kind {
                    crate::syntax::item::ItemKind::ExternCrate { rename: Some(name) } => Some(name),
                    _ => item.name,
                };
                return name.map_or(value.span.start, |name| name.start);
            }
            Kind::Generic(index) => return ast.generics[index as usize].name.start,
            Kind::Variant(index) => return ast.variants[index as usize].name.start,
            Kind::FieldDefinition {
                name: Some(name), ..
            } => return name.start,
            Kind::Path { segments, .. } => {
                let Some(first) = ast.list(segments).first() else {
                    return value.span.start;
                };
                node = *first;
            }
            Kind::Applied { base, .. } => node = base,
            Kind::Attributed { value, .. } => node = value,
            _ => return value.span.start,
        }
    }
}

#[cfg(test)]
#[path = "../../tests/invariants/identifier_keys.rs"]
mod tests;
