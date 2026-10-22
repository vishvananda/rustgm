//! Opaque result bounds are alternatives, retained until symbolic and runtime proof.
use super::{Constraints, Error, Region, Scope, identity_guard::View};
use crate::support::hash::{HashMap, HashSet};
use crate::types::{
    Kind,
    outlives::{Projection, ResultId, TypeComponents},
    region_graph::VerificationId,
};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct Assumption {
    projection: Projection,
    scope: Scope,
    longer: Region,
}
#[derive(Clone, Copy)]
pub(super) struct Request {
    pub view: View,
    pub shorter: Region,
    pub source: u32,
}
#[derive(Default)]
pub(super) struct State {
    assumptions: HashMap<ResultId, Vec<Assumption>>,
    unique: HashSet<Assumption>,
    pub pending: Vec<Request>,
    components: TypeComponents,
    pub(super) quantified: Option<Box<super::universal_bounds::State>>,
}
impl State {
    pub(super) fn test(
        &mut self,
        proof: &mut Constraints<'_>,
        view: View,
        shorter: Region,
    ) -> Result<VerificationId, Error> {
        Builder {
            proof,
            state: self,
            source: 0,
            nodes: HashMap::default(),
            done: HashMap::default(),
            work: Vec::new(),
        }
        .build(Key {
            projection: view.projection,
            scope: view.scope,
            shorter,
            forced: false,
        })
    }
    pub fn assume(&mut self, item: ResultId, view: View, longer: Region) {
        let value = Assumption {
            projection: view.projection,
            scope: view.scope,
            longer,
        };
        if self.unique.insert(value) {
            self.assumptions.entry(item).or_default().push(value);
        }
    }
    pub fn prove(&mut self, proof: &mut Constraints<'_>) -> Result<(), Error> {
        for request in std::mem::take(&mut self.pending) {
            Builder {
                proof,
                state: self,
                source: request.source,
                nodes: HashMap::default(),
                done: HashMap::default(),
                work: Vec::new(),
            }
            .run(request)?;
        }
        Ok(())
    }
}
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct Key {
    projection: Projection,
    scope: Scope,
    shorter: Region,
    forced: bool,
}
struct Node {
    alternatives: Vec<VerificationId>,
    inputs: Vec<VerificationId>,
    children: Vec<Key>,
}
struct Builder<'a, 't> {
    proof: &'a mut Constraints<'t>,
    state: &'a mut State,
    source: u32,
    nodes: HashMap<Key, Node>,
    done: HashMap<Key, VerificationId>,
    work: Vec<(Key, bool)>,
}
impl Builder<'_, '_> {
    fn charge(&mut self, count: usize) -> Result<(), Error> {
        self.proof.steps = self.proof.steps.saturating_add(count);
        if self.proof.steps.saturating_add(self.work.len()) > self.proof.limit {
            return Err(Error::Budget);
        }
        Ok(())
    }
    fn run(&mut self, request: Request) -> Result<(), Error> {
        let root = Key {
            projection: request.view.projection,
            scope: request.view.scope,
            shorter: request.shorter,
            forced: true,
        };
        let result = self.build(root)?;
        self.proof.graph.require_verification(result, self.source)
    }
    fn build(&mut self, root: Key) -> Result<VerificationId, Error> {
        self.work.push((root, false));
        while let Some((key, finish)) = self.work.pop() {
            self.charge(1)?;
            if self.done.contains_key(&key) {
                continue;
            }
            if finish {
                let mut node = self.nodes.remove(&key).ok_or(Error::Identity)?;
                for child in node.children {
                    node.inputs
                        .push(*self.done.get(&child).ok_or(Error::Identity)?);
                }
                self.charge(node.inputs.len().saturating_add(node.alternatives.len()))?;
                node.alternatives
                    .push(self.proof.graph.verify_all(&node.inputs)?);
                let result = self.proof.graph.verify_any(&node.alternatives)?;
                self.done.insert(key, result);
            } else {
                let node = self.node(key)?;
                self.charge(node.children.len().saturating_add(1))?;
                self.work.push((key, true));
                self.work
                    .extend(node.children.iter().rev().map(|&child| (child, false)));
                self.nodes.insert(key, node);
            }
        }
        self.done.get(&root).copied().ok_or(Error::Identity)
    }
    fn alternatives(&mut self, key: Key) -> Result<(Vec<VerificationId>, Option<Region>), Error> {
        let mut alternatives = Vec::new();
        let types = self.proof.types;
        let mut unique = match types.kind(key.projection.ty) {
            Some(Kind::Projection(application)) => self.declared(
                key,
                types
                    .projection_outlives(application, key.projection.depth)
                    .map_err(|_| Error::Identity)?,
                &mut alternatives,
            )?,
            Some(Kind::Opaque(id, captures)) => self.declared(
                key,
                types.opaque_outlives(id, captures, key.projection.depth),
                &mut alternatives,
            )?,
            _ => return Err(Error::Identity),
        };
        let identity = types
            .result_identity(key.projection.ty)
            .ok_or(Error::Identity)?;
        if self
            .state
            .components
            .captures(self.proof.types, key.projection)
            .map_err(|_| Error::Identity)?
        {
            return Ok((alternatives, unique));
        }
        if let Some(patterns) = &mut self.state.quantified {
            let before = patterns.steps();
            let found = patterns.alternatives(
                self.proof,
                View {
                    projection: key.projection,
                    scope: key.scope,
                },
                key.shorter,
            );
            let work = patterns.steps().saturating_sub(before);
            self.charge(work)?;
            let found = found?;
            if !found.is_empty() {
                unique = None;
            }
            alternatives.extend(found);
        }
        for index in 0..self.state.assumptions.get(&identity).map_or(0, Vec::len) {
            self.charge(1)?;
            let assumption = self.state.assumptions[&identity][index];
            if self
                .state
                .components
                .captures(self.proof.types, assumption.projection)
                .map_err(|_| Error::Identity)?
            {
                continue;
            }
            if let Some(mut guard) = self.proof.projection_guards(
                View {
                    projection: key.projection,
                    scope: key.scope,
                },
                View {
                    projection: assumption.projection,
                    scope: assumption.scope,
                },
            )? {
                if unique != Some(assumption.longer) {
                    unique = None;
                }
                guard.push(
                    self.proof
                        .graph
                        .verify_outlives(assumption.longer, key.shorter)?,
                );
                alternatives.push(self.proof.graph.verify_all(&guard)?);
            }
        }
        Ok((alternatives, unique))
    }
    fn declared(
        &mut self,
        key: Key,
        promises: impl Iterator<Item = Result<Option<crate::types::RegionId>, &'static str>>,
        alternatives: &mut Vec<VerificationId>,
    ) -> Result<Option<Region>, Error> {
        let mut unique = None;
        let mut multiple = false;
        for longer in promises {
            self.charge(1)?;
            if let Some(longer) = longer.map_err(|_| Error::Identity)? {
                let longer = self.proof.region(longer, key.scope)?;
                multiple |= unique.is_some_and(|prior| prior != longer);
                unique = Some(longer);
                alternatives.push(self.proof.graph.verify_outlives(longer, key.shorter)?);
            } else {
                multiple = true;
            }
        }
        Ok(unique.filter(|_| !multiple))
    }
    fn node(&mut self, key: Key) -> Result<Node, Error> {
        let (alternatives, unique) = self.alternatives(key)?;
        // A sole declared target also occurs in the alias inputs. With no
        // competing caller promise, every available rule needs this edge.
        // Retain it for inference instead of merely testing the least solution.
        if key.forced
            && let Some(longer) = unique
        {
            self.proof.graph.require(longer, key.shorter, self.source)?;
        }
        let forced = key.forced && alternatives.is_empty();
        let parts = self
            .state
            .components
            .projection_inputs(self.proof.types, key.projection)
            .map_err(|_| Error::Identity)?;
        let mut inputs = Vec::new();
        for &longer in parts.scalars.regions {
            let longer = self.proof.region(longer, key.scope)?;
            if forced {
                self.proof.graph.require(longer, key.shorter, self.source)?;
            }
            inputs.push(self.proof.graph.verify_outlives(longer, key.shorter)?);
        }
        for &id in parts.scalars.parameters {
            let longer = *self.proof.parameters.get(&id).ok_or(Error::Identity)?;
            if forced {
                self.proof.graph.require(longer, key.shorter, self.source)?;
            }
            inputs.push(self.proof.graph.verify_outlives(longer, key.shorter)?);
        }
        let children = parts
            .projections
            .iter()
            .map(|&projection| Key {
                projection,
                forced,
                ..key
            })
            .collect();
        self.charge(inputs.len())?;
        Ok(Node {
            alternatives,
            inputs,
            children,
        })
    }
}
