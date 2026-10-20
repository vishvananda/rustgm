//! Callable input scopes own normalizer caches, selected equations and result restoration.
use super::{Context, Declarations, Query, inference};
use crate::types::{
    TypeId,
    lifetime_proof::{ScopeId, map_scopes},
    substitute::{Bound, Normalizer, RegionEquations, Resolution},
};
use std::collections::{HashMap, HashSet};

struct Closure {
    fold: Bound,
}
pub(super) struct Local {
    pub scope: ScopeId,
    bindings: Vec<ScopeId>,
    root: Option<Closure>,
    requests: HashMap<ScopeId, Closure>,
}
pub(super) struct Scoped {
    pub active: Local,
    saved: HashMap<ScopeId, (Normalizer, Local)>,
    caller_end: u32,
    seen: HashSet<usize>,
    equations: RegionEquations,
}
impl Local {
    fn new(scope: ScopeId) -> Self {
        Self {
            scope,
            bindings: Vec::new(),
            root: None,
            requests: HashMap::new(),
        }
    }
}
impl Scoped {
    pub fn new(scope: ScopeId, caller_end: u32) -> Self {
        Self {
            active: Local::new(scope),
            saved: HashMap::new(),
            caller_end,
            seen: HashSet::new(),
            equations: RegionEquations::default(),
        }
    }

    pub fn at_root(
        query: &Query,
        context: &mut Context,
        scope: ScopeId,
        parent: ScopeId,
        caller_end: u32,
    ) -> Result<Self, String> {
        let mut scoped = Self::new(scope, caller_end);
        if scope != parent {
            scoped.active.root = Some(Closure::new(query, context, scope, parent, caller_end)?);
        }
        Ok(scoped)
    }

    pub fn enter(
        &mut self,
        query: &Query,
        context: &mut Context,
        fold: &mut Normalizer,
        scope: ScopeId,
        parent: ScopeId,
    ) -> Result<(), String> {
        if self.active.scope == scope {
            return Ok(());
        }
        let next = if let Some(saved) = self.saved.remove(&scope) {
            saved
        } else {
            let root = if scope == parent {
                None
            } else {
                Some(Closure::new(
                    query,
                    context,
                    scope,
                    parent,
                    self.caller_end,
                )?)
            };
            (
                Normalizer::default(),
                Local {
                    root,
                    ..Local::new(scope)
                },
            )
        };
        let old = std::mem::replace(&mut self.active, next.1);
        self.saved
            .insert(old.scope, (std::mem::replace(fold, next.0), old));
        Ok(())
    }

    pub fn request(
        &mut self,
        query: &mut Query,
        context: &mut Context,
        fold: &Normalizer,
        scope: ScopeId,
    ) -> Result<ScopeId, String> {
        let target = query.map_normalization_scopes(
            &mut self.active.bindings,
            self.active.scope,
            scope,
            |id| fold.binder(id).ok_or("missing normalization binder"),
            context,
        )?;
        if target != self.active.scope && !self.active.requests.contains_key(&target) {
            let closure = Closure::new(query, context, target, self.active.scope, self.caller_end)?;
            self.active.requests.insert(target, closure);
        }
        Ok(target)
    }

    pub fn selected(
        &mut self,
        query: &Query,
        facts: &mut Declarations,
        context: &mut Context,
        scope: ScopeId,
        node: usize,
        resolution: Resolution,
    ) -> Result<Resolution, String> {
        self.collect(query, context, node)?;
        if scope != self.active.scope
            && let Resolution::Type(ty) = resolution
        {
            return self
                .active
                .requests
                .get_mut(&scope)
                .ok_or("missing callable result closure")?
                .close(facts, context, &mut self.equations, ty)
                .map(Resolution::Type);
        }
        Ok(resolution)
    }

    pub fn complete(
        &mut self,
        facts: &mut Declarations,
        context: &mut Context,
        ty: TypeId,
    ) -> Result<TypeId, String> {
        self.active.root.as_mut().map_or(Ok(ty), |closure| {
            closure.close(facts, context, &mut self.equations, ty)
        })
    }

    fn collect(&mut self, query: &Query, context: &mut Context, node: usize) -> Result<(), String> {
        let scopes = query
            .scopes
            .as_ref()
            .ok_or("missing selected proof scopes")?;
        let mut pending = vec![node];
        while let Some(index) = pending.pop() {
            context.account_work(1).map_err(inference)?;
            if !self.seen.insert(index) {
                continue;
            }
            let node = &query.nodes[index];
            context
                .account_work(node.children.len())
                .map_err(inference)?;
            pending.extend(node.children.iter().copied());
            let scope = query.node_scope(index);
            for &(left, right) in &node.equalities {
                let (left, left_work) = scopes.equation_region(scope, left)?;
                let (right, right_work) = scopes.equation_region(scope, right)?;
                let steps = self.equations.extend([(left, right)])?;
                context
                    .account_work(left_work + right_work + steps)
                    .map_err(inference)?;
            }
        }
        Ok(())
    }
}
impl Closure {
    fn new(
        query: &Query,
        context: &mut Context,
        scope: ScopeId,
        parent: ScopeId,
        caller_end: u32,
    ) -> Result<Self, String> {
        let (fold, steps) = query
            .scopes
            .as_ref()
            .ok_or("missing callable query scopes")?
            .closing(scope, parent, caller_end)?;
        context.account_work(steps).map_err(inference)?;
        Ok(Self { fold })
    }

    fn close(
        &mut self,
        facts: &mut Declarations,
        context: &mut Context,
        equations: &mut RegionEquations,
        ty: TypeId,
    ) -> Result<TypeId, String> {
        self.fold.exchange_projection_equations(equations)?;
        let before = self.fold.steps;
        let result = self.fold.apply(&mut facts.types, ty);
        self.fold.exchange_projection_equations(equations)?;
        context
            .account_work(self.fold.steps.saturating_sub(before))
            .map_err(inference)?;
        result.map_err(String::from)
    }
}
impl Query {
    pub(super) fn map_normalization_scopes(
        &mut self,
        bindings: &mut Vec<ScopeId>,
        root: ScopeId,
        needed: ScopeId,
        binder: impl FnMut(ScopeId) -> Result<(ScopeId, u32), &'static str>,
        context: &mut Context,
    ) -> Result<ScopeId, String> {
        if needed == ScopeId::ROOT {
            return Ok(root);
        }
        let scopes = self.scopes.get_or_insert_with(Default::default);
        let mut steps = 0;
        let result = map_scopes(bindings, root, needed, binder, |parent, count| {
            let first = self.allocations.reserve_scoped();
            let scope = scopes.introduce(parent, count, first)?;
            self.allocations.record(first + count);
            steps += count as usize + 1;
            Ok(scope)
        })?;
        context.account_work(steps).map_err(inference)?;
        Ok(result)
    }

    pub(super) fn open_normalization_type(
        &mut self,
        facts: &mut Declarations,
        context: &mut Context,
        scope: ScopeId,
        parent: ScopeId,
        ty: TypeId,
    ) -> Result<TypeId, String> {
        if scope == parent {
            return Ok(ty);
        }
        let (ty, steps) = self
            .scopes
            .as_mut()
            .ok_or("missing normalization scope")?
            .open_type(facts, scope, parent, ty)?;
        context.account_work(steps).map_err(inference)?;
        Ok(ty)
    }

    pub(super) fn normalization_equations(
        &mut self,
        node: usize,
        scope: ScopeId,
        equalities: Vec<crate::types::outlives::Pair>,
    ) -> usize {
        if equalities.is_empty() {
            return node;
        }
        let selected = &self.nodes[node];
        let (first, end) = (selected.first, selected.end);
        self.push(
            super::Node {
                cycle_dependent: false,
                provisional_size: false,
                goal: selected.goal,
                source: selected.source,
                children: vec![node],
                first,
                end,
                equalities,
                outlives: Vec::new(),
                type_outlives: Vec::new(),
                sized: Vec::new(),
            },
            scope,
            first..first,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        sema::traits::{assumptions::Assumptions, implementations::Availability},
        types::{Kind, Primitive, RegionId, parameters::predicates},
    };

    #[test]
    fn selected_prefixes_share_one_scoped_equation_traversal()
    -> Result<(), Box<dyn std::error::Error>> {
        for width in [16, 64, 256, 1024] {
            let (_, mut facts) = super::super::tests::declarations("fn main(){}")?;
            let assumptions = Assumptions::default();
            let mut query = super::super::Search::new(
                super::super::tests::caller(&assumptions, None),
                Availability::Validated,
            )
            .query;
            let scope = query
                .scopes
                .get_or_insert_with(Default::default)
                .introduce(ScopeId::ROOT, 1, 10)?;
            let mut context = Context::default();
            let mut scoped = Scoped::at_root(&query, &mut context, scope, ScopeId::ROOT, 10)?;
            let byte = TypeId::primitive(Primitive::U8);
            let local = RegionId::bound(0, 0)?;
            let expected = facts.types.intern(Kind::Reference {
                mutable: false,
                region: local,
                referent: byte,
            })?;
            let mut previous = None;
            for offset in 1..=width {
                let witness = RegionId(10 + offset);
                let node = query.push(
                    super::super::Node {
                        cycle_dependent: false,
                        provisional_size: false,
                        goal: predicates::Predicate {
                            ty: byte,
                            bound: predicates::Bound::Copy,
                        },
                        source: None,
                        children: previous.into_iter().collect(),
                        first: witness.0,
                        end: witness.0 + 1,
                        equalities: vec![(witness, local)],
                        outlives: Vec::new(),
                        type_outlives: Vec::new(),
                        sized: Vec::new(),
                    },
                    scope,
                    witness.0..witness.0,
                );
                let ty = facts.types.intern(Kind::Reference {
                    mutable: false,
                    region: witness,
                    referent: byte,
                })?;
                scoped.selected(
                    &query,
                    &mut facts,
                    &mut context,
                    scope,
                    node,
                    Resolution::Type(ty),
                )?;
                assert_eq!(scoped.complete(&mut facts, &mut context, ty)?, expected);
                assert_eq!(query.nodes[node].equalities, [(witness, local)]);
                previous = Some(node);
            }
            assert_eq!(scoped.seen.len(), width as usize);
            assert!(
                context.steps < width as usize * 30,
                "{width}: {}",
                context.steps
            );
        }
        Ok(())
    }
}
