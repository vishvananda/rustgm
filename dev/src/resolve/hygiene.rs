//! Local binding histories preserve definition-site visibility across shadowing.
use super::{
    expansion,
    model::{Binding, DefId, DefKind, Definition, Namespace, Origin, Resolution, ScopeId},
    package::ModuleId,
    unit::NodeKey,
};
use crate::{expand::stream::ContextId, support::names::NameId, syntax::ast::NodeId};
use std::collections::HashMap;
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct Ident(u64);
impl Ident {
    pub const fn new(name: NameId, context: ContextId) -> Self {
        Self((context.0 as u64) << 32 | name.0 as u64)
    }
}
#[derive(Clone, Copy)]
enum History {
    One(Binding),
    Many(usize),
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct Key {
    scope: ScopeId,
    name: NameId,
    namespace: Namespace,
    context: ContextId,
}
struct MacroSite {
    context: ContextId,
    scope: Option<ScopeId>,
    visible: usize,
}
pub(crate) struct Query {
    pub scope: ScopeId,
    pub name: NameId,
    pub namespace: Namespace,
    pub context: ContextId,
    pub visible: usize,
}
pub(crate) struct Hygiene {
    sites: Vec<MacroSite>,
    definitions: HashMap<(ModuleId, NodeId), usize>,
    expansions: Vec<usize>,
    locals: HashMap<Key, History>,
    shadowed: Vec<Vec<Binding>>,
}
impl Hygiene {
    pub fn new(
        definitions: &[expansion::Definition],
        invocations: &[expansion::Invocation],
    ) -> Self {
        Self {
            sites: definitions
                .iter()
                .map(|d| MacroSite {
                    context: d.context,
                    scope: None,
                    visible: 0,
                })
                .collect(),
            definitions: definitions
                .iter()
                .enumerate()
                .map(|(index, d)| ((d.module, d.node.node), index))
                .collect(),
            expansions: invocations.iter().map(|i| i.definition).collect(),
            locals: HashMap::new(),
            shadowed: Vec::new(),
        }
    }
    pub fn scope(&mut self, module: ModuleId, node: NodeId, scope: ScopeId) {
        if let Some(index) = self.definitions.get(&(module, node)) {
            self.sites[*index].scope = Some(scope);
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
    pub fn define(
        &mut self,
        definition: &Definition,
        target: DefId,
        context: ContextId,
        shadow: bool,
    ) -> Result<(), &'static str> {
        let key = Key {
            scope: definition.scope,
            name: definition.name,
            namespace: definition.namespace,
            context,
        };
        let binding = Binding {
            target,
            access: definition.access,
            origin: Origin::Direct,
            ambiguous: false,
        };
        if let Some(history) = self.locals.get_mut(&key) {
            if !shadow {
                return Err("duplicate hygienic local binding");
            }
            match *history {
                History::One(previous) => {
                    let index = self.shadowed.len();
                    self.shadowed.push(vec![previous, binding]);
                    *history = History::Many(index);
                }
                History::Many(index) => self.shadowed[index].push(binding),
            }
        } else {
            self.locals.insert(key, History::One(binding));
        }
        Ok(())
    }
    fn site(&self, context: ContextId) -> Result<&MacroSite, &'static str> {
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
    pub fn lookup(
        &self,
        facts: &Resolution,
        mut query: Query,
    ) -> Result<Option<(ScopeId, Binding)>, &'static str> {
        let mut current = Some(query.scope);
        let mut barriers = 0;
        let mut label_barrier = false;
        let mut remaining = (self.expansions.len() + 1).saturating_mul(facts.scopes.len() + 1);
        while remaining > 0 {
            remaining -= 1;
            let Some(scope) = current else {
                if query.context.0 == 0 {
                    return Ok(None);
                }
                let site = self.site(query.context)?;
                query.context = site.context;
                query.visible = query.visible.min(site.visible);
                current = site.scope;
                continue;
            };
            let frame = facts
                .scopes
                .get(scope.0 as usize)
                .ok_or("invalid hygienic scope")?;
            if barriers == 0 && !label_barrier {
                let key = Key {
                    scope,
                    name: query.name,
                    namespace: query.namespace,
                    context: query.context,
                };
                if let Some(history) = self.locals.get(&key) {
                    let found = match *history {
                        History::One(binding) => {
                            ((binding.target.0 as usize) < query.visible).then_some(binding)
                        }
                        History::Many(index) => {
                            let history = &self.shadowed[index];
                            let end = history.partition_point(|binding| {
                                (binding.target.0 as usize) < query.visible
                            });
                            end.checked_sub(1).map(|index| history[index])
                        }
                    };
                    if let Some(binding) = found {
                        return Ok(Some((scope, binding)));
                    }
                }
            }
            if query.context.0 != 0 {
                let site = self.site(query.context)?;
                if site.scope == Some(scope) {
                    query.context = site.context;
                    query.visible = query.visible.min(site.visible);
                    continue;
                }
            }
            if let Some(binding) = frame.bindings.get(&(query.name, query.namespace))
                && (barriers == 0
                    || !facts.definitions[binding.target.0 as usize].kind.is_local()
                    || (barriers == 1
                        && frame.associated
                        && matches!(
                            facts.definitions[binding.target.0 as usize].kind,
                            DefKind::Generic | DefKind::SelfType
                        )))
            {
                return Ok(Some((scope, *binding)));
            }

            barriers += u32::from(frame.barrier);
            label_barrier |= query.namespace == Namespace::Label && frame.label_barrier;
            current = frame.parent;
        }
        Err("hygienic lookup work budget exceeded")
    }
}

pub(crate) fn name_offset(package: &super::package::Package, key: NodeKey) -> u32 {
    use crate::syntax::ast::Kind;
    let ast = &package.unit(key.file).configured.ast;
    let mut node = key.node;
    loop {
        let value = ast.node(node);
        match value.kind {
            Kind::Name(name)
            | Kind::Binding { name, .. }
            | Kind::Label { name, .. }
            | Kind::Receiver { name, .. } => {
                return name.start;
            }
            Kind::Path { segments, .. } => {
                let Some(first) = ast.list(segments).first() else {
                    return value.span.start;
                };
                node = *first;
            }
            Kind::Applied { base, .. } => node = base,
            _ => return value.span.start,
        }
    }
}
