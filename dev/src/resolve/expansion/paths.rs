//! Early macro paths reuse the ordinary import dependency graph and glob journals.
use super::{Call, Error, ModuleId, NodeId, NodeKey, Package, bindings::Bindings};
use crate::{
    expand::{
        stream::ContextId,
        textual::{self, Observer, Scope as TextScope, Scopes},
    },
    resolve::{
        imports::{TextualSource, Work},
        index::{Context, State},
        lookup::{BindingKey, Lookup, LookupMode, Observations},
        model::{Access, Binding, DefId, DefKind, Definition, Namespace, Origin, ScopeId},
    },
    support::names::NameId,
    syntax::item::Visibility,
};
use std::collections::{HashMap, HashSet, VecDeque};

pub(super) struct Paths {
    pub state: State,
    work: Work,
    macros: Vec<DefId>,
    macro_targets: HashMap<DefId, usize>,
    textual: Vec<Option<(TextScope, ContextId)>>,
    text_ready: VecDeque<usize>,
    text_queued: Vec<bool>,
    waiting: HashMap<BindingKey, HashSet<usize>>,
}
impl Paths {
    pub fn new(package: &mut Package) -> Result<Self, Error> {
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
        let mut work = Work::new(&state.facts);
        work.observe();
        Ok(Self {
            state,
            work,
            macros: Vec::new(),
            macro_targets: HashMap::new(),
            textual: Vec::new(),
            text_ready: VecDeque::new(),
            text_queued: Vec::new(),
            waiting: HashMap::new(),
        })
    }
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
        self.state.facts.node_scopes.insert((module, node), scope);
        self.work.extend(&self.state.facts);
        Ok(scope)
    }
    pub fn module(
        &mut self,
        package: &mut Package,
        context: Context,
        child: ModuleId,
        node: NodeId,
        visibility: Visibility,
        expansion: ContextId,
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
            self.work.mark_definition(id);
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
        expansion: ContextId,
    ) -> Result<(), Error> {
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
            self.work.mark_definition(id);
        }
        self.macro_targets.insert(id, self.macros.len());
        self.macros.push(id);
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
        if resolver.facts.scopes[context.scope.0 as usize]
            .bindings
            .get(&(name, namespace))
            .is_some_and(|b| !matches!(b.origin, Origin::Glob(_)))
        {
            return Err(resolver.error(context, node, "duplicate macro-path binding"));
        }
        resolver.publish(
            (context.scope, name, namespace),
            Some(Binding {
                target,
                access,
                origin: Origin::Direct,
                ambiguous: false,
            }),
            work,
        );
        Ok(())
    }
    pub fn import(
        &mut self,
        package: &mut Package,
        context: Context,
        node: NodeId,
        visibility: Visibility,
        textual: (TextScope, ContextId),
    ) -> Result<(), Error> {
        let first = self.state.facts.imports.len();
        let mut resolver = self.state.bind(package);
        let access = resolver.visibility(context, visibility)?;
        resolver.import(context, node, &[], false, access)?;
        self.work.extend(resolver.facts);
        for index in first..resolver.facts.imports.len() {
            if textual.1.0 != 0 {
                self.work.mark_import(index);
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
                    let target = self.macros[index];
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
            if let Some(calls) = self.waiting.remove(&key) {
                let mut calls: Vec<_> = calls.into_iter().collect();
                calls.sort_unstable();
                ready.extend(calls);
            }
        }
        Ok(ready)
    }
    pub fn lookup(
        &mut self,
        package: &mut Package,
        call: &Call,
        request: usize,
    ) -> Result<Option<usize>, Error> {
        let context = Context {
            module: call.visit.module,
            file: call.file,
            scope: call.visit.path_scope,
        };
        let node = call.visit.node;
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
        let ambiguous = self.work.ambiguous_expanded_path(&observed);
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
            Lookup::Associated(_) => {
                Err(resolver.error(context, node, "associated items are not macro paths"))
            }
        }
    }
    pub fn take_error(&mut self) -> Option<Error> {
        self.work.take_error()
    }
}
