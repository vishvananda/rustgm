//! Namespace-directed lookup returns missing binding keys for import dependencies.
pub(crate) use super::table::BindingKey;
use super::{
    index::{Context, Resolver},
    model::{Access, Binding, DefId, DefKind, Namespace, Origin, ScopeId, Target},
    names::NameId,
    package::{Error, ModuleId},
};
use crate::{expand::marks::ItemContext, syntax::ast::NodeId};
#[derive(Clone, Copy)]
pub(crate) enum LookupMode {
    Use(Namespace),
    Import(Namespace),
}
pub(crate) struct Selection {
    pub key: BindingKey,
    pub binding: Binding,
    pub lexical: bool,
}
#[derive(Default)]
pub(crate) struct Observations {
    pub dependencies: Vec<BindingKey>,
    pub selected: Option<Vec<Selection>>,
    pub privacy: Option<ModuleId>,
}
pub(crate) enum Lookup {
    Found(Binding),
    /// A member resolved directly in an associated scope still has a typed prefix.
    Member(Binding, DefId, usize),
    Associated(DefId, usize),
    SelfConstructor(DefId, bool),
    Missing,
}
impl Resolver<'_> {
    /// Rust's value-form Self crosses item barriers only when its eventual type
    /// is closed. Keep that distinction for the typed constructor elaborator.
    pub fn self_constructor(&self, c: Context, name: NameId) -> Lookup {
        let mut current = Some(c.scope);
        let mut barriers = 0_u32;
        while let Some(scope) = current {
            let frame = &self.facts.scopes[scope.0 as usize];
            if frame.module != c.module {
                break;
            }
            if let Some(binding) = self.facts.bindings.get(scope, name, Namespace::Type)
                && self.facts.definitions[binding.target.0 as usize].kind == DefKind::SelfType
            {
                return Lookup::SelfConstructor(binding.target, barriers > 1);
            }
            barriers += u32::from(frame.barrier);
            current = frame.parent;
        }
        Lookup::Missing
    }
    pub fn textual_conflict(
        &self,
        scope: ScopeId,
        name: NameId,
        target: DefId,
        introduction: crate::expand::marks::ExpansionId,
        dependencies: &mut Vec<BindingKey>,
    ) -> bool {
        let definition_scope = self.facts.definitions[target.0 as usize].scope;
        let marks = &self.facts.introductions;
        let defined = marks
            .definitions
            .get(&target)
            .copied()
            .unwrap_or(crate::expand::marks::ExpansionId(0));
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
            dependencies.push((scope, name, Namespace::Macro, ItemContext::ROOT));
            if let Some(binding) = self.facts.bindings.get(scope, name, Namespace::Macro) {
                return (binding.ambiguous || binding.target != target)
                    && (inside || !marks.ancestor(defined, marks.binding(*binding)));
            }
            current = self.facts.scopes[scope.0 as usize].parent;
        }
        more_expanded
            && self
                .facts
                .macro_prelude
                .get(&name)
                .is_some_and(|builtin| *builtin != target)
    }
    pub fn hygienic_lexical(
        &self,
        c: Context,
        offset: u32,
        name: NameId,
        namespace: Namespace,
    ) -> Result<Option<(ScopeId, Binding)>, Error> {
        Ok(self
            .hygienic_selection(c, offset, name, namespace)?
            .map(|selected| (selected.key.0, selected.binding)))
    }
    fn hygienic_selection(
        &self,
        c: Context,
        offset: u32,
        name: NameId,
        namespace: Namespace,
    ) -> Result<Option<super::item_lookup::Selected>, Error> {
        let raw = self.package.syntax_context(c.file, offset)?;
        let item = self
            .package
            .marks
            .item(raw)
            .map_err(|e| Error::at(c.file, offset, e))?;
        if let Some(hygiene) = self.hygiene.as_ref()
            && !(namespace == Namespace::Type && self.package.names.get(name) == "Self")
        {
            let context = self
                .package
                .marks
                .local(raw)
                .map_err(|e| Error::at(c.file, offset, e))?;
            let mut query = super::hygiene::Query {
                scope: c.scope,
                name,
                namespace,
                context,
                item,
                privacy: c.module,
                visible: self.facts.definitions.len(),
            };
            let found = hygiene
                .lookup(self.facts, &self.package.marks, &mut query)
                .map_err(|e| Error::at(c.file, offset, e))?;
            return Ok(found.or_else(|| {
                self.lexical_prelude(query.privacy, name, namespace)
                    .map(|(scope, binding)| super::item_lookup::Selected {
                        key: (scope, name, namespace, ItemContext::ROOT),
                        binding,
                        privacy: query.privacy,
                    })
            }));
        }
        self.item_lexical(c, name, namespace, item, None)
            .map_err(|e| Error::at(c.file, offset, e))
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
            Access::ExternalPrivate => false,
            Access::Within(ancestor) => self.package.is_descendant(module, ancestor),
        }
    }
    pub(super) fn lexical_prelude(
        &self,
        module: ModuleId,
        name: NameId,
        namespace: Namespace,
    ) -> Option<(ScopeId, Binding)> {
        if namespace == Namespace::Macro
            && let Some(target) = self.facts.macro_prelude.get(&name)
        {
            return Some((
                ScopeId(0),
                Binding {
                    target: *target,
                    access: Access::Public,
                    origin: Origin::Direct,
                    ambiguous: false,
                },
            ));
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
        self.builtin_binding(module, name, namespace)
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
        path: crate::resolve::paths::Identified<'_>,
        mode: LookupMode,
        mut observed: Option<&mut Observations>,
    ) -> Result<Lookup, Error> {
        let (namespace, check_final_privacy) = match mode {
            LookupMode::Use(ns) => (ns, true),
            LookupMode::Import(ns) => (ns, false),
        };
        let Some(first) = path.first().copied() else {
            return Err(self.error(c, node, "empty path"));
        };
        let mut index = 0;
        let mut scope = if absolute {
            // A later explicit extern declaration can introduce this name.
            if let Some(observed) = observed.as_mut() {
                observed
                    .dependencies
                    .push((ScopeId(0), first, Namespace::Type, ItemContext::ROOT));
            }
            Some(self.external_root(c, node, first)?)
        } else {
            None
        };
        let mut associated = None;
        let mut module = c.module;
        match self.package.names.get(first) {
            "crate" => {
                scope = Some(self.facts.module_scopes[0]);
                index = 1;
                module = ModuleId(0);
            }
            "self" if path.len() > 1 => {
                let mut context = path.context(0).map_err(|e| self.error(c, node, e))?;
                module = self
                    .self_module(module, &mut context)
                    .map_err(|e| self.error(c, node, e))?;
                scope = Some(self.facts.module_scopes[module.0 as usize]);
                index = 1;
            }
            "super" => {
                while index < path.len() && self.package.names.get(path[index]) == "super" {
                    let mut context = path.context(index).map_err(|e| self.error(c, node, e))?;
                    if index == 0 {
                        module = self
                            .self_module(module, &mut context)
                            .map_err(|e| self.error(c, node, e))?;
                    }
                    module = self.package.modules[module.0 as usize]
                        .parent
                        .ok_or_else(|| self.error(c, node, "super escapes crate root"))?;
                    module = self
                        .self_module(module, &mut context)
                        .map_err(|e| self.error(c, node, e))?;
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
            let context = path.context(index).map_err(|e| self.error(c, node, e))?;
            let found = if let Some(scope) = scope {
                self.item_in_scope(c, (scope, name, ns, context), observed.as_deref_mut())
                    .map_err(|e| self.error(c, node, e))?
            } else if self.hygiene.is_some() && !matches!(mode, LookupMode::Import(_)) {
                let offset = super::hygiene::name_offset(
                    self.package,
                    super::unit::NodeKey { file: c.file, node },
                );
                self.hygienic_selection(c, offset, name, ns)?
            } else {
                self.item_lexical(c, name, ns, context, observed.as_deref_mut())
                    .map_err(|e| self.error(c, node, e))?
            };
            let selected = match found {
                Some(found) => found,
                None if associated.is_some() => {
                    if let Some(target) = associated {
                        return Ok(Lookup::Associated(target, index));
                    }
                    return Ok(Lookup::Missing);
                }
                None => return Ok(Lookup::Missing),
            };
            let found_scope = selected.key.0;
            let binding = selected.binding;
            if let Some(trace) = observed.as_mut().and_then(|o| o.selected.as_mut()) {
                trace.push(Selection {
                    key: selected.key,
                    binding,
                    lexical: scope.is_none(),
                });
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
            if !self.visible(binding.access, selected.privacy)
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
                if let Some(observed) = observed.as_deref_mut() {
                    observed.privacy = Some(selected.privacy);
                }
                if check_final_privacy
                    && self.facts.scopes[found_scope.0 as usize].associated
                    && let Some(owner) = associated
                {
                    return Ok(Lookup::Member(binding, owner, index - 1));
                }
                return Ok(Lookup::Found(binding));
            }
            let definition = &self.facts.definitions[binding.target.0 as usize];
            if let Some(members) = definition.members {
                scope = Some(members);
                // Enum variants occupy ordinary import namespaces. A record
                // variant intentionally has no value constructor; that absence
                // cannot turn a valid type-only import into an associated import.
                associated = (namespace != Namespace::Macro
                    && !matches!(definition.kind, DefKind::Module | DefKind::ExternCrate)
                    && !(matches!(mode, LookupMode::Import(_))
                        && definition.kind == DefKind::Enum))
                    .then_some(binding.target);
            } else if matches!(definition.kind, DefKind::Module | DefKind::ExternCrate) {
                return Err(self.error(c, node, "module metadata unavailable"));
            } else {
                return Ok(if namespace == Namespace::Macro {
                    Lookup::Missing
                } else {
                    Lookup::Associated(binding.target, index)
                });
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
            Lookup::Found(binding) | Lookup::Member(binding, ..) => {
                Ok(Target::Definition(binding.target))
            }
            Lookup::Associated(def, _) => Ok(Target::Associated(*def)),
            Lookup::SelfConstructor(def, false) => Ok(Target::SelfConstructor(*def)),
            Lookup::SelfConstructor(def, true) => Ok(Target::InheritedSelfConstructor(*def)),
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
