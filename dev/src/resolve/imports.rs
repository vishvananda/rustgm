//! Import dependencies and glob deltas wake only the scopes and paths they affect.
use super::{
    index::Resolver,
    lookup::{BindingKey, Lookup, LookupMode, Observations, Selection},
    model::{Access, Binding, DefId, DefKind, Import, Namespace, Origin, ScopeId},
    names::NameId,
    package::Error,
};
use crate::support::hash::{HashMap, HashSet};
use std::collections::VecDeque;
type Key = (NameId, Namespace);
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum TextualSource {
    Missing,
    Pending,
    Found(Binding),
}
pub(crate) struct Work {
    queue: VecDeque<usize>,
    queued: Vec<bool>,
    waiting: HashMap<BindingKey, HashSet<usize>>,
    globs: HashMap<ScopeId, Vec<usize>>,
    sources: Vec<Option<ScopeId>>,
    resolved: Vec<bool>,
    changes: Vec<Vec<Key>>,
    cursors: Vec<usize>,
    errors: Vec<Option<Error>>,
    recorded: HashSet<BindingKey>,
    candidates: HashMap<BindingKey, Vec<Binding>>,
    provided: Vec<HashSet<Key>>,
    textual: HashMap<usize, TextualSource>,
    attempts: usize,
    observe: bool,
    selections: Vec<Selection>,
    pub events: Vec<BindingKey>,
}
impl Work {
    pub fn new(facts: &super::model::Resolution) -> Self {
        let count = facts.imports.len();
        Self {
            queue: (0..count).collect(),
            queued: vec![true; count],
            waiting: HashMap::default(),
            globs: HashMap::default(),
            sources: vec![None; count],
            resolved: vec![false; count],
            changes: facts.scopes.iter().map(|s| s.order.clone()).collect(),
            cursors: vec![0; count],
            errors: (0..count).map(|_| None).collect(),
            recorded: facts
                .scopes
                .iter()
                .enumerate()
                .flat_map(|(index, scope)| {
                    scope.order.iter().map(move |key| {
                        (
                            ScopeId(u32::try_from(index).unwrap_or(u32::MAX)),
                            key.0,
                            key.1,
                        )
                    })
                })
                .collect(),
            candidates: HashMap::default(),
            provided: (0..count).map(|_| HashSet::default()).collect(),
            textual: facts
                .textual_imports
                .iter()
                .map(|(index, binding)| (*index, TextualSource::Found(*binding)))
                .collect(),
            attempts: 0,
            observe: false,
            selections: Vec::new(),
            events: Vec::new(),
        }
    }
    pub fn observe(&mut self) {
        self.observe = true;
    }
    pub fn extend(&mut self, facts: &super::model::Resolution) {
        self.changes.resize_with(facts.scopes.len(), Vec::new);
        for index in self.resolved.len()..facts.imports.len() {
            self.queue.push_back(index);
            self.queued.push(true);
            self.sources.push(None);
            self.resolved.push(false);
            self.cursors.push(0);
            self.errors.push(None);
            self.provided.push(HashSet::default());
        }
    }
    /// Publish a direct declaration already installed by the shared item indexer.
    /// Its scope order was recorded by `define`; only dependency journals remain.
    pub fn declared(&mut self, key: BindingKey) {
        self.recorded.insert(key);
        self.changes[key.0.0 as usize].push((key.1, key.2));
        self.changed(key);
    }
    pub fn set_textual(&mut self, index: usize, binding: TextualSource) {
        let previous = self
            .textual
            .get(&index)
            .copied()
            .unwrap_or(TextualSource::Missing);
        if binding == TextualSource::Missing {
            self.textual.remove(&index);
        } else {
            self.textual.insert(index, binding);
        }
        if previous != binding {
            self.wake(index);
        }
    }
    pub fn take_error(&mut self) -> Option<Error> {
        self.errors.iter_mut().find_map(Option::take)
    }
    fn glob_conflicts(&self, key: BindingKey, target: DefId) -> bool {
        self.candidates.get(&key).is_some_and(|candidates| {
            candidates
                .iter()
                .any(|candidate| candidate.target != target || candidate.ambiguous)
        })
    }
    pub fn ambiguous_early_path(
        &self,
        facts: &super::model::Resolution,
        observations: &Observations,
        introduction: crate::expand::stream::ContextId,
    ) -> bool {
        observations.selected.iter().flatten().any(|selected| {
            let (key, binding) = (selected.key, selected.binding);
            let marks = &facts.introductions;
            let defined = marks.binding(binding);
            let glob = matches!(binding.origin, Origin::Glob(_));
            if !glob && defined.0 != 0 && self.glob_conflicts(key, binding.target) {
                return true;
            }
            let more_expanded = !marks.ancestor(defined, introduction);
            if selected.lexical && (glob || more_expanded) {
                let mut current = facts.scopes[key.0.0 as usize].parent;
                while let Some(scope) = current {
                    let frame = &facts.scopes[scope.0 as usize];
                    if let Some(outer) = frame.bindings.get(&(key.1, key.2)) {
                        return (outer.ambiguous || outer.target != binding.target)
                            && (glob || !marks.ancestor(defined, marks.binding(*outer)));
                    }
                    current = frame.parent;
                }
            }
            false
        })
    }

    pub fn wake(&mut self, index: usize) {
        if !self.queued[index] {
            self.queued[index] = true;
            self.queue.push_back(index);
        }
    }
    fn changed(&mut self, key: BindingKey) {
        if self.observe {
            self.events.push(key);
        }
        if let Some(waiters) = self.waiting.remove(&key) {
            let mut waiters: Vec<_> = waiters.into_iter().collect();
            waiters.sort_unstable();
            for index in waiters {
                self.wake(index);
            }
        }
        if let Some(globs) = self.globs.get(&key.0).cloned() {
            for index in globs {
                self.wake(index);
            }
        }
    }
    fn watch(&mut self, index: usize, keys: Vec<BindingKey>) {
        for key in keys {
            let waiting = self.waiting.entry(key).or_default();
            waiting.insert(index);
        }
    }
}
impl Resolver<'_> {
    pub fn imports(&mut self) -> Result<(), Error> {
        let mut work = Work::new(self.facts);
        self.drive_imports(&mut work)?;
        for (index, error) in work.errors.into_iter().enumerate() {
            if let Some(error) = error {
                return Err(error);
            }
            if !work.resolved[index] {
                return Err(self.import_error(&self.facts.imports[index], "unresolved import"));
            }
        }
        Ok(())
    }
    pub fn drive_imports(&mut self, work: &mut Work) -> Result<(), Error> {
        let count = self.facts.imports.len();
        while let Some(index) = work.queue.pop_front() {
            work.attempts += 1;
            work.queued[index] = false;
            work.resolved[index] = false;
            self.facts.anonymous_types.remove(&index);
            let import = self.facts.imports[index].clone();
            if work.attempts > 1_000_000 + count.saturating_mul(128) {
                return Err(self.import_error(&import, "import work budget exceeded"));
            }
            let result = if import.glob {
                self.glob(index, &import, work)
            } else {
                self.single_import(index, &import, work)
            };
            // Ambiguity/private failures can be provisional while a stronger import is pending.
            work.errors[index] = result.err();
        }
        Ok(())
    }
    fn import_error(&self, import: &Import, message: &str) -> Error {
        self.error(
            self.context(
                self.facts.scopes[import.scope.0 as usize].module,
                import.scope,
            ),
            import.node.node,
            message,
        )
    }
    fn import_lookup(
        &mut self,
        index: usize,
        import: &Import,
        path: &[NameId],
        ns: Namespace,
        work: &mut Work,
    ) -> Result<Lookup, Error> {
        if ns == Namespace::Macro
            && path.len() == 1
            && let Some(binding) = work.textual.get(&index).copied()
        {
            return Ok(match binding {
                TextualSource::Found(binding) => {
                    let mut dependencies = Vec::new();
                    let introduction = self
                        .facts
                        .introductions
                        .imports
                        .get(&index)
                        .copied()
                        .unwrap_or(crate::expand::stream::ContextId(0));
                    let conflict = self.textual_conflict(
                        import.scope,
                        path[0],
                        binding.target,
                        introduction,
                        &mut dependencies,
                    );
                    work.watch(index, dependencies);
                    if conflict {
                        return Err(self.import_error(
                            import,
                            "ambiguous textual and path-based macro binding",
                        ));
                    }
                    Lookup::Found(binding)
                }
                TextualSource::Missing | TextualSource::Pending => Lookup::Missing,
            });
        }
        let c = self.context(
            self.facts.scopes[import.scope.0 as usize].module,
            import.scope,
        );
        if ns == Namespace::Type {
            let before = self.facts.definitions.len();
            self.prepare_builtin_path(c, import.node.node, import.absolute, Namespace::Type, path)?;
            if self.facts.definitions.len() != before {
                work.extend(self.facts);
                for definition in &self.facts.definitions[before..] {
                    work.declared((definition.scope, definition.name, definition.namespace));
                }
            }
        }
        let mut observed = Observations {
            selected: Some(std::mem::take(&mut work.selections)),
            ..Observations::default()
        };
        let result = self.lookup_path(
            c,
            import.node.node,
            import.absolute,
            path,
            LookupMode::Import(ns),
            Some(&mut observed),
        );
        let introduction = self
            .facts
            .introductions
            .imports
            .get(&index)
            .copied()
            .unwrap_or(crate::expand::stream::ContextId(0));
        let ambiguous = work.ambiguous_early_path(self.facts, &observed, introduction);
        if let Some(mut selections) = observed.selected.take() {
            selections.clear();
            work.selections = selections;
        }
        work.watch(index, observed.dependencies);
        if let Ok(Lookup::Found(binding)) = &result
            && self.facts.definitions[binding.target.0 as usize].kind == DefKind::BuiltinMacro
            && self
                .package
                .names
                .get(self.facts.definitions[binding.target.0 as usize].name)
                == "cfg"
        {
            return Err(self.import_error(import, "ambiguous builtin cfg import"));
        }
        if ambiguous {
            return Err(self.import_error(import, "ambiguous import path binding"));
        }
        result
    }
    fn single_import(
        &mut self,
        index: usize,
        import: &Import,
        work: &mut Work,
    ) -> Result<(), Error> {
        let mut path = import.path.as_slice();
        let self_import = path.len() > 1
            && path
                .last()
                .is_some_and(|n| self.package.names.get(*n) == "self");
        if self_import {
            path = &path[..path.len() - 1];
        }
        let name = import
            .alias
            .or_else(|| path.last().copied())
            .ok_or_else(|| self.import_error(import, "empty import"))?;
        let origin = Origin::Import(
            u32::try_from(index).map_err(|_| self.import_error(import, "too many imports"))?,
        );
        let mut inaccessible = None;
        let module = self.facts.scopes[import.scope.0 as usize].module;
        for ns in Namespace::IMPORTABLE {
            if self_import && ns != Namespace::Type {
                continue;
            }
            match self.import_lookup(index, import, path, ns, work)? {
                Lookup::Found(binding) => {
                    if !self.visible(binding.access, module)
                        || !self.can_reexport(binding.access, import.access)
                    {
                        inaccessible =
                            Some(self.import_error(import, "private import or widened visibility"));
                        self.withdraw_single((import.scope, name, ns), origin, work);
                        continue;
                    }
                    work.resolved[index] = true;
                    if self.package.names.get(name) == "_" {
                        if ns == Namespace::Type {
                            self.facts.anonymous_types.insert(
                                index,
                                Binding {
                                    access: import.access,
                                    origin,
                                    ..binding
                                },
                            );
                        }
                        continue;
                    }
                    let key = (import.scope, name, ns);
                    if let Some(previous) = self.facts.scopes[import.scope.0 as usize]
                        .bindings
                        .get(&(name, ns))
                        && !matches!(previous.origin, Origin::Glob(_))
                        && previous.origin != origin
                    {
                        return Err(self.import_error(
                            import,
                            &format!(
                                "duplicate imported {} name `{}`",
                                ns.name(),
                                self.package.names.get(name)
                            ),
                        ));
                    }
                    self.publish(
                        key,
                        Some(Binding {
                            access: import.access,
                            origin,
                            ..binding
                        }),
                        work,
                    );
                }
                Lookup::Missing => self.withdraw_single((import.scope, name, ns), origin, work),
                Lookup::Associated(..) | Lookup::Member(..) => {
                    return Err(self.import_error(import, "associated items cannot be imported"));
                }
            }
        }
        if !work.resolved[index]
            && let Some(error) = inaccessible
        {
            return Err(error);
        }
        Ok(())
    }
    fn withdraw_single(&mut self, key: BindingKey, origin: Origin, work: &mut Work) {
        if self.facts.scopes[key.0.0 as usize]
            .bindings
            .get(&(key.1, key.2))
            .is_some_and(|b| b.origin == origin)
        {
            let fallback = self.glob_binding(key, work);
            self.publish(key, fallback, work);
        }
    }
    fn glob(&mut self, index: usize, import: &Import, work: &mut Work) -> Result<(), Error> {
        let found = self.import_lookup(index, import, &import.path, Namespace::Type, work)?;
        let source = match found {
            Lookup::Found(binding) => {
                let module = self.facts.scopes[import.scope.0 as usize].module;
                if !self.visible(binding.access, module) {
                    return Err(self.import_error(import, "private glob target"));
                }
                let definition = &self.facts.definitions[binding.target.0 as usize];
                if !matches!(definition.kind, DefKind::Module | DefKind::Enum) {
                    return Err(self.import_error(import, "glob target is not a module or enum"));
                }
                definition
                    .members
                    .ok_or_else(|| self.import_error(import, "missing glob scope"))?
            }
            Lookup::Missing => {
                self.disconnect_glob(index, import, work);
                return Ok(());
            }
            Lookup::Associated(..) | Lookup::Member(..) => {
                return Err(self.import_error(import, "glob target is not a module or enum"));
            }
        };
        if source == import.scope {
            return Err(self.import_error(import, "cannot glob-import a scope into itself"));
        }
        if work.sources[index] != Some(source) {
            self.disconnect_glob(index, import, work);
            work.sources[index] = Some(source);
            work.globs.entry(source).or_default().push(index);
        }
        work.resolved[index] = true;
        let module = self.facts.scopes[import.scope.0 as usize].module;
        let end = work.changes[source.0 as usize].len();
        for offset in work.cursors[index]..end {
            let key = work.changes[source.0 as usize][offset];
            if !Namespace::IMPORTABLE.contains(&key.1) {
                continue;
            }
            let binding = self.facts.scopes[source.0 as usize]
                .bindings
                .get(&key)
                .copied()
                .filter(|b| self.visible(b.access, module));
            let binding = binding.map(|binding| Binding {
                access: if self.can_reexport(binding.access, import.access) {
                    import.access
                } else {
                    binding.access
                },
                origin: Origin::Glob(u32::try_from(index).unwrap_or(u32::MAX)),
                ..binding
            });
            self.contribute(index, (import.scope, key.0, key.1), binding, work);
        }
        work.cursors[index] = end;
        Ok(())
    }
    fn disconnect_glob(&mut self, index: usize, import: &Import, work: &mut Work) {
        if let Some(source) = work.sources[index].take()
            && let Some(subscribers) = work.globs.get_mut(&source)
        {
            subscribers.retain(|i| *i != index);
        }
        let provided = std::mem::take(&mut work.provided[index]);
        // Sort only cold removal events so randomized sets cannot affect observable work order.
        let mut keys: Vec<_> = provided.into_iter().collect();
        keys.sort_by_key(|key| (key.0.0, key.1.name()));
        for key in keys {
            self.contribute(index, (import.scope, key.0, key.1), None, work);
        }
        work.cursors[index] = 0;
    }
    fn contribute(
        &mut self,
        index: usize,
        key: BindingKey,
        binding: Option<Binding>,
        work: &mut Work,
    ) {
        let origin = Origin::Glob(u32::try_from(index).unwrap_or(u32::MAX));
        let candidates = work.candidates.entry(key).or_default();
        let previous = candidates.iter().find(|b| b.origin == origin).copied();
        if let Some(position) = candidates.iter().position(|b| b.origin == origin) {
            if let Some(binding) = binding {
                candidates[position] = binding;
            } else {
                candidates.remove(position);
            }
        } else if let Some(binding) = binding {
            candidates.push(binding);
        }
        if binding.is_some() {
            work.provided[index].insert((key.1, key.2));
        } else {
            work.provided[index].remove(&(key.1, key.2));
        }
        if self.facts.scopes[key.0.0 as usize]
            .bindings
            .get(&(key.1, key.2))
            .is_some_and(|b| !matches!(b.origin, Origin::Glob(_)))
        {
            if previous != binding {
                work.changed(key);
            }
            return;
        }
        let merged = self.glob_binding(key, work);
        self.publish(key, merged, work);
    }
    fn glob_binding(&self, key: BindingKey, work: &Work) -> Option<Binding> {
        let candidates = work.candidates.get(&key)?;
        let mut result = *candidates.iter().min_by_key(|binding| {
            if let Origin::Glob(id) = binding.origin {
                id
            } else {
                u32::MAX
            }
        })?;
        for candidate in candidates {
            result.ambiguous |= candidate.ambiguous || candidate.target != result.target;
            result.access = self.access_union(result.access, candidate.access);
        }
        Some(result)
    }
    pub fn publish(&mut self, key: BindingKey, binding: Option<Binding>, work: &mut Work) {
        let frame = &mut self.facts.scopes[key.0.0 as usize];
        let name = (key.1, key.2);
        let previous = frame.bindings.get(&name).copied();
        if previous == binding {
            return;
        }
        if let Some(binding) = binding {
            if work.recorded.insert(key) {
                frame.order.push(name);
            }
            frame.bindings.insert(name, binding);
        } else {
            frame.bindings.remove(&name);
        }
        work.changes[key.0.0 as usize].push(name);
        work.changed(key);
    }
    fn can_reexport(&self, target: Access, requested: Access) -> bool {
        match (target, requested) {
            (Access::ExternalPrivate, _)
            | (_, Access::ExternalPrivate)
            | (Access::Within(_), Access::Public) => false,
            (Access::Public, _) => true,
            (Access::Within(target), Access::Within(requested)) => {
                self.package.is_descendant(requested, target)
            }
        }
    }
    fn access_union(&self, a: Access, b: Access) -> Access {
        match (a, b) {
            (Access::ExternalPrivate, _) | (_, Access::ExternalPrivate) => Access::ExternalPrivate,
            (Access::Public, _) | (_, Access::Public) => Access::Public,
            (Access::Within(a), Access::Within(b)) => {
                if self.package.is_descendant(a, b) {
                    Access::Within(b)
                } else {
                    Access::Within(a)
                }
            }
        }
    }
}
