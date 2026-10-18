//! Matching summaries retain conditional equalities and persistent slot bindings.
use super::super::{
    Constraints, Error, Region, RegionId, Scope, TypeId,
    identity_guard::View,
    persistent::{Id, Map},
    structural::Compare,
};
use crate::types::{GenericArgs, ListId, Store, region_graph::VerificationId};
use std::collections::HashMap;

#[derive(Clone, Copy, Default)]
pub(super) struct Summary {
    pub bindings: Id,
    pub guards: Id,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum Inputs {
    Types(TypeId, TypeId),
    Lists(ListId, ListId),
    Arguments(GenericArgs, GenericArgs),
}
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct Key {
    inputs: Inputs,
    nested: u16,
    depth: u16,
    scope: Scope,
}
struct Pending {
    summary: Summary,
    children: Vec<Key>,
}
#[derive(Default)]
pub(super) struct Recipes {
    completed: HashMap<Key, Option<Summary>>,
    pending: HashMap<Key, Pending>,
    work: Vec<(Key, bool)>,
    pub steps: usize,
}
impl Recipes {
    pub fn run(
        &mut self,
        proof: &mut Constraints<'_>,
        view: View,
        pattern: TypeId,
        bindings: &mut Map<Region>,
        guards: &mut Map<VerificationId>,
    ) -> Result<Option<Summary>, Error> {
        let root = Key {
            inputs: Inputs::Types(view.projection.ty, pattern),
            nested: 0,
            depth: view.projection.depth,
            scope: view.scope,
        };
        self.work.push((root, false));
        while let Some((key, finish)) = self.work.pop() {
            self.steps = self.steps.saturating_add(1);
            if self.steps.saturating_add(self.work.len()) > 1_000_000 {
                return Err(Error::Budget);
            }
            if self.completed.contains_key(&key) {
                continue;
            }
            if finish {
                let node = self.pending.remove(&key).ok_or(Error::Identity)?;
                let result = self.combine(node, bindings, guards)?;
                self.completed.insert(key, result);
                continue;
            }
            if let Inputs::Types(actual, pattern) = key.inputs
                && actual == pattern
                && proof
                    .types
                    .escaping_binders(actual)
                    .ok_or(Error::Identity)?
                    <= key.nested
            {
                self.completed.insert(key, Some(Summary::default()));
                continue;
            }
            let mut frame = Frame {
                proof,
                bindings,
                guards,
                key,
                node: Pending {
                    summary: Summary::default(),
                    children: Vec::new(),
                },
                steps: &mut self.steps,
            };
            if !frame.inspect()? {
                self.completed.insert(key, None);
                continue;
            }
            self.work.push((key, true));
            self.work.extend(
                frame
                    .node
                    .children
                    .iter()
                    .rev()
                    .map(|&child| (child, false)),
            );
            self.pending.insert(key, frame.node);
        }
        self.completed.get(&root).copied().ok_or(Error::Identity)
    }
    fn combine(
        &mut self,
        node: Pending,
        bindings: &mut Map<Region>,
        guards: &mut Map<VerificationId>,
    ) -> Result<Option<Summary>, Error> {
        let mut result = node.summary;
        for child in node.children {
            self.steps = self.steps.saturating_add(1);
            if self.steps > 1_000_000 {
                return Err(Error::Budget);
            }
            let Some(child) = self.completed.get(&child).copied().ok_or(Error::Identity)? else {
                return Ok(None);
            };
            let Some(merged) = bindings.union(result.bindings, child.bindings)? else {
                return Ok(None);
            };
            result.bindings = merged;
            result.guards = guards
                .union(result.guards, child.guards)?
                .ok_or(Error::Identity)?;
        }
        Ok(Some(result))
    }
}
struct Frame<'a, 't> {
    proof: &'a mut Constraints<'t>,
    bindings: &'a mut Map<Region>,
    guards: &'a mut Map<VerificationId>,
    key: Key,
    node: Pending,
    steps: &'a mut usize,
}
impl Frame<'_, '_> {
    fn inspect(&mut self) -> Result<bool, Error> {
        match self.key.inputs {
            Inputs::Types(a, b) => self.compare(a, b, self.key.nested),
            Inputs::Lists(a, b) => self.compare_list(a, b, self.key.nested),
            Inputs::Arguments(a, b) => self.compare_arguments(a, b, self.key.nested),
        }
    }
    fn enqueue(&mut self, inputs: Inputs, nested: u16) -> Result<(), Error> {
        self.charge(1)?;
        self.node.children.push(Key {
            inputs,
            nested,
            ..self.key
        });
        Ok(())
    }
    fn resolve(&self, region: RegionId, depth: u16, scope: Scope) -> Result<Region, Error> {
        let region = match region.bound_parts() {
            Some((binder, slot)) => {
                RegionId::bound(binder.checked_sub(depth).ok_or(Error::Identity)?, slot)
                    .map_err(|_| Error::Identity)?
            }
            None => region,
        };
        self.proof.region(region, scope)
    }
}
impl<'t> Compare<'t> for Frame<'_, 't> {
    fn types(&self) -> &'t Store {
        self.proof.types
    }
    fn charge(&mut self, count: usize) -> Result<(), Error> {
        *self.steps = self.steps.saturating_add(count);
        if *self.steps > 1_000_000 {
            return Err(Error::Budget);
        }
        Ok(())
    }
    fn child(&mut self, actual: TypeId, pattern: TypeId, nested: u16) -> Result<(), Error> {
        self.enqueue(Inputs::Types(actual, pattern), nested)
    }
    fn arguments(
        &mut self,
        actual: GenericArgs,
        pattern: GenericArgs,
        nested: u16,
    ) -> Result<bool, Error> {
        self.enqueue(Inputs::Arguments(actual, pattern), nested)?;
        Ok(true)
    }
    fn list(&mut self, actual: ListId, pattern: ListId, nested: u16) -> Result<bool, Error> {
        self.enqueue(Inputs::Lists(actual, pattern), nested)?;
        Ok(true)
    }
    fn regions(&mut self, actual: RegionId, pattern: RegionId, nested: u16) -> Result<bool, Error> {
        let local = |region: RegionId| {
            region
                .bound_parts()
                .is_some_and(|(binder, _)| binder < nested)
        };
        if let Some((binder, slot)) = pattern.bound_parts()
            && binder == nested
        {
            // A surrounding premise slot cannot capture a lifetime introduced
            // inside the requested callable. Its substitution stays in the
            // request's actual proof scope.
            if local(actual) {
                return Ok(false);
            }
            let value = self.resolve(
                actual,
                nested.checked_add(self.key.depth).ok_or(Error::Budget)?,
                self.key.scope,
            )?;
            let leaf = self.bindings.singleton(u32::from(slot), value)?;
            let Some(bindings) = self.bindings.union(self.node.summary.bindings, leaf)? else {
                return Ok(false);
            };
            self.node.summary.bindings = bindings;
            return Ok(true);
        }
        if local(actual) || local(pattern) {
            return Ok(actual == pattern);
        }
        let a = self.resolve(
            actual,
            nested.checked_add(self.key.depth).ok_or(Error::Budget)?,
            self.key.scope,
        )?;
        let b = self.resolve(
            pattern,
            nested.checked_add(1).ok_or(Error::Budget)?,
            Scope(0),
        )?;
        if a != b {
            let forward = self.proof.graph.verify_outlives(a, b)?;
            let reverse = self.proof.graph.verify_outlives(b, a)?;
            let condition = self.proof.graph.verify_all(&[forward, reverse])?;
            let leaf = self.guards.singleton(condition.0, condition)?;
            self.node.summary.guards = self
                .guards
                .union(self.node.summary.guards, leaf)?
                .ok_or(Error::Identity)?;
        }
        Ok(true)
    }
}
