//! Trait lookup consumes final lexical bindings and shares inherited candidate lists.
use crate::{
    resolve::{
        expansion::Report,
        model::{DefId, DefKind, Namespace, ScopeId},
        names::NameId,
    },
    sema::model::Declarations,
    types::ItemId,
};
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

#[derive(Default)]
pub(super) struct Index {
    members: HashMap<DefId, HashMap<NameId, DefId>>,
    scopes: HashMap<ScopeId, HashSet<DefId>>,
    local: HashMap<ScopeId, HashMap<NameId, Vec<DefId>>>,
    lexical: HashMap<(ScopeId, NameId), Arc<[DefId]>>,
    work: Vec<ScopeId>,
    steps: usize,
    prelude: HashMap<NameId, DefId>,
}
impl Index {
    pub(super) fn new(report: &Report, facts: &Declarations) -> Result<Self, &'static str> {
        let mut index = Self::default();
        for (member, _) in facts.associated.declaration_members() {
            index.account(1)?;
            let Some(owner) = facts.associated.trait_owner(member) else {
                continue;
            };
            let definition = &report.resolution.definitions[member.0 as usize];
            if matches!(definition.kind, DefKind::Function | DefKind::Const) {
                index
                    .members
                    .entry(owner)
                    .or_default()
                    .insert(definition.name, member);
            }
        }
        for kind in crate::support::comparisons::Comparison::ALL {
            if let Some(declaration) = report.resolution.builtin_traits.comparison(kind)
                && let Some(members) = index.members.get(&declaration.definition)
            {
                index
                    .prelude
                    .extend(members.iter().map(|(name, member)| (*name, *member)));
                let count = members.len();
                index.account(count)?;
            }
        }
        for (ordinal, scope) in report.resolution.scopes.iter().enumerate() {
            index.account(1)?;
            let scope_id = u32::try_from(ordinal)
                .map(ScopeId)
                .map_err(|_| "trait visibility scope space exhausted")?;
            for (&(_, namespace), binding) in &scope.bindings {
                index.account(1)?;
                // Method lookup uses the retained glob candidate even when naming
                // the trait directly would be ambiguous. Direct bindings replace it.
                if namespace == Namespace::Type
                    && (report.resolution.definitions[binding.target.0 as usize].kind
                        == DefKind::Trait
                        || (facts.associated.operation(binding.target).is_some()
                            || facts.associated.comparison(binding.target).is_some()))
                {
                    index
                        .scopes
                        .entry(scope_id)
                        .or_default()
                        .insert(binding.target);
                }
            }
        }
        for (&import, binding) in &report.resolution.anonymous_types {
            index.account(1)?;
            if report.resolution.definitions[binding.target.0 as usize].kind == DefKind::Trait
                || (facts.associated.operation(binding.target).is_some()
                    || facts.associated.comparison(binding.target).is_some())
            {
                let scope = report.resolution.imports[import].scope;
                index
                    .scopes
                    .entry(scope)
                    .or_default()
                    .insert(binding.target);
            }
        }
        Ok(index)
    }
    pub(super) const fn steps(&self) -> usize {
        self.steps
    }
    pub(super) fn groups(
        &mut self,
        report: &Report,
        scope: ScopeId,
        name: NameId,
        bound_traits: &[ItemId],
    ) -> Result<[Vec<DefId>; 2], &'static str> {
        self.account(bound_traits.len())?;
        let mut bound: Vec<_> = bound_traits
            .iter()
            .filter_map(|owner| self.members.get(&DefId(owner.0))?.get(&name).copied())
            .collect();
        bound.sort_unstable_by_key(|id| id.0);
        bound.dedup();
        let inherited = self.visible(report, scope, name)?;
        self.account(inherited.len())?;
        // A bound method can also be a lexical candidate with a different Self,
        // such as an implementation for a reference to the bounded parameter.
        let mut lexical = inherited.to_vec();
        if !self.prelude.is_empty() {
            self.account(1)?;
            if report.package.modules[report.resolution.scopes[scope.0 as usize].module.0 as usize]
                .implicit_prelude
                && let Some(&method) = self.prelude.get(&name)
                && let Err(index) = lexical.binary_search_by_key(&method.0, |id| id.0)
            {
                self.account(lexical.len().saturating_add(1))?;
                lexical.insert(index, method);
            }
        }
        Ok([bound, lexical])
    }
    fn visible(
        &mut self,
        report: &Report,
        scope: ScopeId,
        name: NameId,
    ) -> Result<Arc<[DefId]>, &'static str> {
        self.work.clear();
        let mut current = Some(scope);
        let mut result: Arc<[DefId]> = Arc::from([]);
        while let Some(scope) = current {
            self.account(1)?;
            if let Some(cached) = self.lexical.get(&(scope, name)) {
                result = Arc::clone(cached);
                break;
            }
            self.work.push(scope);
            current = report.resolution.scopes[scope.0 as usize].parent;
        }
        while let Some(scope) = self.work.pop() {
            self.index_scope(scope)?;
            if let Some(local) = self.local.get(&scope).and_then(|names| names.get(&name)) {
                let cost = local.len().saturating_add(result.len());
                let mut combined = result.to_vec();
                combined.extend_from_slice(local);
                self.account(cost)?;
                combined.sort_unstable_by_key(|id| id.0);
                combined.dedup();
                result = Arc::from(combined);
            }
            self.lexical.insert((scope, name), Arc::clone(&result));
        }
        Ok(result)
    }
    fn index_scope(&mut self, scope: ScopeId) -> Result<(), &'static str> {
        let Some(traits) = self.scopes.remove(&scope) else {
            return Ok(());
        };
        let mut names: HashMap<NameId, Vec<DefId>> = HashMap::new();
        for owner in traits {
            self.account(1 + self.members.get(&owner).map_or(0, HashMap::len))?;
            if let Some(members) = self.members.get(&owner) {
                for (&name, &method) in members {
                    names.entry(name).or_default().push(method);
                }
            }
        }
        self.local.insert(scope, names);
        Ok(())
    }
    fn account(&mut self, amount: usize) -> Result<(), &'static str> {
        self.steps = self.steps.saturating_add(amount);
        if self.steps > 1_000_000 {
            return Err("trait visibility work budget exceeded");
        }
        Ok(())
    }
}
