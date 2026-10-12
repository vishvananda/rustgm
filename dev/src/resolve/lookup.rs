//! Namespace-directed lookup returns missing binding keys for import dependencies.
use super::{
    index::{Context, Resolver},
    model::{Access, Binding, DefId, DefKind, Namespace, Origin, ScopeId, Target},
    names::NameId,
    package::{Error, ModuleId},
};
use crate::syntax::ast::NodeId;
#[derive(Clone, Copy)]
pub(crate) enum LookupMode {
    Use(Namespace),
    Import(Namespace),
}
pub(crate) type BindingKey = (ScopeId, NameId, Namespace);
#[derive(Default)]
pub(crate) struct Observations {
    pub dependencies: Vec<BindingKey>,
    pub selected: Option<Vec<(BindingKey, Binding)>>,
}
pub(crate) enum Lookup {
    Found(Binding),
    Associated(DefId),
    Missing,
}
impl Resolver<'_> {
    pub fn textual_conflict(
        &self,
        scope: ScopeId,
        name: NameId,
        target: DefId,
        introduction: crate::expand::stream::ContextId,
        dependencies: &mut Vec<BindingKey>,
    ) -> bool {
        let definition_scope = self.facts.definitions[target.0 as usize].scope;
        let marks = &self.facts.introductions;
        let defined = marks
            .definitions
            .get(&target)
            .copied()
            .unwrap_or(crate::expand::stream::ContextId(0));
        let more_expanded = !marks.ancestor(defined, introduction);
        let mut inside = true;
        let mut current = Some(scope);
        while let Some(scope) = current {
            if scope == definition_scope {
                inside = false;
            }
            if !inside && !more_expanded {
                break;
            }
            dependencies.push((scope, name, Namespace::Macro));
            let frame = &self.facts.scopes[scope.0 as usize];
            if let Some(binding) = frame.bindings.get(&(name, Namespace::Macro)) {
                return (binding.ambiguous || binding.target != target)
                    && (inside || !marks.ancestor(defined, marks.binding(*binding)));
            }
            current = frame.parent;
        }
        false
    }
    pub fn hygienic_lexical(
        &self,
        c: Context,
        offset: u32,
        name: NameId,
        namespace: Namespace,
    ) -> Result<Option<(ScopeId, Binding)>, Error> {
        if matches!(namespace, Namespace::Value | Namespace::Label)
            && let Some(hygiene) = self.hygiene.as_ref()
        {
            let context = self.package.expansion_context(c.file, offset)?;
            return hygiene
                .lookup(
                    self.facts,
                    super::hygiene::Query {
                        scope: c.scope,
                        name,
                        namespace,
                        context,
                        visible: self.facts.definitions.len(),
                    },
                )
                .map_err(|e| Error::at(c.file, offset, e));
        }
        Ok(self.lexical_binding(c.scope, name, namespace))
    }
    pub fn hygienic_name(
        &self,
        c: Context,
        node: NodeId,
        offset: u32,
        name: NameId,
        namespace: Namespace,
    ) -> Result<Target, Error> {
        let found = self
            .hygienic_lexical(c, offset, name, namespace)?
            .map_or(Lookup::Missing, |(_, binding)| Lookup::Found(binding));
        self.target(c, node, &found, namespace)
    }
    pub fn visible(&self, access: Access, module: ModuleId) -> bool {
        match access {
            Access::Public => true,
            Access::Within(ancestor) => self.package.is_descendant(module, ancestor),
        }
    }
    pub fn lexical(&self, scope: ScopeId, name: NameId, namespace: Namespace) -> Lookup {
        self.lexical_binding(scope, name, namespace)
            .map_or(Lookup::Missing, |(_, binding)| Lookup::Found(binding))
    }
    fn lexical_binding(
        &self,
        scope: ScopeId,
        name: NameId,
        namespace: Namespace,
    ) -> Option<(ScopeId, Binding)> {
        let mut current = Some(scope);
        let mut barriers = 0_u32;
        while let Some(scope) = current {
            let frame = &self.facts.scopes[scope.0 as usize];
            if let Some(binding) = frame.bindings.get(&(name, namespace))
                && (barriers == 0
                    || !self.facts.definitions[binding.target.0 as usize]
                        .kind
                        .is_local()
                    || (barriers == 1
                        && frame.associated
                        && matches!(
                            self.facts.definitions[binding.target.0 as usize].kind,
                            DefKind::Generic | DefKind::SelfType
                        )))
            {
                return Some((scope, *binding));
            }
            if namespace == Namespace::Label && frame.label_barrier {
                break;
            }
            barriers += u32::from(frame.barrier);
            current = frame.parent;
        }
        if namespace == Namespace::Type
            && let Some(def) = self.facts.primitives.get(&name)
        {
            return Some((
                ScopeId(0),
                Binding {
                    target: *def,
                    access: Access::Public,
                    origin: Origin::Direct,
                    ambiguous: false,
                },
            ));
        }
        None
    }
    #[expect(
        clippy::too_many_lines,
        reason = "bounded path-prefix and namespace traversal share privacy checks"
    )]
    pub fn lookup_path(
        &self,
        c: Context,
        node: NodeId,
        absolute: bool,
        path: &[NameId],
        mode: LookupMode,
        mut observed: Option<&mut Observations>,
    ) -> Result<Lookup, Error> {
        let (namespace, check_final_privacy) = match mode {
            LookupMode::Use(ns) => (ns, true),
            LookupMode::Import(ns) => (ns, false),
        };
        if absolute {
            return Err(self.error(c, node, "external-crate paths require external metadata"));
        }
        let Some(first) = path.first().copied() else {
            return Err(self.error(c, node, "empty path"));
        };
        let mut index = 0;
        let mut scope = None;
        let mut associated = None;
        let mut module = c.module;
        match self.package.names.get(first) {
            "crate" => {
                scope = Some(self.facts.module_scopes[0]);
                index = 1;
                module = ModuleId(0);
            }
            "self" if path.len() > 1 => {
                scope = Some(self.facts.module_scopes[c.module.0 as usize]);
                index = 1;
            }
            "super" => {
                while index < path.len() && self.package.names.get(path[index]) == "super" {
                    module = self.package.modules[module.0 as usize]
                        .parent
                        .ok_or_else(|| self.error(c, node, "super escapes crate root"))?;
                    index += 1;
                }
                scope = Some(self.facts.module_scopes[module.0 as usize]);
            }
            _ => {}
        }
        if index == path.len() {
            if namespace != Namespace::Type {
                return Ok(Lookup::Missing);
            }
            if module == ModuleId(0)
                && let Some(target) = self.facts.crate_root
            {
                return Ok(Lookup::Found(Binding {
                    target,
                    access: Access::Public,
                    origin: Origin::Direct,
                    ambiguous: false,
                }));
            }
            // Importing self/crate names the module itself, whose declaration belongs to its parent.
            if let Some(declaration) = self.package.modules[module.0 as usize].declaration
                && let Some(parent) = self.package.modules[module.0 as usize].parent
                && let Some(def) =
                    self.facts
                        .item_definitions
                        .get(&(parent, declaration.node, Namespace::Type))
            {
                let definition = &self.facts.definitions[def.0 as usize];
                return Ok(Lookup::Found(Binding {
                    target: *def,
                    access: definition.access,
                    origin: Origin::Direct,
                    ambiguous: false,
                }));
            }
            return Err(self.error(c, node, "crate-root import requires an item path"));
        }
        loop {
            let name = path[index];
            let ns = if index + 1 == path.len() {
                namespace
            } else {
                Namespace::Type
            };
            if let Some(observed) = observed.as_mut() {
                let keys = &mut observed.dependencies;
                if let Some(scope) = scope {
                    keys.push((scope, name, ns));
                } else {
                    let mut parent = Some(c.scope);
                    while let Some(scope) = parent {
                        keys.push((scope, name, ns));
                        parent = self.facts.scopes[scope.0 as usize].parent;
                    }
                }
            }
            let found = if let Some(scope) = scope {
                self.facts.scopes[scope.0 as usize]
                    .bindings
                    .get(&(name, ns))
                    .copied()
                    .map(|binding| (scope, binding))
            } else if self.hygiene.is_some() {
                let offset = super::hygiene::name_offset(
                    self.package,
                    super::unit::NodeKey { file: c.file, node },
                );
                self.hygienic_lexical(c, offset, name, ns)?
            } else {
                self.lexical_binding(c.scope, name, ns)
            };
            let (found_scope, binding) = match found {
                Some(found) => found,
                None if associated.is_some() => {
                    if let Some(target) = associated {
                        return Ok(Lookup::Associated(target));
                    }
                    return Ok(Lookup::Missing);
                }
                None => return Ok(Lookup::Missing),
            };
            if let Some(trace) = observed.as_mut().and_then(|o| o.selected.as_mut()) {
                trace.push(((found_scope, name, ns), binding));
            }
            if binding.ambiguous {
                return Err(self.error(
                    c,
                    node,
                    format!(
                        "ambiguous {} name `{}`",
                        ns.name(),
                        self.package.names.get(name)
                    ),
                ));
            }
            if !self.visible(binding.access, c.module)
                && (check_final_privacy || index + 1 != path.len())
            {
                return Err(self.error(
                    c,
                    node,
                    format!(
                        "private {} name `{}`",
                        ns.name(),
                        self.package.names.get(name)
                    ),
                ));
            }
            index += 1;
            if index == path.len() {
                return Ok(Lookup::Found(binding));
            }
            let definition = &self.facts.definitions[binding.target.0 as usize];
            if let Some(members) = definition.members {
                scope = Some(members);
                associated = (definition.kind != DefKind::Module).then_some(binding.target);
            } else if matches!(definition.kind, DefKind::Module | DefKind::ExternCrate) {
                return Err(self.error(c, node, "module metadata unavailable"));
            } else {
                return Ok(Lookup::Associated(binding.target));
            }
        }
    }
    pub fn target(
        &self,
        c: Context,
        node: NodeId,
        found: &Lookup,
        namespace: Namespace,
    ) -> Result<Target, Error> {
        match found {
            Lookup::Found(binding) => Ok(Target::Definition(binding.target)),
            Lookup::Associated(def) => Ok(Target::Associated(*def)),
            Lookup::Missing => {
                let mut current = Some(c.scope);
                while let Some(scope) = current {
                    if self
                        .facts
                        .expansions
                        .iter()
                        .any(|(owner, _)| *owner == scope)
                    {
                        return Ok(Target::Expansion(scope));
                    }
                    current = self.facts.scopes[scope.0 as usize].parent;
                }
                Err(self.error(c, node, format!("unresolved {} name", namespace.name())))
            }
        }
    }
}
