//! Sparse universe metadata preserves selected graph debts without copying ancestor proofs.
use super::{Decision, Declarations, Evidence, Node, Predicate, Query};
use crate::types::{
    RegionId,
    lifetime_proof::{Obligations, ScopeId},
    substitute::Bound,
};
use std::{collections::HashMap, ops::Range};

struct Universe {
    parent: ScopeId,
    count: u32,
    aliases: Range<u32>,
    opening: Bound,
    proof: Obligations,
    selected: bool,
}
struct Local {
    scope: ScopeId,
    regions: Range<u32>,
    nested: bool,
}
struct Reservation {
    scope: ScopeId,
    regions: Range<u32>,
    alias: bool,
    selected: bool,
}
#[derive(Default)]
pub(super) struct Scopes {
    universes: Vec<Universe>,
    nodes: HashMap<usize, Local>,
    completed: HashMap<(ScopeId, Predicate), Decision>,
    reservations: Vec<Reservation>,
    by_start: HashMap<u32, usize>,
}
impl Query {
    pub(super) fn reserve(
        &mut self,
        scope: ScopeId,
        regions: Range<u32>,
    ) -> Result<(), &'static str> {
        if let Some(scopes) = &mut self.scopes {
            scopes.reserve(scope, regions, false)?;
        }
        Ok(())
    }
    pub(super) fn push(&mut self, node: Node, scope: ScopeId, regions: Range<u32>) -> usize {
        let index = self.nodes.len();
        if let Some(scopes) = &mut self.scopes {
            let nested = node.children.iter().any(|child| {
                scopes
                    .nodes
                    .get(child)
                    .is_some_and(|child| child.scope != scope || child.nested)
            });
            if nested || scope != ScopeId::ROOT {
                scopes.nodes.insert(
                    index,
                    Local {
                        scope,
                        regions,
                        nested,
                    },
                );
            }
        }
        self.nodes.push(node);
        index
    }
    pub(super) fn node_scope(&self, node: usize) -> ScopeId {
        self.scopes
            .as_ref()
            .and_then(|scopes| scopes.nodes.get(&node))
            .map_or(ScopeId::ROOT, |node| node.scope)
    }
    pub(super) fn cacheable(&self, node: usize) -> bool {
        self.scopes
            .as_ref()
            .and_then(|scopes| scopes.nodes.get(&node))
            .is_none_or(|node| !node.nested)
    }
    pub(super) fn completed(&self, scope: ScopeId, goal: Predicate) -> Option<Decision> {
        if scope == ScopeId::ROOT {
            self.completed.get(&goal).copied()
        } else {
            self.scopes.as_ref()?.completed.get(&(scope, goal)).copied()
        }
    }
    pub(super) fn remember(&mut self, scope: ScopeId, goal: Predicate, decision: Decision) {
        if scope == ScopeId::ROOT {
            self.completed.insert(goal, decision);
        } else if let Some(scopes) = &mut self.scopes {
            scopes.completed.insert((scope, goal), decision);
        }
    }
}
impl Scopes {
    fn reserve(
        &mut self,
        scope: ScopeId,
        regions: Range<u32>,
        alias: bool,
    ) -> Result<(), &'static str> {
        if scope == ScopeId::ROOT || regions.is_empty() {
            return Ok(());
        }
        if scope.0 as usize > self.universes.len()
            || regions.start > regions.end
            || self
                .reservations
                .last()
                .is_some_and(|previous| previous.regions.end > regions.start)
        {
            return Err("invalid quantified impl allocation order");
        }
        self.by_start.insert(regions.start, self.reservations.len());
        self.reservations.push(Reservation {
            scope,
            regions,
            alias,
            selected: false,
        });
        Ok(())
    }
    pub fn introduce(
        &mut self,
        parent: ScopeId,
        count: u32,
        first: u32,
    ) -> Result<ScopeId, &'static str> {
        if parent.0 as usize > self.universes.len()
            || self.universes.len() >= 1_000_000
            || count > 65_536
        {
            return Err("quantified impl scope budget exhausted");
        }
        let end = first
            .checked_add(count)
            .filter(|end| *end < 0x8000_0000)
            .ok_or("quantified impl region identity space exhausted")?;
        self.universes.push(Universe {
            parent,
            count,
            aliases: first..end,
            opening: Bound::new((first..end).map(RegionId).collect()),
            proof: Obligations::new(0),
            selected: false,
        });
        let scope =
            ScopeId(u32::try_from(self.universes.len()).map_err(|_| "too many impl scopes")?);
        self.reserve(scope, first..end, true)?;
        Ok(scope)
    }
    pub fn open(
        &mut self,
        facts: &mut Declarations,
        scope: ScopeId,
        predicate: Predicate,
        first: u32,
    ) -> Result<(Predicate, u32, usize), &'static str> {
        let universe = self
            .universes
            .get_mut(scope.0 as usize - 1)
            .ok_or("missing impl universe")?;
        let before = universe.opening.steps;
        let predicate = universe.opening.predicate(&mut facts.types, predicate)?;
        Ok((
            predicate,
            first.max(universe.aliases.end),
            universe
                .opening
                .steps
                .saturating_sub(before)
                .saturating_add(1),
        ))
    }
    pub fn collect(
        &mut self,
        index: usize,
        node: &mut Node,
        evidence: &mut Evidence,
    ) -> Result<(), &'static str> {
        let local = self
            .nodes
            .get(&index)
            .filter(|local| local.scope != ScopeId::ROOT);
        let proof = if let Some(local) = local {
            if !node.sized.is_empty() {
                return Err("quantified impl proof retained unscoped sized requirements");
            }
            let universe = self
                .universes
                .get_mut(local.scope.0 as usize - 1)
                .ok_or("missing selected impl universe")?;
            universe.selected = true;
            if !local.regions.is_empty() {
                let position = self
                    .by_start
                    .get(&local.regions.start)
                    .copied()
                    .ok_or("missing selected impl allocation")?;
                let reservation = &mut self.reservations[position];
                if reservation.scope != local.scope
                    || reservation.regions != local.regions
                    || reservation.alias
                {
                    return Err("selected impl allocation changed ownership");
                }
                reservation.selected = true;
            }
            &mut universe.proof
        } else {
            evidence.sized.append(&mut node.sized);
            &mut evidence.proof
        };
        proof.equalities.append(&mut node.equalities);
        proof.outlives.append(&mut node.outlives);
        proof.types.append(&mut node.type_outlives);
        Ok(())
    }
    pub fn finish(mut self, proof: &mut Obligations) -> Result<(), &'static str> {
        for index in (0..self.universes.len()).rev() {
            let universe = &self.universes[index];
            if universe.selected && universe.parent != ScopeId::ROOT {
                let parent = universe.parent.0 as usize - 1;
                self.universes[parent].selected = true;
            }
        }
        let mut bindings = vec![ScopeId::ROOT];
        for universe in &mut self.universes {
            let scope = if universe.selected {
                let scope = proof.scope(bindings[universe.parent.0 as usize], universe.count)?;
                for (slot, region) in universe.aliases.clone().enumerate() {
                    universe.proof.equalities.push((
                        RegionId(region),
                        RegionId::bound(
                            0,
                            u16::try_from(slot)
                                .map_err(|_| "too many quantified impl lifetimes")?,
                        )?,
                    ));
                }
                scope
            } else {
                ScopeId::ROOT
            };
            bindings.push(scope);
        }
        for reservation in self.reservations {
            if !(reservation.selected
                || reservation.alias && self.universes[reservation.scope.0 as usize - 1].selected)
            {
                continue;
            }
            let mut allocation = Obligations::new(reservation.regions.start);
            allocation.regions.end = reservation.regions.end;
            proof.place_at(allocation, bindings[reservation.scope.0 as usize])?;
        }
        for (index, mut universe) in self.universes.into_iter().enumerate() {
            if universe.selected {
                universe.proof.regions = proof.regions.end..proof.regions.end;
                proof.place_at(universe.proof, bindings[index + 1])?;
            }
        }
        Ok(())
    }
}
