//! Import dependencies and glob deltas wake only the scopes and paths they affect.
mod trees;
use super::{
    index::{PRIMITIVES, Resolver},
    lookup::{BindingKey, Lookup, LookupMode, Observations, Selection},
    model::{Access, Binding, BindingTable, DefId, DefKind, Import, Namespace, Origin, ScopeId},
    names::NameId,
    package::Error,
};
use crate::{
    expand::marks::ItemContext,
    support::hash::{HashMap, HashSet},
};
use std::collections::VecDeque;
type Key = (NameId, Namespace, ItemContext);
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
    /// Per-scope change journals, materialized on first use (`journaled`) and
    /// sized on demand, so creating a scope costs the worklist nothing.
    changes: Vec<Vec<Key>>,
    journaled: Vec<bool>,
    /// The per-scope first-binding tails at creation. Capture four bytes per
    /// existing scope once; journals still materialize only on demand.
    snapshot: super::table::Snapshot,
    cursors: Vec<usize>,
    errors: Vec<Option<Error>>,
    candidates: HashMap<BindingKey, Vec<Binding>>,
    provided: Vec<HashSet<Key>>,
    textual: HashMap<usize, TextualSource>,
    attempts: usize,
    observe: bool,
    selections: Vec<Selection>,
    /// Imports naming `primitive` that ran before the primitive index existed;
    /// `finish_imports` retries them once it is published.
    primitive_paths: Vec<usize>,
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
            changes: Vec::new(),
            journaled: Vec::new(),
            snapshot: facts.bindings.snapshot(),
            cursors: vec![0; count],
            errors: (0..count).map(|_| None).collect(),
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
            primitive_paths: Vec::new(),
            events: Vec::new(),
        }
    }
    pub fn observe(&mut self) {
        self.observe = true;
    }
    pub fn extend(&mut self, facts: &super::model::Resolution) {
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
    pub fn declared(&mut self, bindings: &BindingTable, key: BindingKey) {
        self.journal(bindings, key.0);
        self.changes[key.0.0 as usize].push((key.1, key.2, key.3));
        self.changed(key);
    }
    /// Materialize a scope's journal from the keys it recorded before this state.
    fn journal(&mut self, bindings: &BindingTable, scope: ScopeId) {
        let index = scope.0 as usize;
        if index >= self.journaled.len() {
            self.journaled.resize(index + 1, false);
            self.changes.resize_with(index + 1, Vec::new);
        }
        if self.journaled[index] {
            return;
        }
        self.journaled[index] = true;
        let keys = bindings.keys_before(scope, &self.snapshot);
        self.changes[index].splice(0..0, keys);
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
        introduction: crate::expand::marks::ExpansionId,
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
                    if let Some(outer) = facts.bindings.get_key((scope, key.1, key.2, key.3)) {
                        return (outer.ambiguous || outer.target != binding.target)
                            && (glob || !marks.ancestor(defined, marks.binding(*outer)));
                    }
                    current = facts.scopes[scope.0 as usize].parent;
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
        if !self.waiting.is_empty()
            && let Some(waiters) = self.waiting.remove(&key)
        {
            let mut waiters: Vec<_> = waiters.into_iter().collect();
            waiters.sort_unstable();
            for index in waiters {
                self.wake(index);
            }
        }
        if !self.globs.is_empty()
            && let Some(globs) = self.globs.get(&key.0).cloned()
        {
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
        if self.facts.imports.is_empty() {
            return Ok(());
        }
        let mut work = Work::new(self.facts);
        self.drive_imports(&mut work)?;
        self.diagnose_imports(work)
    }
    /// Finish the expansion phase's import worklist instead of re-queueing
    /// every import: only imports woken by later declarations run again.
    pub fn finish_imports(&mut self, mut work: Work) -> Result<(), Error> {
        work.observe = false;
        work.extend(self.facts);
        for index in std::mem::take(&mut work.primitive_paths) {
            work.wake(index);
        }
        self.drive_imports(&mut work)?;
        self.diagnose_imports(work)
    }
    /// Report the first failed or unresolved import, in import order.
    fn diagnose_imports(&self, work: Work) -> Result<(), Error> {
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
    /// Demand-create language metadata a type-namespace import path may name
    /// and publish the new declarations to the worklist.
    fn prepare_import_path(
        &mut self,
        index: usize,
        import: &Import,
        c: super::index::Context,
        path: crate::resolve::paths::Identified<'_>,
        work: &mut Work,
    ) -> Result<(), Error> {
        let before = self.facts.definitions.len();
        // Expansion defines primitives late; a primitive first segment needs them now.
        if !import.absolute
            && self.facts.primitives.is_empty()
            && path
                .first()
                .is_some_and(|name| PRIMITIVES.contains(&self.package.names.get(*name)))
        {
            self.primitives()?;
        }
        if self.facts.primitives.is_empty()
            && path
                .iter()
                .skip(1)
                .any(|name| self.package.names.get(*name) == "primitive")
        {
            work.primitive_paths.push(index);
        }
        self.prepare_builtin_path(c, import.node.node, import.absolute, Namespace::Type, path)?;
        if self.facts.definitions.len() != before {
            work.extend(self.facts);
            for definition in &self.facts.definitions[before..] {
                work.declared(
                    &self.facts.bindings,
                    (
                        definition.scope,
                        definition.name,
                        definition.namespace,
                        ItemContext::ROOT,
                    ),
                );
            }
        }
        Ok(())
    }
    fn import_lookup(
        &mut self,
        index: usize,
        import: &Import,
        path: crate::resolve::paths::Identified<'_>,
        ns: Namespace,
        work: &mut Work,
    ) -> Result<(Lookup, super::package::ModuleId), Error> {
        if ns == Namespace::Macro
            && path.len() == 1
            && let Some(binding) = work.textual.get(&index).copied()
        {
            let found = match binding {
                TextualSource::Found(binding) => {
                    let mut dependencies = Vec::new();
                    let introduction = self
                        .facts
                        .introductions
                        .imports
                        .get(&index)
                        .copied()
                        .unwrap_or(crate::expand::marks::ExpansionId(0));
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
            };
            return Ok((found, self.facts.scopes[import.scope.0 as usize].module));
        }
        let c = self.context(
            self.facts.scopes[import.scope.0 as usize].module,
            import.scope,
        );
        if ns == Namespace::Type {
            self.prepare_import_path(index, import, c, path, work)?;
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
            .unwrap_or(crate::expand::marks::ExpansionId(0));
        let ambiguous = work.ambiguous_early_path(self.facts, &observed, introduction);
        let privacy = observed.privacy.unwrap_or(c.module);
        if let Some(mut selections) = observed.selected.take() {
            if !ambiguous
                && matches!(&result, Ok(Lookup::Found(_)))
                && let Some(imports) = self
                    .facts
                    .crates
                    .as_deref_mut()
                    .and_then(|imports| imports.availability.as_deref_mut())
            {
                imports.record(import.node, &selections)?;
            }
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
        result.map(|found| (found, privacy))
    }
    fn single_import(
        &mut self,
        index: usize,
        import: &Import,
        work: &mut Work,
    ) -> Result<(), Error> {
        let mut path = super::paths::Identified::imported(import);
        let self_import = path.len() > 1
            && path
                .last()
                .is_some_and(|n| self.package.names.get(*n) == "self");
        if self_import {
            path = path.prefix(path.len() - 1);
        }
        let (name, context) = import
            .alias
            .or_else(|| {
                path.last()
                    .map(|name| (*name, import.segment_context(path.len() - 1)))
            })
            .ok_or_else(|| self.import_error(import, "empty import"))?;
        let origin = Origin::Import(
            u32::try_from(index).map_err(|_| self.import_error(import, "too many imports"))?,
        );
        let mut inaccessible = None;
        for ns in Namespace::IMPORTABLE {
            if self_import && ns != Namespace::Type {
                continue;
            }
            let key = (import.scope, name, ns, context);
            let (found, privacy) = self.import_lookup(index, import, path, ns, work)?;
            match found {
                Lookup::Found(binding) => {
                    if !self.visible(binding.access, privacy)
                        || !self.can_reexport(binding.access, import.access)
                    {
                        inaccessible =
                            Some(self.import_error(import, "private import or widened visibility"));
                        self.withdraw_single(key, origin, work)?;
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
                    if let Some(previous) = self.facts.bindings.get_key(key)
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
                    )?;
                }
                Lookup::Missing => self.withdraw_single(key, origin, work)?,
                Lookup::Associated(..) | Lookup::Member(..) | Lookup::SelfConstructor(..) => {
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
    fn withdraw_single(
        &mut self,
        key: BindingKey,
        origin: Origin,
        work: &mut Work,
    ) -> Result<(), Error> {
        if self
            .facts
            .bindings
            .get_key(key)
            .is_some_and(|b| b.origin == origin)
        {
            let fallback = self.glob_binding(key, work);
            self.publish(key, fallback, work)?;
        }
        Ok(())
    }
    fn glob(&mut self, index: usize, import: &Import, work: &mut Work) -> Result<(), Error> {
        let (found, privacy) = self.import_lookup(
            index,
            import,
            super::paths::Identified::imported(import),
            Namespace::Type,
            work,
        )?;
        let source = match found {
            Lookup::Found(binding) => {
                if !self.visible(binding.access, privacy) {
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
                return self.disconnect_glob(index, import, work);
            }
            Lookup::Associated(..) | Lookup::Member(..) | Lookup::SelfConstructor(..) => {
                return Err(self.import_error(import, "glob target is not a module or enum"));
            }
        };
        if source == import.scope {
            return Err(self.import_error(import, "cannot glob-import a scope into itself"));
        }
        if work.sources[index] != Some(source) {
            self.disconnect_glob(index, import, work)?;
            work.sources[index] = Some(source);
            work.globs.entry(source).or_default().push(index);
        }
        work.resolved[index] = true;
        let module = self.facts.scopes[import.scope.0 as usize].module;
        work.journal(&self.facts.bindings, source);
        let end = work.changes[source.0 as usize].len();
        for offset in work.cursors[index]..end {
            let key = work.changes[source.0 as usize][offset];
            if !Namespace::IMPORTABLE.contains(&key.1) {
                continue;
            }
            let introductions = &self.facts.introductions;
            let glob = import
                .contexts
                .as_ref()
                .map_or(ItemContext::ROOT, |contexts| contexts.glob);
            let Some(adjusted) = self
                .package
                .marks
                .reverse_glob(
                    key.2,
                    introductions.scope(source),
                    glob,
                    &introductions.ancestry,
                )
                .map_err(|e| self.import_error(import, e))?
            else {
                continue;
            };
            let privacy = if let Some(expansion) = adjusted.definition {
                let scope = introductions
                    .definition_scope(expansion)
                    .map_err(|e| self.import_error(import, e))?;
                self.facts
                    .scopes
                    .get(scope.0 as usize)
                    .ok_or_else(|| self.import_error(import, "unknown glob privacy scope"))?
                    .module
            } else {
                module
            };
            let binding = self
                .facts
                .bindings
                .get_key((source, key.0, key.1, key.2))
                .copied()
                .filter(|b| self.visible(b.access, privacy));
            let binding = binding.map(|binding| Binding {
                access: if self.can_reexport(binding.access, import.access) {
                    import.access
                } else {
                    binding.access
                },
                origin: Origin::Glob(u32::try_from(index).unwrap_or(u32::MAX)),
                ..binding
            });
            self.contribute(
                index,
                (import.scope, key.0, key.1, adjusted.context),
                binding,
                work,
            )?;
        }
        work.cursors[index] = end;
        Ok(())
    }
    fn disconnect_glob(
        &mut self,
        index: usize,
        import: &Import,
        work: &mut Work,
    ) -> Result<(), Error> {
        if let Some(source) = work.sources[index].take()
            && let Some(subscribers) = work.globs.get_mut(&source)
        {
            subscribers.retain(|i| *i != index);
        }
        let provided = std::mem::take(&mut work.provided[index]);
        // Sort only cold removal events so randomized sets cannot affect observable work order.
        let mut keys: Vec<_> = provided.into_iter().collect();
        keys.sort_by_key(|key| (key.0.0, key.1.name(), key.2.key()));
        for key in keys {
            self.contribute(index, (import.scope, key.0, key.1, key.2), None, work)?;
        }
        work.cursors[index] = 0;
        Ok(())
    }
    fn contribute(
        &mut self,
        index: usize,
        key: BindingKey,
        binding: Option<Binding>,
        work: &mut Work,
    ) -> Result<(), Error> {
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
            work.provided[index].insert((key.1, key.2, key.3));
        } else {
            work.provided[index].remove(&(key.1, key.2, key.3));
        }
        if self
            .facts
            .bindings
            .get_key(key)
            .is_some_and(|b| !matches!(b.origin, Origin::Glob(_)))
        {
            if previous != binding {
                work.changed(key);
            }
            return Ok(());
        }
        let merged = self.glob_binding(key, work);
        self.publish(key, merged, work)
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
    pub fn publish(
        &mut self,
        key: BindingKey,
        binding: Option<Binding>,
        work: &mut Work,
    ) -> Result<(), Error> {
        work.journal(&self.facts.bindings, key.0);
        let (scope, name, namespace, context) = key;
        let previous = self.facts.bindings.get_key(key).copied();
        if previous == binding {
            return Ok(());
        }
        if let Some(binding) = binding {
            self.facts
                .bindings
                .insert_key(key, binding)
                .map_err(|e| Error::at(super::unit::FileId(0), 0, e))?;
        } else {
            self.facts.bindings.remove_key(key);
        }
        work.changes[scope.0 as usize].push((name, namespace, context));
        work.changed(key);
        Ok(())
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

#[cfg(test)]
#[path = "../../tests/invariants/item_journals.rs"]
mod tests;

#[cfg(test)]
#[path = "../../tests/invariants/item_globs.rs"]
mod glob_tests;
