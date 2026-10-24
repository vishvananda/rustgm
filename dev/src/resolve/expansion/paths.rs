//! Early macro paths reuse the ordinary import dependency graph and glob journals.
use super::{
    Builtin, Call, Error, MacroTarget, ModuleId, NodeId, NodeKey, Package, bindings::Bindings,
};
use crate::expand::marks::ItemContext;
use crate::support::hash::{HashMap, HashSet};
use crate::{
    expand::{
        marks::ExpansionId,
        textual::{self, Observer, Scope as TextScope, Scopes},
    },
    resolve::{
        imports::{TextualSource, Work},
        index::{Context, Pending, State},
        lookup::{BindingKey, Lookup, LookupMode, Observations},
        model::{Access, Binding, DefId, DefKind, Definition, Namespace, Origin, ScopeId},
    },
    support::names::NameId,
    syntax::item::Visibility,
};
use std::collections::VecDeque;

pub(super) struct Paths {
    pub state: State,
    work: Work,
    macros: Vec<Option<DefId>>,
    macro_targets: HashMap<DefId, MacroTarget>,
    textual: Vec<Option<(TextScope, ExpansionId)>>,
    text_ready: VecDeque<usize>,
    text_queued: Vec<bool>,
    waiting: HashMap<BindingKey, HashSet<usize>>,
    pending: Vec<Pending>,
}
impl Paths {
    pub fn new(
        package: &mut Package,
        dependencies: Option<std::sync::Arc<crate::support::crates::Dependencies>>,
    ) -> Result<Self, Error> {
        let mut state = State::default();
        let scope = state.bind(package).new_scope(None, ModuleId(0), true)?;
        state.facts.module_scopes.push(scope);
        state.facts.crate_root = Some(DefId(0));
        state.facts.definitions.push(Definition {
            name: package.modules[0].name,
            namespace: Namespace::Type,
            kind: DefKind::Module,
            scope,
            node: None,
            access: Access::Public,
            members: Some(scope),
        });
        let mut macro_targets = HashMap::default();
        let bootstrap = package.source_core.is_none();
        for builtin in Builtin::ALL
            .into_iter()
            .filter(|builtin| bootstrap && *builtin != Builtin::OptionEnv)
        {
            let name = package
                .names
                .intern(builtin.name())
                .map_err(|e| Error::at(super::FileId(0), 0, e))?;
            let target = DefId(
                u32::try_from(state.facts.definitions.len())
                    .map_err(|_| Error::at(super::FileId(0), 0, "too many builtin definitions"))?,
            );
            state.facts.definitions.push(Definition {
                name,
                namespace: Namespace::Macro,
                kind: DefKind::BuiltinMacro,
                scope,
                node: None,
                access: Access::Public,
                members: None,
            });
            state.facts.macro_prelude.insert(name, target);
            macro_targets.insert(target, MacroTarget::Builtin(builtin));
        }
        if let Some(dependencies) = dependencies {
            state.bind(package).import_crates(dependencies)?;
        }
        let mut work = Work::new(&state.facts);
        work.observe();
        Ok(Self {
            state,
            work,
            macros: Vec::new(),
            macro_targets,
            textual: Vec::new(),
            text_ready: VecDeque::new(),
            text_queued: Vec::new(),
            waiting: HashMap::default(),
            pending: Vec::new(),
        })
    }
    pub fn item(
        &mut self,
        package: &mut Package,
        context: Context,
        node: NodeId,
        item: crate::syntax::item::Item,
        introduction: ExpansionId,
    ) -> Result<ScopeId, Error> {
        let mut resolver = self.state.bind(package);
        let first = resolver.facts.definitions.len();
        let access = resolver.visibility(context, item.visibility)?;
        let body = resolver.declare_item(context, node, item, access)?;
        self.work.extend(resolver.facts);
        for index in first..resolver.facts.definitions.len() {
            let definition = &resolver.facts.definitions[index];
            if introduction.0 != 0 {
                let id = DefId(u32::try_from(index).map_err(|_| {
                    resolver.error(context, node, "too many early item definitions")
                })?);
                resolver
                    .facts
                    .introductions
                    .definitions
                    .insert(id, introduction);
            }
            self.work.declared(
                &resolver.facts.bindings,
                (
                    definition.scope,
                    definition.name,
                    definition.namespace,
                    ItemContext::ROOT,
                ),
            );
        }
        // Unnameable `impl Trait` parameters have no binding to publish.
        if matches!(item.kind, crate::syntax::item::ItemKind::Function { .. }) {
            self.pending.push(Pending::Anonymous(body, node, item));
        }
        Ok(body.scope)
    }
    /// A block's scope, which may declare items and textual macros.
    pub fn block(
        &mut self,
        package: &mut Package,
        parent: ScopeId,
        module: ModuleId,
        node: NodeId,
    ) -> Result<ScopeId, Error> {
        let scope = self
            .state
            .bind(package)
            .new_scope(Some(parent), module, false)?;
        let file = package.modules[module.0 as usize].file;
        self.state
            .facts
            .node_scopes
            .insert(module, file, node, scope);
        self.work.extend(&self.state.facts);
        Ok(scope)
    }
    /// Any other lexical scope (closure, arm, loop, label, binder or `if`),
    /// created on first visit so later phases see each scope's real parent.
    pub fn lexical(
        &mut self,
        package: &mut Package,
        context: Context,
        node: NodeId,
    ) -> Result<ScopeId, Error> {
        let scope = self.state.bind(package).lexical_scope(context, node)?;
        self.work.extend(&self.state.facts);
        Ok(scope)
    }
    /// Generic parameters are declared by resolution, after expansion.
    pub fn generic(&mut self, context: Context, node: NodeId, index: u32) {
        self.pending.push(Pending::Generic(context, node, index));
    }
    /// Resolution validates a textual macro's restricted visibility.
    pub fn macro_visibility(&mut self, context: Context, node: NodeId, visibility: Visibility) {
        if matches!(visibility, Visibility::Restricted(_)) {
            self.pending
                .push(Pending::Visibility(context, node, visibility));
        }
    }
    pub fn module(
        &mut self,
        package: &mut Package,
        context: Context,
        child: ModuleId,
        node: NodeId,
        visibility: Visibility,
        expansion: ExpansionId,
    ) -> Result<ScopeId, Error> {
        let mut resolver = self.state.bind(package);
        let access = resolver.visibility(context, visibility)?;
        let scope = resolver.new_scope(None, child, true)?;
        resolver
            .facts
            .module_scopes
            .resize(resolver.package.modules.len(), ScopeId(0));
        resolver.facts.module_scopes[child.0 as usize] = scope;
        let name = resolver.package.modules[child.0 as usize].name;
        let id = DefId(
            u32::try_from(resolver.facts.definitions.len())
                .map_err(|_| resolver.error(context, node, "too many macro-path definitions"))?,
        );
        resolver.facts.definitions.push(Definition {
            name,
            namespace: Namespace::Type,
            kind: DefKind::Module,
            scope: context.scope,
            node: Some(NodeKey {
                file: context.file,
                node,
            }),
            access,
            members: Some(scope),
        });
        if expansion.0 != 0 {
            resolver
                .facts
                .introductions
                .definitions
                .insert(id, expansion);
        }
        resolver
            .facts
            .item_definitions
            .insert((context.module, node, Namespace::Type), id);
        self.work.extend(resolver.facts);
        Self::direct(&mut resolver, &mut self.work, context, node, id)?;
        Ok(scope)
    }
    pub fn definition(
        &mut self,
        package: &mut Package,
        context: Context,
        node: NodeId,
        name: NameId,
        exported: bool,
        introduced: (usize, ExpansionId),
    ) -> Result<(), Error> {
        let (index, expansion) = introduced;
        let access = if exported {
            Access::Public
        } else {
            Access::Within(ModuleId(0))
        };
        let mut resolver = self.state.bind(package);
        let id = DefId(
            u32::try_from(resolver.facts.definitions.len())
                .map_err(|_| resolver.error(context, node, "too many macro-path definitions"))?,
        );
        resolver.facts.definitions.push(Definition {
            name,
            namespace: Namespace::Macro,
            kind: DefKind::Macro,
            scope: context.scope,
            node: Some(NodeKey {
                file: context.file,
                node,
            }),
            access,
            members: None,
        });
        if expansion.0 != 0 {
            resolver
                .facts
                .introductions
                .definitions
                .insert(id, expansion);
        }
        // Hygienic lookups from this macro's expansions resume at its scope.
        resolver
            .facts
            .node_scopes
            .insert(context.module, context.file, node, context.scope);
        self.macro_targets
            .insert(id, MacroTarget::Declarative(index));
        self.macros.resize(index + 1, None);
        self.macros[index] = Some(id);
        if exported {
            Self::direct(
                &mut resolver,
                &mut self.work,
                Context {
                    scope: ScopeId(0),
                    ..context
                },
                node,
                id,
            )?;
        }
        Ok(())
    }
    fn direct(
        resolver: &mut crate::resolve::index::Resolver<'_>,
        work: &mut Work,
        context: Context,
        node: NodeId,
        target: DefId,
    ) -> Result<(), Error> {
        let definition = &resolver.facts.definitions[target.0 as usize];
        let (name, namespace, access) = (definition.name, definition.namespace, definition.access);
        if resolver
            .facts
            .bindings
            .get(context.scope, name, namespace)
            .is_some_and(|b| !matches!(b.origin, Origin::Glob(_)))
        {
            return Err(resolver.error(context, node, "duplicate macro-path binding"));
        }
        resolver.publish(
            (context.scope, name, namespace, ItemContext::ROOT),
            Some(Binding {
                target,
                access,
                origin: Origin::Direct,
                ambiguous: false,
            }),
            work,
        )
    }
    pub fn import(
        &mut self,
        package: &mut Package,
        context: Context,
        node: NodeId,
        visibility: Visibility,
        textual: (TextScope, ExpansionId),
    ) -> Result<(), Error> {
        let first = self.state.facts.imports.len();
        let mut resolver = self.state.bind(package);
        let first_definition = resolver.facts.definitions.len();
        let access = resolver.visibility(context, visibility)?;
        resolver.import(context, node, &[], false, access)?;
        self.work.extend(resolver.facts);
        for definition in &resolver.facts.definitions[first_definition..] {
            self.work.declared(
                &resolver.facts.bindings,
                (
                    definition.scope,
                    definition.name,
                    definition.namespace,
                    ItemContext::ROOT,
                ),
            );
        }
        let library_builtin = resolver.facts.imports[first..].iter().any(|import| {
            import
                .path
                .last()
                .is_some_and(|name| resolver.package.names.get(*name) == Builtin::OptionEnv.name())
        });
        for index in first..resolver.facts.imports.len() {
            if textual.1.0 != 0 {
                resolver
                    .facts
                    .introductions
                    .imports
                    .insert(index, textual.1);
            }
            let import = &resolver.facts.imports[index];
            let text =
                (!import.absolute && !import.glob && import.path.len() == 1).then_some(textual);
            self.textual.push(text);
            self.text_queued.push(text.is_some());
            if text.is_some() {
                self.text_ready.push_back(index);
            }
        }
        if library_builtin {
            self.library_builtin(package, context, node)?;
        }
        Ok(())
    }
    pub fn wake_textual(&mut self, index: usize) {
        if !self.text_queued[index] {
            self.text_queued[index] = true;
            self.text_ready.push_back(index);
        }
    }
    pub fn drive(
        &mut self,
        package: &mut Package,
        scopes: &mut Scopes,
        bindings: Bindings<'_>,
    ) -> Result<Vec<usize>, Error> {
        while let Some(index) = self.text_ready.pop_front() {
            self.text_queued[index] = false;
            let Some((scope, context)) = self.textual[index] else {
                continue;
            };
            let import = &self.state.facts.imports[index];
            let name = import.path[0];
            let source = match bindings
                .textual(scopes, scope, name, context)
                .map_err(|e| Error::at(import.node.file, e.offset, e.message))?
            {
                textual::Lookup::Missing => TextualSource::Missing,
                textual::Lookup::Pending(scope) => {
                    scopes.watch(scope, Observer::Import(index));
                    TextualSource::Pending
                }
                textual::Lookup::Found(index) => {
                    let target = self.macros.get(index).copied().flatten().ok_or_else(|| {
                        Error::at(import.node.file, 0, "missing textual macro identity")
                    })?;
                    let definition = &self.state.facts.definitions[target.0 as usize];
                    TextualSource::Found(Binding {
                        target,
                        access: definition.access,
                        origin: Origin::Direct,
                        ambiguous: false,
                    })
                }
            };
            self.work.set_textual(index, source);
            if let TextualSource::Found(binding) = source {
                self.state.facts.textual_imports.insert(index, binding);
            } else {
                self.state.facts.textual_imports.remove(&index);
            }
        }
        self.state.bind(package).drive_imports(&mut self.work)?;
        let mut ready = Vec::new();
        for key in self.work.events.drain(..) {
            if !self.waiting.is_empty()
                && let Some(calls) = self.waiting.remove(&key)
            {
                let mut calls: Vec<_> = calls.into_iter().collect();
                calls.sort_unstable();
                ready.extend(calls);
            }
        }
        Ok(ready)
    }
    pub fn validate_textual(
        &mut self,
        package: &mut Package,
        call: &Call,
        request: usize,
        definition: usize,
    ) -> Result<(), Error> {
        let target = self
            .macros
            .get(definition)
            .copied()
            .flatten()
            .ok_or_else(|| call.diagnostic("missing textual macro identity"))?;
        let mut dependencies = Vec::new();
        let conflict = self.state.bind(package).textual_conflict(
            call.visit.path_scope,
            call.path[0],
            target,
            call.visit.introduction,
            &mut dependencies,
        );
        for key in dependencies {
            self.waiting.entry(key).or_default().insert(request);
        }
        if conflict {
            return Err(call.diagnostic("ambiguous textual and path-based macro binding"));
        }
        Ok(())
    }
    pub fn lookup(
        &mut self,
        package: &mut Package,
        call: &Call,
        request: usize,
    ) -> Result<Option<MacroTarget>, Error> {
        let context = Context {
            module: call.visit.module,
            file: call.file,
            scope: call.visit.path_scope,
            foreign: call.visit.foreign,
        };
        let node = call.visit.node;
        if call
            .path
            .last()
            .is_some_and(|name| package.names.get(*name) == Builtin::OptionEnv.name())
        {
            self.library_builtin(package, context, node)?;
        }
        let resolver = self.state.bind(package);
        let mut observed = Observations {
            selected: Some(Vec::new()),
            ..Observations::default()
        };
        let found = resolver.lookup_path(
            context,
            node,
            call.absolute,
            &call.path,
            LookupMode::Use(Namespace::Macro),
            Some(&mut observed),
        );
        let ambiguous =
            self.work
                .ambiguous_early_path(resolver.facts, &observed, call.visit.introduction);
        for key in observed.dependencies {
            self.waiting.entry(key).or_default().insert(request);
        }
        if ambiguous {
            return Err(call.diagnostic("ambiguous macro-expanded path binding"));
        }
        match found? {
            Lookup::Missing => Ok(None),
            Lookup::Found(binding) => self
                .macro_targets
                .get(&binding.target)
                .copied()
                .map(Some)
                .ok_or_else(|| resolver.error(context, node, "path does not name a macro")),
            Lookup::Associated(..) | Lookup::Member(..) | Lookup::SelfConstructor(..) => {
                Err(resolver.error(context, node, "associated items are not macro paths"))
            }
        }
    }
    fn library_builtin(
        &mut self,
        package: &mut Package,
        context: Context,
        node: NodeId,
    ) -> Result<(), Error> {
        // Source providers resolve their actual macro declarations through the
        // import worklist. A familiar final spelling cannot create a library.
        if package.source_core.is_some() {
            return Ok(());
        }
        let name = package
            .names
            .intern(Builtin::OptionEnv.name())
            .map_err(|e| Error::at(context.file, 0, e))?;
        if self.state.facts.macro_prelude.contains_key(&name) {
            return Ok(());
        }
        let mut resolver = self.state.bind(package);
        let scope = resolver.prepare_library_value(
            context,
            node,
            crate::support::library_types::LibraryValue::EnvironmentOption,
        )?;
        let target = resolver.define(
            Definition {
                name,
                namespace: Namespace::Macro,
                kind: DefKind::BuiltinMacro,
                scope,
                node: None,
                access: Access::Public,
                members: None,
            },
            false,
        )?;
        resolver.facts.macro_prelude.insert(name, target);
        self.macro_targets
            .insert(target, MacroTarget::Builtin(Builtin::OptionEnv));
        self.work.extend(resolver.facts);
        self.work.declared(
            &resolver.facts.bindings,
            (scope, name, Namespace::Macro, ItemContext::ROOT),
        );
        Ok(())
    }
    pub fn take_error(&mut self) -> Option<Error> {
        self.work.take_error()
    }
    /// The declared facts and the import worklist, which resolution finishes.
    pub fn finish(self) -> (crate::resolve::model::Resolution, Work, Vec<Pending>) {
        (self.state.facts, self.work, self.pending)
    }
}
